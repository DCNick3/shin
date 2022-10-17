use anyhow::{Context, Result};
use clap::Parser;
use shin::format::rom::IndexEntry;
use std::fs::File;
use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    action: SduAction,
}

#[derive(clap::Subcommand, Debug)]
enum SduAction {
    #[clap(subcommand)]
    Rom(RomCommand),
    #[clap(subcommand)]
    Scenario(ScenarioCommand),
}

#[derive(clap::Subcommand, Debug)]
enum RomCommand {
    List {
        rom_path: PathBuf,
    },
    ExtractOne {
        // TODO: make a more generalized interface, maybe like tar or 7z
        rom_path: PathBuf,
        rom_filename: String,
        output_path: PathBuf,
    },
}

#[derive(clap::Subcommand, Debug)]
enum ScenarioCommand {
    Dump { scenario_path: PathBuf },
}

fn rom_command(command: RomCommand) -> Result<()> {
    match command {
        RomCommand::List { rom_path: path } => {
            let rom = File::open(path).context("Opening rom file")?;
            let reader = shin::format::rom::RomReader::new(rom).context("Parsing ROM")?;
            for (name, entry) in reader.traverse() {
                let ty = match entry {
                    IndexEntry::File(_) => "FILE",
                    IndexEntry::Directory(_) => "DIR ",
                };
                println!("{} {}", ty, name);
            }
            Ok(())
        }
        RomCommand::ExtractOne {
            rom_path,
            rom_filename,
            output_path,
        } => {
            use std::io::Read;
            let rom = File::open(rom_path).context("Opening rom file")?;
            let mut reader = shin::format::rom::RomReader::new(rom).context("Parsing ROM")?;
            let file = reader
                .find_file(&rom_filename)
                .context("Searching for file in ROM")?;
            let mut file = reader.open_file(file).context("Opening file in rom")?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            std::fs::write(output_path, buf).context("Writing file")?;
            Ok(())
        }
    }
}

fn scenario_command(command: ScenarioCommand) -> Result<()> {
    match command {
        ScenarioCommand::Dump {
            scenario_path: path,
        } => {
            let scenario = std::fs::read(path)?;
            let reader = shin::format::scenario::ScenarioReader::new(scenario)?;
            // println!("{:#?}", reader);
            Ok(())
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.action {
        SduAction::Rom(cmd) => rom_command(cmd),
        SduAction::Scenario(cmd) => scenario_command(cmd),
    }
}
