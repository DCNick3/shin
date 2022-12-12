use anyhow::{Context, Result};
use bytes::Bytes;
use clap::Parser;
use image::{GenericImageView, Rgba, RgbaImage};
use itertools::Itertools;
use shin_core::format::picture::SimpleMergedPicture;
use shin_core::format::rom::IndexEntry;
use shin_core::vm::command::{CommandResult, RuntimeCommand};
use std::fs::File;
use std::io::BufReader;
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
    #[clap(subcommand)]
    Font(FontCommand),
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
    TestLayouter {
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

#[derive(clap::Subcommand, Debug)]
enum FontCommand {
    Decode {
        font_path: PathBuf,
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

            let mut vm = shin_core::vm::Scripter::new(&scenario, init_val, 42);
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
        ScenarioCommand::TestLayouter {
            scenario_path: path,
            init_val,
        } => {
            let scenario = std::fs::read(path)?;
            let scenario = Bytes::from(scenario);
            let scenario = shin_core::format::scenario::Scenario::new(scenario)?;

            let mut vm = shin_core::vm::Scripter::new(&scenario, init_val, 42);
            let mut result = CommandResult::None;
            loop {
                // NOTE: usually you would want to do something when the VM has returned "Pending"
                // stuff like running game loop to let the command progress...
                let command = vm.run(result)?;

                if let RuntimeCommand::MSGSET(msgset) = &command {
                    let layouter = shin_core::layout::LayouterParser::new(&msgset.text);
                    let commands = layouter.collect::<Vec<_>>();
                    println!("{:?}", commands);
                }

                if let Some(new_result) = command.execute_dummy() {
                    result = new_result
                } else {
                    break;
                }
            }

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

fn font_command(command: FontCommand) -> Result<()> {
    match command {
        FontCommand::Decode {
            font_path: path,
            output_path,
        } => {
            use shin_core::format::font::{read_lazy_font, GlyphMipLevel, GlyphTrait};
            use std::fmt::Write;

            let font = File::open(path)?;
            let mut font = BufReader::new(font);
            let font = read_lazy_font(&mut font)?;
            std::fs::create_dir_all(&output_path)?;

            let (min_size, max_size) = font.get_size_range();

            // first, write the metadata & character mappings to a text file
            let mut metadata = String::new();
            writeln!(metadata, "min_size: {}", min_size)?;
            writeln!(metadata, "max_size: {}", max_size)?;
            writeln!(metadata, "characters:")?;
            for (character, glyph) in font.get_character_mapping().iter().enumerate() {
                writeln!(metadata, "  {:04x}: {:04}", character, glyph.0)?;
            }
            // finally, write the glyph metadata
            writeln!(metadata, "glyphs:")?;
            for (glyph, glyph_data) in font.get_glyphs().iter().sorted_by_key(|v| v.0) {
                let info = glyph_data.get_info();
                writeln!(metadata, "  {:04}", glyph.0)?;
                writeln!(metadata, "    bearing_y: {}", info.bearing_y)?;
                writeln!(metadata, "    bearing_x: {}", info.bearing_x)?;
                writeln!(metadata, "    advance  : {}", info.advance_width)?;
            }
            std::fs::write(output_path.join("metadata.txt"), metadata)?;

            // then, write each glyph to a separate file
            for (&glyph_id, glyph_data) in font.get_glyphs().iter() {
                let glyph_data = glyph_data.decompress();

                let size = glyph_data.get_info().size();
                let glyph_pic = glyph_data
                    .get_image(GlyphMipLevel::Level0)
                    .view(0, 0, size.0, size.1);

                let mut new_glyph_pic = RgbaImage::new(size.0, size.1);

                for (x, y, pixel) in glyph_pic.pixels() {
                    let new_pixel = Rgba([0, 0, 0, pixel[0]]);

                    new_glyph_pic.put_pixel(x, y, new_pixel);
                }

                new_glyph_pic.save(output_path.join(format!("{:04}.png", glyph_id.0)))?;
            }
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
        SduAction::Font(cmd) => font_command(cmd),
    }
}
