use clap::{Parser, Subcommand};

mod buffer_parser;
mod debug_tex_parser;

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
}

fn main() {
    let opts = Opts::parse();
    match opts.action {
        JunkAction::BufferParser => buffer_parser::main(),
        JunkAction::DebugTexParser => debug_tex_parser::main(),
    }
}
