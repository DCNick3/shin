use std::{fs::File, path::PathBuf};

use anyhow::{Context, Result};
use shin_core::{
    format::rom::{IndexEntry, IndexFile},
    primitives::stateless_reader::StatelessFile,
};

#[derive(clap::Subcommand, Debug)]
pub enum RomCommand {
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
    /// Extract multiple files from the archive, creating a directory tree
    Extract {
        /// Path to the ROM file
        rom_path: PathBuf,
        /// Path to the output directory (will be created if it does not exist)
        output_dir: PathBuf,
        /// Names of specific files to be extracted. If none are specified, all files in the ROM will be extracted.
        file_names: Vec<String>,
    },
}

pub fn rom_command(command: RomCommand) -> Result<()> {
    match command {
        RomCommand::List { rom_path: path } => {
            let rom = File::open(path).context("Opening rom file")?;
            let rom = StatelessFile::new(rom).context("Creating stateless file")?;
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
            let rom = StatelessFile::new(rom).context("Creating stateless file")?;
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
        RomCommand::Extract {
            rom_path,
            output_dir,
            file_names,
        } => {
            use std::io::Read;
            let rom = File::open(rom_path).context("Opening rom file")?;
            let rom = StatelessFile::new(rom).context("Creating stateless file")?;
            let mut reader = shin_core::format::rom::RomReader::new(rom).context("Parsing ROM")?;

            // First, make a list of all the files in the rom
            let files: Vec<(String, IndexFile)> = reader
                .traverse()
                .filter_map(|(name, entry)| match entry {
                    IndexEntry::File(file_entry) => {
                        if file_names.is_empty() || file_names.contains(&name) {
                            Some((name, *file_entry))
                        } else {
                            None
                        }
                    }
                    IndexEntry::Directory(_) => None,
                })
                .collect();

            // Then go through the files, read each one from the rom, and write it to the filesystem
            for (name, file_entry) in files {
                // Construct output path
                let mut output_path = output_dir.clone();
                output_path.extend(name.split('/'));

                let mut file = reader
                    .open_file(file_entry)
                    .context("Opening file in rom")?;
                let mut buf = Vec::new();
                let len = file
                    .read_to_end(&mut buf)
                    .context("Reading file data from rom")?;
                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent)
                        .context("Creating directory to write file in")?
                }
                std::fs::write(output_path.as_path(), buf).context("Writing file")?;

                println!("Wrote file {} ({} bytes)", output_path.display(), len);
            }
            Ok(())
        }
    }
}
