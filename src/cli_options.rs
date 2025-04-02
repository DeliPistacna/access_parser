use clap::Parser;
use clap::{self, ArgAction};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "accessparser")]
#[command(about = "A program to parse access logs and retrieve top IP addresses")]
pub struct CliOptions {
    #[arg(value_name = "FILE_PATH")]
    pub file_path: PathBuf,

    #[arg(short, long, default_value_t = 15)]
    pub max_ips: usize,

    #[arg(short, long, default_value_t = 3)]
    pub top_params: usize,

    #[arg(short = 'h', long)]
    pub filter_hours: Option<u16>,

    #[arg(short = 'l', long = "ignore-location", default_value_t = false)]
    pub ignore_location: bool,

    #[arg(short = 'c', long = "no-colors", default_value_t = true, action= ArgAction::SetFalse)]
    pub colors: bool,

    #[arg(short = 'f', long = "no-footer", default_value_t = true, action= ArgAction::SetFalse)]
    pub footer: bool,
}
