use crate::ip_location::IpLocation;
//use reqwest;

use std::collections::HashSet;
use std::error::Error;

pub struct FreeIpApi {}
impl FreeIpApi {
    pub async fn fetch_ip_details(ip_address: &str) -> Result<IpLocation, Box<dyn Error>> {
        let url = format!("https://freeipapi.com/api/json/{}", ip_address);
        let response = reqwest::get(&url).await?.text().await?;
        let ip_info: IpLocation = serde_json::from_str(&response)?;
        Ok(ip_info)
    }

    pub async fn get_loc_info(
        ip_addresses: HashSet<String>, // Accept HashSet of IPs
    ) -> Result<HashSet<IpLocation>, Box<dyn std::error::Error>> {
        // Create tasks to fetch location info for each IP in parallel.
        let tasks: Vec<_> = ip_addresses
            .into_iter()
            .map(|ip_clone| {
                tokio::task::spawn(async move {
                    // Fetch the location info
                    if let Ok(info) = FreeIpApi::fetch_ip_details(&ip_clone).await {
                        // Create and return the IpLocation including the IP
                        Some(info)
                    } else {
                        None
                    }
                })
            })
            .collect();

        // Collect the results
        let mut results = HashSet::new();
        for task in tasks {
            if let Some(location) = task.await? {
                // Double unwrap because of Result inside the future
                results.insert(location);
            }
        }

        Ok(results) // Return the set of IpLocation
    }
}
