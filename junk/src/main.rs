//! Just random one-time use scripts to parse some data.

use clap::Parser;

mod buffer_parser;
mod check_info_uniqueness;
mod debug_tex_parser;
mod dump_bup_headers;
mod mask_visualize_vertices;

// use clap to select what to do
#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    action: JunkAction,
}

#[derive(Parser)]
enum JunkAction {
    BufferParser,
    DebugTexParser,
    CheckInfoUniqueness { snr_path: String },
    MaskVisualizeVertices { msk_path: String },
    DumpBupHeaders { root_path: String },
}

fn main() {
    let opts = Opts::parse();
    match opts.action {
        JunkAction::BufferParser => buffer_parser::main(),
        JunkAction::DebugTexParser => debug_tex_parser::main(),
        JunkAction::CheckInfoUniqueness { snr_path } => check_info_uniqueness::main(snr_path),
        JunkAction::MaskVisualizeVertices { msk_path } => mask_visualize_vertices::main(msk_path),
        JunkAction::DumpBupHeaders { root_path } => dump_bup_headers::main(root_path),
    }
}
