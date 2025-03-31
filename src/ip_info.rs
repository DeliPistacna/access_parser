use chrono::{DateTime, Duration, Local};
use std::collections::HashMap;

use crate::{ip_location::IpLocation, log_entry::LogEntry};

#[derive(Debug, Clone)]
pub struct IpInfo {
    pub count: usize,
    pub ua_map: HashMap<String, usize>,
    pub url_map: HashMap<String, usize>,
    pub referrer_map: HashMap<String, usize>,
    pub timestamps: Vec<DateTime<Local>>,
    pub location_data: Option<IpLocation>,
}
impl IpInfo {
    pub fn new() -> Self {
        Self {
            count: 0,
            ua_map: HashMap::new(),
            url_map: HashMap::new(),
            referrer_map: HashMap::new(),
            timestamps: Vec::new(),
            location_data: None,
        }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub fn collect_entry(&mut self, info: LogEntry) {
        //self.count += 1;
        if let Some(ua) = info.ua {
            let map = self.ua_map.entry(ua).or_insert(0);
            *map += 1;
        }
        if let Some(url) = info.url {
            let map = self.url_map.entry(url).or_insert(0);
            *map += 1;
        }

        if let Some(referrer) = info.referrer {
            let map = self.referrer_map.entry(referrer).or_insert(0);
            *map += 1;
        }
        if let Some(timestamp) = info.timestamp {
            self.timestamps.push(timestamp);
        }
    }

    pub fn average_rpm(&self) -> f64 {
        // Ensure there are access times to work with
        if self.timestamps.is_empty() {
            return 0.0; // No requests, so no RPM
        }

        // Find the min and max timestamps from the timestamps
        let min_time = self.timestamps.iter().min().unwrap(); // Earliest time
        let max_time = self.timestamps.iter().max().unwrap(); // Latest time

        // Calculate the duration between the min and max times (in minutes)
        let duration = max_time.signed_duration_since(*min_time); // Duration between the first and last access time
        let duration_in_minutes = duration.num_minutes(); // Convert to minutes

        // Calculate the RPM (requests per minute) over the full period
        if duration_in_minutes == 0 {
            return self.timestamps.len() as f64; // If duration is less than 1 minute, treat as 1 minute interval
        }

        // Calculate the average RPM by dividing the total count by the number of minutes in the range
        self.timestamps.len() as f64 / duration_in_minutes as f64
    }

    // Calculate average RPM over the last hour based on the most recent timestamp in timestamps
    pub fn average_rpm_last_hour(&self) -> f64 {
        // Ensure there are access times to work with
        if self.timestamps.is_empty() {
            return 0.0; // No requests, so no RPM
        }

        // Get the most recent timestamp (last access time)
        let last_timestamp = self.timestamps.iter().max().unwrap(); // Most recent timestamp

        // Calculate the time range for the last hour (relative to the last timestamp)
        let one_hour_ago = *last_timestamp - Duration::hours(1);

        // Filter access times to include only those in the last hour
        let recent_timestamps: Vec<_> = self
            .timestamps
            .iter()
            .filter(|&&time| time >= one_hour_ago)
            .collect();

        // If no requests were in the last hour, return 0.0 RPM
        if recent_timestamps.is_empty() {
            return 0.0;
        }

        // Calculate the RPM for the last hour
        let duration_in_minutes = 60; // We always consider a 1-hour period (60 minutes)

        // Return the RPM: number of requests in the last hour divided by 60 minutes
        recent_timestamps.len() as f64 / duration_in_minutes as f64
    }

    pub fn last_timestamp(&self) -> Option<DateTime<Local>> {
        self.timestamps.iter().max().cloned()
    }
}
