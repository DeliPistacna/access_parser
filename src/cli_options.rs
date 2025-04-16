use clap::Parser;
use clap::{self, ArgAction};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "accessparser")]
#[command(about = "A program to parse access logs and retrieve top IP addresses")]
pub struct CliOptions {
    #[arg(value_name = "FILE_PATH")]
    pub file_path: PathBuf,

    #[arg(short, long, default_value_t = 0)]
    pub max_ips: usize,

    #[arg(short, long, default_value_t = 3)]
    pub top_params: usize,

    #[arg(short = 'q', long)]
    pub filter_requests: Option<usize>,

    #[arg(short = 'p', long)]
    pub filter_rpm: Option<u16>,

    #[arg(short = 'r', long)]
    pub filter_hours: Option<f64>,

    #[arg(short = 'l', long = "ignore-location", default_value_t = false)]
    pub geolocate: bool,

    #[arg(short = 'c', long = "no-colors", default_value_t = true, action= ArgAction::SetFalse)]
    pub colors: bool,

    #[arg(short = 'f', long = "no-footer", default_value_t = true, action= ArgAction::SetFalse)]
    pub footer: bool,
}
