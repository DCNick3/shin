use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

mod buffer_parser;

// use clap to select what to do
#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    action: JunkAction,
}

#[derive(Parser)]
enum JunkAction {
    BufferParser,
}

fn main() {
    let opts = Opts::parse();
    match opts.action {
        JunkAction::BufferParser => buffer_parser::main(),
    }
}
