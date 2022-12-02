use anyhow::{Context, Result};
use bytes::Bytes;
use clap::Parser;
use shin_core::format::picture::SimpleMergedPicture;
use shin_core::format::rom::IndexEntry;
use shin_core::vm::command::CommandResult;
use std::fs::File;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

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
    #[clap(subcommand)]
    Picture(PictureCommand),
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
    Dump {
        scenario_path: PathBuf,
        #[clap(short, long, default_value = "0")]
        init_val: i32,
    },
    Decompile {
        scenario_path: PathBuf,
    },
}

#[derive(clap::Subcommand, Debug)]
enum PictureCommand {
    Decode {
        picture_path: PathBuf,
        output_path: PathBuf,
    },
}

fn rom_command(command: RomCommand) -> Result<()> {
    match command {
        RomCommand::List { rom_path: path } => {
            let rom = File::open(path).context("Opening rom file")?;
            let reader = shin_core::format::rom::RomReader::new(rom).context("Parsing ROM")?;
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
            let mut reader = shin_core::format::rom::RomReader::new(rom).context("Parsing ROM")?;
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
            init_val,
        } => {
            let scenario = std::fs::read(path)?;
            let scenario = Bytes::from(scenario);
            let scenario = shin_core::format::scenario::Scenario::new(scenario)?;

            let mut vm = shin_core::vm::AdvVm::new(&scenario, init_val, 42);
            let mut result = CommandResult::None;
            loop {
                // NOTE: usually you would want to do something when the VM has returned "Pending"
                // stuff like running game loop to let the command progress...
                let command = vm.run(result)?;
                println!("{:08x} {}", vm.position().0, command);
                if let Some(new_result) = command.execute_dummy() {
                    result = new_result
                } else {
                    break;
                }
            }

            // println!("{:#?}", reader);
            Ok(())
        }
        ScenarioCommand::Decompile { scenario_path: _ } => {
            todo!("Decompile scenario");
        }
    }
}

fn picture_command(command: PictureCommand) -> Result<()> {
    match command {
        PictureCommand::Decode {
            picture_path: path,
            output_path,
        } => {
            let picture = std::fs::read(path)?;
            let picture =
                shin_core::format::picture::read_picture::<SimpleMergedPicture>(&picture, ())?;
            picture.image.save(output_path)?;
            Ok(())
        }
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        // .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NEW)
        .compact()
        .init();
    let args = Args::parse();
    match args.action {
        SduAction::Rom(cmd) => rom_command(cmd),
        SduAction::Scenario(cmd) => scenario_command(cmd),
        SduAction::Picture(cmd) => picture_command(cmd),
    }
}
