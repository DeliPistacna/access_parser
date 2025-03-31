mod file_reader;
mod free_ip_api;
mod ip_info;
mod ip_location;
mod log_entry;
mod printer;

use chrono::{DateTime, Local};
use clap::{ArgAction, Parser};
use file_reader::FileReader;
use free_ip_api::FreeIpApi;
use ip_info::IpInfo;
//use ip_location::IpLocation;
use log_entry::LogEntry;
use printer::Printer;
//use tokio;

use std::{
    cmp::Reverse,
    collections::{hash_map::Entry, HashMap, HashSet},
    error::Error,
    path::PathBuf,
    time::{Duration, Instant},
    usize,
};
#[derive(Parser, Debug)]
#[command(name = "accessparser")]
#[command(about = "A program to parse access logs and retrieve top IP addresses")]
struct CliOptions {
    #[arg(value_name = "FILE_PATH")]
    file_path: PathBuf,

    #[arg(short, long, default_value_t = 15)]
    max_ips: usize,

    #[arg(short, long, default_value_t = 3)]
    top_params: usize,

    #[arg(short = 'h', long)]
    filter_hours: Option<u16>,

    #[arg(short = 'l', long = "ignore-location", default_value_t = false)]
    ignore_location: bool,

    #[arg(short = 'c', long = "no-colors", default_value_t = true, action= ArgAction::SetFalse)]
    colors: bool,

    #[arg(short = 'f', long = "no-footer", default_value_t = true, action= ArgAction::SetFalse)]
    footer: bool,
}

fn ip_map_to_vect(ip_map: HashMap<String, IpInfo>) -> Vec<(String, IpInfo)> {
    let mut ip_vec: Vec<(String, IpInfo)> = ip_map.into_iter().collect();
    ip_vec.sort_unstable_by_key(|(_, info)| Reverse(info.count));
    ip_vec
}

fn count_hashmap_to_vect(map: HashMap<String, usize>) -> Vec<(String, usize)> {
    let mut vec: Vec<(String, usize)> = map.into_iter().collect();
    vec.sort_unstable_by_key(|&(_, count)| Reverse(count));
    vec
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let timer = Instant::now();
    let opts = CliOptions::parse();
    let mut ip_map: HashMap<String, IpInfo> = HashMap::new();
    let file_reader = FileReader::new(opts.file_path.clone()).unwrap();
    let mut line_count = 0;

    // Quick pass over file, collecting ips
    // This is used to create HashSet of n most common top_ips
    // HashSet will be used to filter in second pass where more data is parsed and collected

    let mut most_recent_timestamp: chrono::DateTime<Local> = DateTime::default();

    for line in file_reader.get_lines().unwrap() {
        let line = line?;

        // If hours filter is present we need to parse timestamps too
        if let Some(_hours) = opts.filter_hours {
            let (ip, timestamp) = LogEntry::parse_ip_and_timestamp(&line);
            if let Some(ip) = ip {
                let entry = ip_map.entry(ip.to_string()).or_insert_with(IpInfo::new);
                entry.increment();

                if let Some(timestamp) = timestamp {
                    if timestamp > most_recent_timestamp {
                        most_recent_timestamp = timestamp
                    }
                    entry.timestamps.push(timestamp);
                }
            }
        } else {
            // No timestamp needed without filter
            let ip_opt = LogEntry::parse_ip(&line);
            if let Some(ip) = ip_opt {
                let entry = ip_map.entry(ip.to_string()).or_insert_with(IpInfo::new);
                entry.increment();
            }
        }
    }

    // Filter results by timestamps if needed
    let mut filter_timestamp: chrono::DateTime<Local> = chrono::Local::now();
    let filter_hours = opts.filter_hours.is_some();

    if let Some(hours) = opts.filter_hours {
        filter_timestamp = most_recent_timestamp - chrono::Duration::hours(hours as i64);
        // Collect IPs to remove first
        let ips_to_remove: Vec<String> = ip_map
            .iter()
            .filter(|(_, entry)| entry.timestamps.iter().any(|&t| t < filter_timestamp))
            .map(|(ip, _)| ip.clone())
            .collect();

        // Remove collected IPs
        for ip in ips_to_remove {
            ip_map.remove(&ip);
        }
    }

    let mut top_ips = ip_map_to_vect(ip_map.clone());
    top_ips.truncate(opts.max_ips);
    let top_ip_set: HashSet<String> = top_ips.into_iter().map(|(ip, _)| ip).collect();

    // Second pass -> collect all data

    let file_reader = FileReader::new(opts.file_path).unwrap();
    for line in file_reader.get_lines().unwrap() {
        line_count += 1;
        let line = line?;
        let ip_opt = LogEntry::parse_ip(&line);
        if let Some(ip) = ip_opt {
            if !top_ip_set.contains(ip) {
                ip_map.remove(ip);
                continue;
            }

            let mut log_entry = LogEntry::parse(&line);

            if filter_hours {
                if let Some(timestamp) = log_entry.timestamp {
                    if timestamp < filter_timestamp {
                        continue;
                    }
                }
                log_entry.timestamp = None;
            }

            let entry = ip_map.entry(ip.to_string()).or_insert_with(IpInfo::new);

            entry.collect_entry(log_entry);
        }
    }

    let collected_ips: HashSet<String> = ip_map.keys().cloned().collect();

    let elapsed = timer.elapsed();
    let mut time_fetching = Duration::default();

    // Fetch location info for collected ips
    if !opts.ignore_location {
        let timer = Instant::now();
        let location_data = FreeIpApi::get_loc_info(collected_ips).await?;
        for loc in location_data {
            let ip = match &loc.ip_address {
                Some(ip) => ip,
                None => continue,
            };

            if let Entry::Occupied(mut entry) = ip_map.entry(ip.to_string()) {
                entry.get_mut().location_data = Some(loc);
            }
        }
        time_fetching = timer.elapsed();
    }

    let ip_vec = ip_map_to_vect(ip_map.clone());
    let printer = Printer::new(opts.colors);
    let mut ln = 0;
    for (ip, ip_info) in ip_vec {
        ln += 1;
        printer.ip(ln, &ip, &ip_info);
        if opts.top_params > 0 {
            println!();
            printer.list(
                count_hashmap_to_vect(ip_info.url_map),
                "URL",
                opts.top_params,
            );
            println!();
            printer.list(
                count_hashmap_to_vect(ip_info.referrer_map),
                "Referrer",
                opts.top_params,
            );
            println!();
            printer.list(count_hashmap_to_vect(ip_info.ua_map), "UA", opts.top_params);
            println!();
            if !opts.ignore_location {
                printer.location(ip_info.location_data);
            }
            println!();
        }
    }

    if opts.footer {
        printer.footer(
            line_count,
            elapsed.as_millis(),
            time_fetching.as_millis(),
            opts.ignore_location,
        );
    }

    Ok(())
}
