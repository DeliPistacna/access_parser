use crate::{file_reader::{FileReader, ReaderDirection}, ip_info::IpInfo, log_entry::LogEntry};
use chrono::{DateTime, Local, TimeDelta};
use std::{ collections::{HashMap, HashSet}, io::Error, path::Path};

#[derive(PartialEq, Debug)]
pub enum ParseType {
    IpOnly,
    IpAndTimestamp,
    Full,
    FullReverse,
}

#[derive(Debug)]
pub struct LogProcessor {
    reader: FileReader,
    filter_hours: Option<f64>,
    most_recent_timestamp: DateTime<Local>,
    break_line: Option<String>,
    pub filter_ips: HashSet<String>,
}

impl LogProcessor {
    pub fn new(path: &Path, filter_hours: Option<f64>) -> Result<Self, Error> {
        let reader = FileReader::new(path.to_path_buf())?;
        Ok(Self {
            reader,
            filter_hours,
            break_line: None,
            most_recent_timestamp: DateTime::default(),
            filter_ips: HashSet::new(),
        })
    }

    pub fn get_latest_timestamp(&self) -> DateTime<Local> {
        self.most_recent_timestamp
    }

    pub fn process_log(
        &mut self,
        ip_map: &mut HashMap<String, IpInfo>,
        parse_type: ParseType,
    ) -> Result<usize , Box<dyn std::error::Error>> {
        let should_filter_ips = !self.filter_ips.is_empty();
        let mut processed_lines:usize = 0;

        let dir = match parse_type {
            ParseType::IpAndTimestamp => ReaderDirection::Reverse,
            ParseType::FullReverse => ReaderDirection::Reverse,
            _ => ReaderDirection::Normal,
        };

        let filter_delta = match self.filter_hours {
            Some(hours) => TimeDelta::new((hours*3600f64).floor() as i64 ,0 ),
            _ => None,
        };

        for line in self.reader.get_lines(dir)? {
            processed_lines += 1;
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

                        // println!("{timestamp:?}");

                        let entry = ip_map.entry(ip.to_string()).or_insert_with(IpInfo::new);
                        entry.increment();
                        if timestamp > self.most_recent_timestamp {
                            self.most_recent_timestamp = timestamp;
                            // println!("Most recent timestamp: {timestamp}");
                        } else if let Some(filter_delta) = filter_delta {
                            // Calculate the time difference in hours
                            let time_difference = self.most_recent_timestamp - timestamp;
                            let hours_difference = time_difference; // Assuming timestamp is in seconds

                            // println!("DIFF: {hours_difference:?} OF {filter_delta:?}");

                            // Break if the timestamp is older than the threshold
                            if hours_difference > filter_delta {
                                // println!("Final TS: {timestamp}");
                                self.break_line = Some(line);
                                return Ok(processed_lines); // Break out of the processing loop
                            }
                        }
                        entry.timestamps.push(timestamp);
                    }
                }

                _ => {

                    if let Some(ip) = LogEntry::parse_ip(&line) {
                        if should_filter_ips && !self.filter_ips.contains(ip) {
                            ip_map.remove(ip);

                            if let Some(break_line) = &self.break_line{
                                if break_line == &line {
                                    // println!("Breakin on line: {processed_lines}");
                                    return  Ok(processed_lines);
                                }
                            }

                            continue;
                        }

                        if let Some(entry) = LogEntry::parse(&line) {
                            let ip_info = ip_map.entry(ip.to_string()).or_insert_with(IpInfo::new);
                            ip_info.collect_entry(entry);
                        }
                    }

                    if let Some(break_line) = &self.break_line{
                        if break_line == &line {
                            // println!("Breakin on line: {processed_lines}");
                            return  Ok(processed_lines);
                        }
                    }

                }

            };
        }

        Ok(processed_lines)
    }
}
