use anyhow::{Context, Result};
use bytes::Bytes;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use image::{GenericImageView, Rgba, RgbaImage};
use itertools::Itertools;
use shin_core::format::picture::SimpleMergedPicture;
use shin_core::format::rom::{IndexEntry, IndexFile};
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
    /// Generate shell complete script for the given shell
    GenerateCompletion(GenerateCommand),
    /// Operations on ROM archive files
    #[clap(subcommand)]
    Rom(RomCommand),
    /// Operations on SNR scenario files
    #[clap(subcommand)]
    Scenario(ScenarioCommand),
    /// Operations on PIC picture files
    #[clap(subcommand)]
    Picture(PictureCommand),
    /// Operations on FNT font files
    #[clap(subcommand)]
    Font(FontCommand),
    /// Operations on BUP character bustup files
    #[clap(subcommand)]
    Bustup(BustupCommand),
    /// Operations on TXA texture archive files
    #[clap(subcommand, alias("txa"))]
    TextureArchive(TextureArchiveCommand),
}

#[derive(clap::Args, Debug)]
struct GenerateCommand {
    /// If provided, outputs the completion file for given shell
    #[clap(value_enum)]
    shell: Shell,
}

#[derive(clap::Subcommand, Debug)]
enum RomCommand {
    /// List file and directory entries in the archive
    // TODO: print file sizes
    List { rom_path: PathBuf },
    /// Extract one file from the archive (arguments subject to change)
    ExtractOne {
        // TODO: this is awkward to use, make it more ergonomic
        /// Path to the ROM file
        rom_path: PathBuf,
        /// Name of the file in the archive to extract
        rom_filename: String,
        /// Path to the output file
        output_path: PathBuf,
    },
    /// Extract all files from the archive, creating a directory tree
    ExtractTree {
        /// Path to the ROM file
        rom_path: PathBuf,
        /// Path to the output directory (will be created)
        output_dir: PathBuf,
    }
}

#[derive(clap::Subcommand, Debug)]
enum ScenarioCommand {
    /// Run a scenario in VM, printing all the commands executed
    Dump {
        /// Path to the SNR file
        scenario_path: PathBuf,
        /// Initial value of the memory cell "0", usually selecting the episode or smth
        #[clap(default_value = "0")]
        init_val: i32,
        output_filename: Option<PathBuf>,
    },
    /// Run a scenario in VM, parsing all the messages with layout parser (for testing)
    TestLayouter {
        scenario_path: PathBuf,
        /// Initial value of the memory cell "0", usually selecting the episode or smth
        #[clap(default_value = "0")]
        init_val: i32,
    },
    CharFrequency {
        scenario_path: PathBuf,
        /// Initial value of the memory cell "0", usually selecting the episode or smth
        #[clap(default_value = "0")]
        init_val: i32,
        #[clap(default_value = "64")]
        top_k: usize,
    },
    /// [WIP] Decompile a scenario into an assembly-like language
    Decompile { scenario_path: PathBuf },
}

#[derive(clap::Subcommand, Debug)]
enum PictureCommand {
    /// Convert a PIC file into a PNG file
    Decode {
        /// Path to the PIC file
        picture_path: PathBuf,
        /// Path to the output PNG file
        output_path: PathBuf,
    },
}

#[derive(clap::Subcommand, Debug)]
enum FontCommand {
    /// Convert a FNT file into a metadata.txt file and a bunch of PNG files (one per glyph)
    Decode {
        /// Path to the FNT file
        font_path: PathBuf,
        /// Path to the output directory
        output_path: PathBuf,
    },
}

#[derive(clap::Subcommand, Debug)]
enum BustupCommand {
    /// Convert a BUP file into a bunch of PNG files (one base image, one per expression, and one per mouth position)
    Decode {
        /// Path to the BUP file
        bustup_path: PathBuf,
        /// Path to the output directory
        output_path: PathBuf,
    },
}

#[derive(clap::Subcommand, Debug)]
enum TextureArchiveCommand {
    /// Convert a TXA file into a bunch of PNG files (one per texture)
    Decode {
        /// Path to the TXA file
        texture_archive_path: PathBuf,
        /// Path to the output directory
        output_path: PathBuf,
    },
}

fn generate_command(command: GenerateCommand) -> Result<()> {
    let mut cmd = Args::command();
    eprintln!("Generating completion file for {:?}...", command.shell);

    let cmd = &mut cmd;

    generate(
        command.shell,
        cmd,
        cmd.get_name().to_string(),
        &mut std::io::stdout(),
    );
    Ok(())
}

fn rom_command(command: RomCommand) -> Result<()> {
    match command {
        RomCommand::List { rom_path: path } => {
            let rom = File::open(path).context("Opening rom file")?;
            let rom = BufReader::new(rom);
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
            let rom = BufReader::new(rom);
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
        RomCommand::ExtractTree { rom_path, output_dir } => {
            use std::io::Read;
            let rom = File::open(rom_path).context("Opening rom file")?;
            let rom = BufReader::new(rom);
            let mut reader = shin_core::format::rom::RomReader::new(rom).context("Parsing ROM")?;

            // First, make a list of all the files in the rom
            let files: Vec<(String, IndexFile)> = reader
                .traverse()
                .filter_map(|(name, entry)| {
                    match entry {
                        IndexEntry::File(file_entry) => Some((name, *file_entry)),
                        IndexEntry::Directory(_) => None
                    }
                })
                .collect();

            // Then go through the files, read each one from the rom, and write it to the filesystem
            for (name, file_entry) in files {
                // Construct output path
                let mut output_path = output_dir.clone();
                output_path.extend(name.split('/'));

                let mut file = reader.open_file(file_entry).context("Opening file in rom")?;
                let mut buf = Vec::new();
                let len = file.read_to_end(&mut buf).context("Reading file data from rom")?;
                match output_path.parent() {
                    Some(parent) => std::fs::create_dir_all(parent).context("Creating directory to write file in")?,
                    None => {},
                }
                std::fs::write(output_path.as_path(), buf).context("Writing file")?;

                println!("Wrote file {} ({} bytes)", output_path.display(), len);
            }
            Ok(())
        },
    }
}

fn scenario_command(command: ScenarioCommand) -> Result<()> {
    match command {
        ScenarioCommand::Dump {
            scenario_path: path,
            init_val,
            output_filename,
        } => {
            let scenario = std::fs::read(path)?;
            let scenario = Bytes::from(scenario);
            let scenario = shin_core::format::scenario::Scenario::new(scenario)?;

            let mut output: Box<dyn std::io::Write> = match output_filename {
                None => Box::new(std::io::stdout().lock()),
                Some(filename) => Box::new(File::create(filename).context("Opening output file")?),
            };

            let mut vm = shin_core::vm::Scripter::new(&scenario, init_val, 42);
            let mut result = CommandResult::None;
            loop {
                // NOTE: usually you would want to do something when the VM has returned "Pending"
                // stuff like running game loop to let the command progress...
                let command = vm.run(result)?;
                writeln!(output, "{:08x} {}", vm.position().0, command)
                    .context("Writing to the output file")?;
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
        ScenarioCommand::CharFrequency {
            scenario_path: path,
            init_val,
            top_k,
        } => {
            let scenario = std::fs::read(path)?;
            let scenario = Bytes::from(scenario);
            let scenario = shin_core::format::scenario::Scenario::new(scenario)?;

            let mut counter = counter::Counter::<_, u64>::new();

            let mut vm = shin_core::vm::Scripter::new(&scenario, init_val, 42);
            let mut result = CommandResult::None;
            loop {
                // NOTE: usually you would want to do something when the VM has returned "Pending"
                // stuff like running game loop to let the command progress...
                let command = vm.run(result)?;

                if let RuntimeCommand::MSGSET(msgset) = &command {
                    let layouter = shin_core::layout::LayouterParser::new(&msgset.text);
                    for command in layouter {
                        match command {
                            shin_core::layout::ParsedCommand::Char(c) => {
                                counter[&c] += 1;
                            }
                            shin_core::layout::ParsedCommand::Furigana(text) => {
                                counter.update(text.chars());
                            }
                            _ => {}
                        }
                    }
                }

                if let Some(new_result) = command.execute_dummy() {
                    result = new_result
                } else {
                    break;
                }
            }

            println!(
                "{:#?}",
                counter
                    .k_most_common_ordered(top_k)
                    .into_iter()
                    .map(|v| v.0)
                    .sorted()
                    .join("")
            );
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

            let ascent = font.get_ascent();
            let descent = font.get_descent();

            // first, write the metadata & character mappings to a text file
            let mut metadata = String::new();
            writeln!(metadata, "ascent: {}", ascent)?;
            writeln!(metadata, "descent: {}", descent)?;
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

                let size = glyph_data.get_info().actual_size();
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

fn bustup_command(command: BustupCommand) -> Result<()> {
    match command {
        BustupCommand::Decode {
            bustup_path,
            output_path,
        } => {
            use std::fmt::Write;

            let bustup = std::fs::read(bustup_path)?;
            let bustup = shin_core::format::bustup::read_bustup(&bustup)?;

            std::fs::create_dir_all(&output_path)?;

            let mut metadata = String::new();
            writeln!(metadata, "expressions:")?;
            for (expression_name, expression) in bustup.expressions.iter().sorted_by_key(|v| v.0) {
                writeln!(metadata, "  \"{}\":", expression_name.replace('\"', "\\\""))?;
                writeln!(
                    metadata,
                    "    face_pos: {:?}",
                    (
                        expression.face_chunk.offset_x,
                        expression.face_chunk.offset_y
                    )
                )?;
                writeln!(metadata, "    mouths:")?;
                for (i, mouth) in expression.mouth_chunks.iter().enumerate() {
                    writeln!(
                        metadata,
                        "      {}: {:?}",
                        i,
                        (mouth.offset_x, mouth.offset_y)
                    )?;
                }
            }
            std::fs::write(output_path.join("metadata.txt"), metadata)?;

            bustup.base_image.save(output_path.join("base.png"))?;

            for (expression_name, expression) in bustup.expressions.iter() {
                expression
                    .face_chunk
                    .data
                    .save(output_path.join(format!("{}_face.png", expression_name)))?;

                for (i, mouth) in expression.mouth_chunks.iter().enumerate() {
                    mouth
                        .data
                        .save(output_path.join(format!("{}_mouth_{}.png", expression_name, i)))?;
                }
            }

            Ok(())
        }
    }
}

fn texture_archive_command(command: TextureArchiveCommand) -> Result<()> {
    match command {
        TextureArchiveCommand::Decode {
            texture_archive_path,
            output_path,
        } => {
            // use std::fmt::Write;

            let texture_archive = std::fs::read(texture_archive_path)?;
            let texture_archive =
                shin_core::format::texture_archive::read_texture_archive(&texture_archive)?;

            std::fs::create_dir_all(&output_path)?;

            // let mut metadata = String::new();
            // TODO: write metadata
            // std::fs::write(output_path.join("metadata.txt"), metadata)?;

            for (texture_name, index) in texture_archive.name_to_index.iter() {
                let texture = &texture_archive.textures[*index];
                texture.save(output_path.join(format!("{}.png", texture_name)))?;
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
        SduAction::GenerateCompletion(command) => generate_command(command),
        SduAction::Rom(cmd) => rom_command(cmd),
        SduAction::Scenario(cmd) => scenario_command(cmd),
        SduAction::Picture(cmd) => picture_command(cmd),
        SduAction::Font(cmd) => font_command(cmd),
        SduAction::Bustup(cmd) => bustup_command(cmd),
        SduAction::TextureArchive(cmd) => texture_archive_command(cmd),
    }
}
