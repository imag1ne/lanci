use std::path::PathBuf;

use clap::Parser;
use url::Url;

#[derive(Debug, Parser)]
#[clap(author, version, about = "LeetCode → Anki .apkg")]
pub struct Cli {
    /// Problem URL (e.g., https://leetcode.com/problems/two-sum/)
    #[arg(short, long)]
    pub url: Url,

    /// Config file path
    #[arg(short, long, default_value = "config.json", value_name = "CONFIG_FILE")]
    pub config: PathBuf,

    /// Output directory
    #[arg(short, long, default_value = "output")]
    pub output_dir: PathBuf,
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
