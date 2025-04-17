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

use clap::{builder::Str, Parser};
use cli_options::CliOptions;
use free_ip_api::FreeIpApi;
use ip_info::IpInfo;
use log_processor::{LogProcessor, ParseType};
use printer::Printer;
use reqwest::header;
use slack_webhook::{Message, SlackWebhook};

use std::{
    cmp::Reverse, collections::HashMap, error::Error, time::{Duration, Instant}
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
    let mut log_processor = LogProcessor::new(&opts.file_path, opts.filter_hours)?;

    let line_count = match opts.filter_hours {
        Some(_filter_hours) => log_processor.process_log(&mut ip_map, ParseType::IpAndTimestamp)?,
        None => log_processor.process_log(&mut ip_map, ParseType::IpOnly)?,
    };

    if opts.max_ips != 0 {
        log_processor.filter_ips = ip_map_to_vect(&ip_map)
            .into_iter()
            .take(opts.max_ips)
            .map(|(ip, _)| ip.to_string() )
            .collect();
    }

    match opts.filter_hours {
        // when filtering by time run in reverse
        Some(_) => { log_processor.process_log(&mut ip_map, ParseType::FullReverse)?; },
        None => { log_processor.process_log(&mut ip_map, ParseType::Full)?; },
    }
    


    // Filter RPM | Requests
    if opts.filter_rpm.is_some() || opts.filter_requests.is_some(){
        for (ip,ip_info) in ip_map.clone() {

            if let Some(min_rpm) = opts.filter_rpm {
                if ip_info.average_rpm() < min_rpm as f64 {
                    ip_map.remove(&ip);
                    continue;
                } 
            }

            if let Some(min_requests) = opts.filter_requests {
                if ip_info.count < min_requests {
                    ip_map.remove(&ip);
                    continue;
                } 
            }

        }
    }

    let elapsed = timer.elapsed();

    let mut time_fetching = Duration::default();
    if opts.geolocate && !ip_map.is_empty() {
        let ip_set = ip_map.keys().cloned().collect();
        for loc in  FreeIpApi::get_loc_info(ip_set ).await? {
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


    let mut output_buff = String::new();

    let printer = Printer::new(opts.colors);
    let mut ln = 0;
    for (ip, ip_info) in ip_vec.clone() {

        ln += 1;
        output_buff += &printer.ip(ln, ip, ip_info, log_processor.get_latest_timestamp());
        if opts.geolocate {
            if let Some(loc) = &ip_info.location_data {
                output_buff += &printer.location(loc.clone());
                output_buff += "\n";
            }
        }
        if opts.top_params > 0 {
            output_buff += "\n";
            output_buff += &printer.list(
                count_hashmap_to_vect(&ip_info.url_map),
                "URL",
                opts.top_params,
            );
            output_buff += "\n";
            output_buff += &printer.list(
                count_hashmap_to_vect(&ip_info.referrer_map),
                "Referrer",
                opts.top_params,
            );
            output_buff += "\n";
            output_buff += &printer.list(
                count_hashmap_to_vect(&ip_info.ua_map),
                "UA",
                opts.top_params,
            );
            output_buff += "\n";
        }

    }

    if opts.footer {
        output_buff += &printer.footer(
            line_count,
            elapsed.as_millis(),
            time_fetching.as_millis(),
            opts.geolocate,
        );
    }


    if !ip_vec.is_empty() && opts.slack {
        dotenv::dotenv()?; 
        if let Ok(webhook_url) = dotenv::var("WEBHOOK"){
            let slack_webhook = SlackWebhook::new(webhook_url);
            let mut text = String::new();
            text += "SUSPICIOUS ACTIVITY\n----------------------------\n";
            text += &output_buff.clone();
            let msg = Message::new(&text);
            // let msg = Message::new("asd");
            slack_webhook.send_message(msg).await?;
        }
       
    }

    println!("{output_buff}");
    
    Ok(())
}
