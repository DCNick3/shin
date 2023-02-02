use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(short, long)]
    pub assets_dir: Option<PathBuf>,
}
