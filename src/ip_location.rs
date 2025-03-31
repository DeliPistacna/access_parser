use serde::Deserialize;
use std::hash::{Hash, Hasher};

#[derive(Deserialize, Debug, Clone)]
pub struct IpLocation {
    // pub ip: String,

    // #[serde(rename = "ipVersion")]
    // pub ip_version: Option<u8>,
    //
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    //
    // #[serde(rename = "latitude")]
    // pub latitude: Option<f64>,
    //
    // #[serde(rename = "longitude")]
    // pub longitude: Option<f64>,
    #[serde(rename = "countryName")]
    pub country_name: Option<String>,

    // #[serde(rename = "countryCode")]
    // pub country_code: Option<String>,
    //
    // #[serde(rename = "timeZone")]
    // pub time_zone: Option<String>,
    //
    // #[serde(rename = "zipCode")]
    // pub zip_code: Option<String>,
    #[serde(rename = "cityName")]
    pub city_name: Option<String>,

    #[serde(rename = "regionName")]
    pub region_name: Option<String>,
    // #[serde(rename = "isProxy")]
    // pub is_proxy: Option<bool>,
    //
    // #[serde(rename = "continent")]
    // pub continent: Option<String>,
    //
    // #[serde(rename = "continentCode")]
    // pub continent_code: Option<String>,

    // #[serde(rename = "currency")]
    // pub currency: Option<Currency>,

    // #[serde(rename = "language")]
    // pub language: Option<String>,
    //
    // #[serde(rename = "timeZones")]
    // pub time_zones: Option<Vec<String>>,
    //
    // #[serde(rename = "tlds")]
    // pub tlds: Option<Vec<String>>,
}

// #[derive(Deserialize, Debug)]
// pub struct Currency {
//     #[serde(rename = "code")]
//     pub code: Option<String>,
//
//     #[serde(rename = "name")]
//     pub name: Option<String>,
// }
// Implement `Hash` for `IpLocation`
impl Hash for IpLocation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ip_address.hash(state); // We will hash the `ip_address` field as the unique identifier
    }
}

// Implement `Eq` for `IpLocation`
impl PartialEq for IpLocation {
    fn eq(&self, other: &Self) -> bool {
        self.ip_address == other.ip_address // IpLocation equality is based on the `ip` field
    }
}

impl Eq for IpLocation {}
