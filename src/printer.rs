use core::str;

use ansi_term::Colour;

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
        igonre_location: bool,
    ) {
        let pb = Colour::Purple;
        println!(
            "\n\nThank you for using {}",
            self.opt_color("Delaja's Access Log Parser", &pb, true),
        );
        println!(
            "Made with {} by {} [{}]",
            self.opt_color("<3", &pb, true),
            self.opt_color("Delaja Fedorco", &pb, true),
            self.opt_color("https://delaja.sk", &pb, false),
        );

        println!(
            "\nProcessed {} lines in {}",
            self.opt_color(&line_count.to_string(), &pb, true),
            self.opt_color(&format!("{}{}", elapsed, "ms"), &pb, true)
        );
        if !igonre_location {
            println!(
                "Ip location fetched in {} (freeipapi.com)",
                self.opt_color(&format!("{}{}", time_fetching, "ms"), &pb, true)
            );
        }
    }

    pub fn ip(&self, ln: usize, ip: &str, ip_info: &IpInfo) {
        let color = Colour::Cyan;
        let last_access = match ip_info.last_timestamp() {
            Some(timestamp) => timestamp.to_string(),
            None => "Unknown".to_string(),
        };
        println!(
            "[{}] {}: ({} requests, average RPM: {}, RPM in last hour: {}, last access: {})",
            ln,
            self.opt_color(ip, &color, true),
            self.opt_color(&ip_info.count.to_string(), &color, true),
            self.opt_color(&ip_info.average_rpm().round().to_string(), &color, true),
            self.opt_color(
                &ip_info.average_rpm_last_hour().round().to_string(),
                &color,
                true
            ),
            self.opt_color(&last_access, &color, true),
        );
    }

    pub fn list(&self, vec: Vec<(String, usize)>, title: &str, limit: usize) {
        let color = Colour::Blue;
        let limited_vec = vec.iter().take(limit);
        println!(
            "\t{} Most common {}s: ({} unique)",
            limited_vec.len(),
            self.opt_color(title, &color, true),
            self.opt_color(&vec.len().to_string(), &color, true)
        );
        for (string, count) in limited_vec {
            println!(
                "\t\t{} ({})",
                self.opt_color(string, &color, false),
                self.opt_color(&format!("{}{}", &count.to_string(), "x"), &color, true),
            )
        }
    }

    pub fn location(&self, location_data: Option<IpLocation>) {
        let color = Colour::Red;
        let unk: String = "Unknown".to_string();
        match location_data {
            Some(location) => {
                println!(
                    "\tLocation: {}, {}, {}",
                    self.opt_color(&location.country_name.unwrap_or(unk.clone()), &color, true),
                    self.opt_color(&location.region_name.unwrap_or(unk.clone()), &color, true),
                    self.opt_color(&location.city_name.unwrap_or(unk.clone()), &color, true),
                )
            }
            None => {
                println!("\tLocation: {}", self.opt_color("Unknown", &color, true))
            }
        }
    }
}
