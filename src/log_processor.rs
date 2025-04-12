use crate::{file_reader::FileReader, ip_info::IpInfo, log_entry::LogEntry};
use chrono::{DateTime, Local};
use std::{
    collections::{HashMap, HashSet},
    process::exit,
};

#[derive(PartialEq, Debug)]
pub enum ParseType {
    IpOnly,
    IpAndTimestamp,
    Full,
}

#[derive(Debug)]
pub struct LogProcessor<'a> {
    reader: &'a FileReader,
    filter_hours: Option<u16>,
    most_recent_timestamp: DateTime<Local>,
    pub filter_ips: HashSet<String>,
}

impl<'a> LogProcessor<'a> {
    pub fn new(reader: &'a FileReader, filter_hours: Option<u16>) -> Self {
        Self {
            reader,
            filter_hours,
            most_recent_timestamp: DateTime::default(),
            filter_ips: HashSet::new(),
        }
    }

    pub fn process_log(
        &mut self,
        ip_map: &mut HashMap<String, IpInfo>,
        parse_type: ParseType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let should_filter_ips = !self.filter_ips.is_empty();

        for (ln, line) in self.reader.get_lines()?.enumerate() {
            if ln > 0 && ln % 50000 == 0 {
                println!("Processed {ln} lines");
            }
            let line = line?;
            match parse_type {
                ParseType::IpOnly => {
                    if let Some(ip) = LogEntry::parse_ip(&line) {
                        let entry = ip_map.entry(ip.to_string()).or_insert_with(IpInfo::new);
                        entry.increment();
                    }
                }

                ParseType::IpAndTimestamp => {
                    if let Some((ip, timestamp)) = LogEntry::parse_ip_and_timestamp(&line) {
                        let entry = ip_map.entry(ip.to_string()).or_insert_with(IpInfo::new);
                        entry.increment();
                        if timestamp > self.most_recent_timestamp {
                            self.most_recent_timestamp = timestamp
                        }
                        entry.timestamps.push(timestamp);
                    }
                }

                ParseType::Full => {
                    if let Some(ip) = LogEntry::parse_ip(&line) {
                        if should_filter_ips && !self.filter_ips.contains(ip) {
                            ip_map.remove(ip);
                            continue;
                        }

                        if let Some(entry) = LogEntry::parse(&line) {
                            let ip_info = ip_map.entry(ip.to_string()).or_insert_with(IpInfo::new);
                            ip_info.collect_entry(entry);
                        }
                    }
                }
            };
        }

        Ok(())
    }
}
