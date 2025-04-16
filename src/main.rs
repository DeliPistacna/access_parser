mod cli_options;
mod cache;
mod config_reader;
mod file_reader;
mod free_ip_api;
mod ip_info;
mod ip_location;
mod log_entry;
mod log_processor;
mod printer;
mod slack_webhook;

use cache::Cache;
use clap::Parser;
use cli_options::CliOptions;
use file_reader::FileReader;
use free_ip_api::FreeIpApi;
use ip_info::IpInfo;
use log_processor::{LogProcessor, ParseType};
use printer::Printer;
// use slack_webhook::{Message, SlackWebhook};

use std::{
    cmp::Reverse, collections::{HashMap, HashSet}, error::Error, process::exit, time::{Duration, Instant}
};

fn ip_map_to_vect(ip_map: &HashMap<String, IpInfo>) -> Vec<(&String, &IpInfo)> {
    let mut ip_vec: Vec<(&String, &IpInfo)> = ip_map.iter().collect();
    ip_vec.sort_unstable_by_key(|(_, info)| Reverse(info.count));
    ip_vec
}

fn count_hashmap_to_vect(map: &HashMap<String, usize>) -> Vec<(&String, &usize)> {
    let mut vec: Vec<(&String, &usize)> = map.iter().collect();
    vec.sort_unstable_by_key(|&(_, count)| Reverse(count));
    vec
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let timer = Instant::now();
    let opts = CliOptions::parse();
    let mut ip_map: HashMap<String, IpInfo> = HashMap::new();
    let file_reader = FileReader::new(opts.file_path.clone())?;
    let mut log_processor = LogProcessor::new(&file_reader, opts.filter_hours);
    // let filter_hours = opts.filter_hours.is_some();

    let line_count = match opts.filter_hours {
        Some(_filter_hours) => log_processor.process_log(&mut ip_map, ParseType::IpAndTimestamp)?,
        None => log_processor.process_log(&mut ip_map, ParseType::IpOnly)?,
    };


    log_processor.filter_ips = ip_map_to_vect(&ip_map)
        .into_iter()
        .take(opts.max_ips)
        .map(|(ip, _)| ip.to_string() )
        .collect();
    log_processor.process_log(&mut ip_map, ParseType::Full)?;

    let elapsed = timer.elapsed();


    let mut time_fetching = Duration::default();
    if !opts.ignore_location {

        for loc in  FreeIpApi::get_loc_info(log_processor.filter_ips.clone()).await? {
            if let Some(ip) = loc.ip_address.clone() {
                if ip_map.contains_key(&ip) {
                    ip_map.entry(ip)
                        .and_modify(|data| data.location_data = Some(loc));
                }
            }
        }
        time_fetching = timer.elapsed();
    }

    let ip_vec = ip_map_to_vect(&ip_map);
    let printer = Printer::new(opts.colors);
    let mut ln = 0;
    for (ip, ip_info) in ip_vec {
        ln += 1;
        printer.ip(ln, ip, ip_info);
        // let whois = whoiz::fetch(&ip.clone())?;
        // println!("{}", whois.get_org_name().unwrap_or("UNK".to_string()));
        // println!("{}", whois.get_net_name().unwrap_or("UNK".to_string()));
        if opts.top_params > 0 {
            println!();
            printer.list(
                count_hashmap_to_vect(&ip_info.url_map),
                "URL",
                opts.top_params,
            );
            println!();
            printer.list(
                count_hashmap_to_vect(&ip_info.referrer_map),
                "Referrer",
                opts.top_params,
            );
            println!();
            printer.list(
                count_hashmap_to_vect(&ip_info.ua_map),
                "UA",
                opts.top_params,
            );
            if !opts.ignore_location {
                if let Some(loc) = &ip_info.location_data {
                    println!();
                    printer.location(loc.clone());
                }
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
