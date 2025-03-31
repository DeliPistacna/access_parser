use chrono::{DateTime, Local};

#[derive(Debug)]
pub struct LogEntry {
    //pub ip: Option<String>,
    pub timestamp: Option<DateTime<Local>>,
    pub url: Option<String>,
    pub referrer: Option<String>,
    pub ua: Option<String>,
}

impl LogEntry {
    pub fn parse(line: &str) -> Self {
        //let mut ip = None;
        let mut timestamp = None;
        let mut url = None;
        let mut referrer = None;
        let mut ua = None;

        if let Some((_found_ip, rest_of_line)) = line.split_once(" ") {
            //ip = Some(found_ip.to_string());

            if let Some((date, _)) = rest_of_line.split_once("]") {
                if let Some((_, date)) = date.split_once("[") {
                    timestamp = DateTime::parse_from_str(date, "%d/%b/%Y:%H:%M:%S %z")
                        .ok()
                        .map(|dt| dt.with_timezone(&Local));
                }
            }
        }

        let parts: Vec<&str> = line.split('"').collect();
        if parts.len() >= 5 {
            url = Some(parts[1].to_string());
            referrer = Some(parts[3].to_string());
            ua = Some(parts[5].to_string());
        }

        LogEntry {
            //ip,
            timestamp,
            url,
            referrer,
            ua,
        }
    }

    pub fn parse_ip(line: &str) -> Option<&str> {
        let mut ip: Option<&str> = None;
        if let Some((found_ip, _)) = line.split_once(" ") {
            ip = Some(found_ip);
        }
        ip
    }

    pub fn parse_ip_and_timestamp(line: &str) -> (Option<&str>, Option<DateTime<Local>>) {
        let mut ip: Option<&str> = None;
        let mut timestamp: Option<DateTime<Local>> = None;

        if let Some((found_ip, rest_of_line)) = line.split_once(" ") {
            ip = Some(found_ip);

            if let Some((date, _)) = rest_of_line.split_once("]") {
                if let Some((_, date)) = date.split_once("[") {
                    timestamp = DateTime::parse_from_str(date, "%d/%b/%Y:%H:%M:%S %z")
                        .ok()
                        .map(|dt| dt.with_timezone(&Local));
                }
            }
        }

        (ip, timestamp)
    }
}
