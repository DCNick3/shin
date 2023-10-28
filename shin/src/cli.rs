use std::path::PathBuf;

use clap::Parser;
use clap_num::maybe_hex;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// A visual novel engine
pub struct Cli {
    /// Search this directory for assets
    ///
    /// The directory must contain either a directory named "data" or a file named "data.rom".
    /// Consult the README for more information.
    #[clap(short, long)]
    pub assets_dir: Option<PathBuf>,
    /// Automatically fast-forward the scenario to the specified address (useful for debugging)
    #[clap(long, value_parser=maybe_hex::<u32>)]
    pub fast_forward_to: Option<u32>,
}
