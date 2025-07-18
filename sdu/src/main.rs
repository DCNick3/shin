//! A CLI tool to work with file formats of shin engine games

mod assembler;
mod audio;
mod bustup;
mod rom;
mod savedata;
mod scenario;

use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use anyhow::{Context, Result};
use assembler::{AssemblerCommand, assembler_command};
use bustup::BustupCommand;
use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use image::{GenericImageView, Rgba, RgbaImage};
use itertools::Itertools;
use rom::{RomCommand, rom_command};
use savedata::{SavedataCommand, savedata_command};
use scenario::{ScenarioCommand, scenario_command};
use shin_core::format::{audio::AudioSource, picture::SimpleMergedPicture};
use tracing_subscriber::EnvFilter;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// A tool for working with file formats of shin engine games
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
    /// Operations on MSK mask files
    #[clap(subcommand)]
    Mask(MaskCommand),
    /// Operations on FNT font files
    #[clap(subcommand)]
    Font(FontCommand),
    /// Operations on BUP character bustup files
    #[clap(subcommand)]
    Bustup(BustupCommand),
    /// Operations on TXA texture archive files
    #[clap(subcommand, alias("txa"))]
    TextureArchive(TextureArchiveCommand),
    /// Operations on NXA audio files
    #[clap(subcommand, alias("nxa"))]
    Audio(AudioCommand),
    /// Operations on shin save files
    #[clap(subcommand, alias("save"))]
    Savedata(SavedataCommand),
    /// Operations on the WIP assembler
    #[clap(subcommand, alias("asm"))]
    Assembler(AssemblerCommand),
    /// Operations on SYSSE.bin files
    #[clap(subcommand)]
    Sysse(SysseCommand),
}

#[derive(clap::Args, Debug)]
struct GenerateCommand {
    /// The shell to generate the completion for
    #[clap(value_enum)]
    shell: Shell,
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
enum MaskCommand {
    /// Convert a MSK file into a PNG file
    Decode {
        /// Path to the MSK file
        mask_path: PathBuf,
        /// Path to the output directory
        ///
        /// The directory will contain a png file, as well as txt file with vertices
        // TODO: is it actually useful to output vertices besides format RE?
        // they can be generated from the mask texture itself
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
enum TextureArchiveCommand {
    /// Convert a TXA file into a bunch of PNG files (one per texture)
    Decode {
        /// Path to the TXA file
        texture_archive_path: PathBuf,
        /// Path to the output directory
        output_path: PathBuf,
    },
}

#[derive(clap::Subcommand, Debug)]
enum AudioCommand {
    /// Convert a NXA file into a WAV file
    Decode {
        /// Path to the NXA file
        audio_path: PathBuf,
        /// Path to the output WAV file
        output_path: PathBuf,
    },
    /// Convert an NXA file into an OPUS file losslessly (it simply remuxes the opus packets)
    Remux {
        /// Path to the NXA file
        audio_path: PathBuf,
        /// Path to the output OPUS file
        output_path: PathBuf,
    },
}

#[derive(clap::Subcommand, Debug)]
enum SysseCommand {
    /// Convert a SYSSE.bin file into a bunch of WAV files (one per system sound effect)
    Decode {
        /// Path to the SYSSE.bin file
        sysse_path: PathBuf,
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

fn mask_command(command: MaskCommand) -> Result<()> {
    match command {
        MaskCommand::Decode {
            mask_path,
            output_path,
        } => {
            use std::io::Write;

            let mask = std::fs::read(mask_path)?;
            let mask = shin_core::format::mask::read_mask(&mask)?;

            std::fs::create_dir_all(&output_path)?;

            mask.texels
                .save(output_path.join("mask.png"))
                .context("Writing mask.png")?;

            let v = &mask.regions;
            let mut v_out =
                File::create(output_path.join("vertices.txt")).context("Creating vertices.txt")?;
            writeln!(
                v_out,
                "({}, {}) ({}, {}) ({}, {})",
                v.black_regions.rect_count,
                v.black_regions.region_area,
                v.white_regions.rect_count,
                v.white_regions.region_area,
                v.transparent_regions.rect_count,
                v.transparent_regions.region_area
            )
            .context("Writing vertices.txt")?;

            for (i, vertex) in (0..).zip(&v.rects) {
                if i == v.black_regions.rect_count
                    || i == v.black_regions.rect_count + v.white_regions.rect_count
                {
                    writeln!(v_out).context("Writing vertices.txt")?;
                }

                writeln!(
                    v_out,
                    "{} {} {} {}",
                    vertex.from_x, vertex.from_y, vertex.to_x, vertex.to_y
                )
                .context("Writing vertices.txt")?;
            }

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
            use std::fmt::Write;

            use shin_core::format::font::{GlyphMipLevel, GlyphTrait, read_lazy_font};

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

fn sysse_command(command: SysseCommand) -> Result<()> {
    match command {
        SysseCommand::Decode {
            sysse_path,
            output_path,
        } => {
            use hound::WavSpec;

            let sysse = std::fs::read(sysse_path).context("Reading input file")?;
            let sysse = shin_core::format::sysse::read_sys_se(&sysse).context("Reading SYSSE")?;

            std::fs::create_dir_all(&output_path)?;

            for (name, sound) in sysse.sounds {
                let sample_rate = sound.sample_rate();

                let writer = File::create(output_path.join(format!("{}.wav", name)))
                    .context("Creating output file")?;
                let writer = BufWriter::new(writer);
                let mut writer = hound::WavWriter::new(
                    writer,
                    WavSpec {
                        channels: 2, // TODO: maybe it would be nice to expose mono/stereo in the shin-core API
                        sample_rate,
                        bits_per_sample: 32,
                        sample_format: hound::SampleFormat::Float,
                    },
                )
                .context("Creating WAV writer")?;

                let mut audio_source = AudioSource::new(sound.decode());

                while let Some((left, right)) = audio_source.read_sample() {
                    writer.write_sample(left).context("Writing sample")?;
                    writer.write_sample(right).context("Writing sample")?;
                }

                writer.finalize().context("Finalizing the WAV file")?;
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
    shin_core::create_task_pools();
    let args = Args::parse();
    match args.action {
        SduAction::GenerateCompletion(command) => generate_command(command),
        SduAction::Rom(cmd) => rom_command(cmd),
        SduAction::Scenario(cmd) => scenario_command(cmd),
        SduAction::Picture(cmd) => picture_command(cmd),
        SduAction::Mask(cmd) => mask_command(cmd),
        SduAction::Font(cmd) => font_command(cmd),
        SduAction::Bustup(cmd) => bustup::bustup_command(cmd),
        SduAction::TextureArchive(cmd) => texture_archive_command(cmd),
        SduAction::Audio(cmd) => audio::audio_command(cmd),
        SduAction::Savedata(cmd) => savedata_command(cmd),
        SduAction::Assembler(cmd) => assembler_command(cmd),
        SduAction::Sysse(cmd) => sysse_command(cmd),
    }
}
