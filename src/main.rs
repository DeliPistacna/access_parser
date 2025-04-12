mod cli_options;
mod config_reader;
mod file_reader;
mod free_ip_api;
mod ip_info;
mod ip_location;
mod log_entry;
mod log_processor;
mod printer;
mod slack_webhook;

use chrono::{DateTime, Local};
use clap::Parser;
use cli_options::CliOptions;
use file_reader::FileReader;
use free_ip_api::FreeIpApi;
use ip_info::IpInfo;
use log_entry::LogEntry;
use log_processor::{LogProcessor, ParseType};
use printer::Printer;
use slack_webhook::{Message, SlackWebhook};

use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet, hash_map::Entry},
    error::Error,
    path::Path,
    process::exit,
    time::{Duration, Instant},
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
    let config = config_reader::get_config(Path::new("./slack.config"))?;
    let slack_webhook = config
        .get("webhook")
        .map(|webhook| SlackWebhook::new(webhook.to_string()));

    let timer = Instant::now();
    let opts = CliOptions::parse();
    let mut ip_map: HashMap<String, IpInfo> = HashMap::new();
    let file_reader = FileReader::new(opts.file_path.clone())?;
    let mut line_count = 0;
    let mut most_recent_timestamp: chrono::DateTime<Local> = DateTime::default();
    let mut filter_timestamp: chrono::DateTime<Local> = chrono::Local::now();
    let filter_hours = opts.filter_hours.is_some();

    // Quick pass over file, collecting ips
    // This is used to create HashSet of n most common top_ips
    // HashSet will be used to filter in second pass where more data is parsed and collected

    // REFACTOR
    let mut log_processor = LogProcessor::new(&file_reader, opts.filter_hours);
    match opts.filter_hours {
        Some(_filter_hours) => {
            log_processor.process_log(&mut ip_map, ParseType::IpAndTimestamp)?;
            // TODO: filter here
        }
        None => {
            log_processor.process_log(&mut ip_map, ParseType::IpOnly)?;
        }
    }

    // TODO: Filter top ips
    {
        let mut top_ips = ip_map_to_vect(&ip_map);
        top_ips.truncate(opts.max_ips);
        let top_ip_set: HashSet<String> = top_ips.into_iter().map(|(ip, _)| ip.clone()).collect();
        log_processor.filter_ips = top_ip_set;
    }

    log_processor.process_log(&mut ip_map, ParseType::Full)?;
    // REFACTOR_END
    let collected_ips: HashSet<String> = ip_map.keys().cloned().collect();

    let elapsed = timer.elapsed();
    let mut time_fetching = Duration::default();

    // Fetch location info for collected ips
    // if !opts.ignore_location {
    //     let timer = Instant::now();
    //     let location_data = FreeIpApi::get_loc_info(collected_ips).await?;
    //     for loc in location_data {
    //         let ip = match &loc.ip_address {
    //             Some(ip) => ip,
    //             None => continue,
    //         };
    //
    //         if let Entry::Occupied(mut entry) = ip_map.entry(ip.to_string()) {
    //             entry.get_mut().location_data = Some(loc);
    //         }
    //         {}
    //     }
    //     time_fetching = timer.elapsed();
    // }

    let ip_vec = ip_map_to_vect(&ip_map);
    let printer = Printer::new(opts.colors);
    let mut ln = 0;
    for (ip, ip_info) in ip_vec {
        ln += 1;
        printer.ip(ln, ip, ip_info);
        // let whois = whoiz::fetch(&ip)?;
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
            println!();
            // if !opts.ignore_location {
            //     printer.location(ip_info.location_data);
            // }
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
