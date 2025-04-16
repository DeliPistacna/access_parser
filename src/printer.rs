use core::str;

use ansi_term::Colour;
use chrono::{DateTime, Local};

use crate::{ip_info::IpInfo, ip_location::IpLocation};

pub struct Printer {
    colors: bool,
}
impl Printer {
    pub fn new(colors: bool) -> Self {
        Self { colors }
    }

    fn opt_color(&self, text: &str, target_color: &Colour, bold: bool) -> String {
        if self.colors {
            if bold {
                return target_color.bold().paint(text).to_string();
            } else {
                return target_color.paint(text).to_string();
            }
        }
        text.to_string()
    }

    pub fn footer(
        &self,
        line_count: usize,
        elapsed: u128,
        time_fetching: u128,
        geolocate: bool,
    ) -> String {
        let pb = Colour::Purple;
        let mut buff = String::new();

        buff += &format!(
            "\n\nThank you for using {}\n",
            self.opt_color("Delaja's Access Log Parser", &pb, true),
        );

        buff += &format!(
            "Made with {} by {} [{}]\n",
            self.opt_color("<3", &pb, true),
            self.opt_color("Delaja Fedorco", &pb, true),
            self.opt_color("https://delaja.sk", &pb, false),
        );

        buff += &format!(
            "\nProcessed {} lines in {}\n",
            self.opt_color(&line_count.to_string(), &pb, true),
            self.opt_color(&format!("{}{}", elapsed, "ms"), &pb, true)
        );
        if geolocate {
            buff += &format!(
                "Ip location fetched in {} (freeipapi.com)\n",
                self.opt_color(&format!("{}{}", time_fetching, "ms"), &pb, true)
            );
        }
        buff += "\n";
        buff
    }

    pub fn ip(&self, ln: usize, ip: &str, ip_info: &IpInfo, latest_timestamp: DateTime<Local>) -> String {
        let color = Colour::Cyan;
        let last_access = match ip_info.last_timestamp() {
            Some(timestamp) => timestamp.to_string(),
            None => "Unknown".to_string(),
        };
        format!(
            "[{}] {}: ({} requests, average RPM: {}, RPM in last hour: {}, last access: {})\n",
            ln,
            self.opt_color(ip, &color, true),
            self.opt_color(&ip_info.count.to_string(), &color, true),
            self.opt_color(&ip_info.average_rpm().round().to_string(), &color, true),
            self.opt_color(
                &ip_info.average_rpm_last_hour(latest_timestamp).round().to_string(),
                &color,
                true
            ),
            self.opt_color(&last_access, &color, true),
        )
    }

    pub fn list(&self, vec: Vec<(&String, &usize)>, title: &str, limit: usize) -> String{
        let color = Colour::Blue;
        let limited_vec = vec.iter().take(limit);
        let mut buff = String::new();

        buff += &format!(
            "\t{} Most common {}s: ({} unique)\n",
            limited_vec.len(),
            self.opt_color(title, &color, true),
            self.opt_color(&vec.len().to_string(), &color, true)
        );
        for (string, count) in limited_vec {
            buff += &format!(
                "\t\t{} ({})",
                self.opt_color(string, &color, false),
                self.opt_color(&format!("{}{}", &count.to_string(), "x"), &color, true),
            );
            buff += "\n";
        }
        buff += "\n";
        buff
    }

    pub fn location(&self, location: IpLocation) -> String{
        let color = Colour::Red;
        format!("\t{}", self.opt_color(&location.to_string(), &color, true))
    }
}
