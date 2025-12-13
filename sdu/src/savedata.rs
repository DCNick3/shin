use std::{fs::File, path::PathBuf};

use anyhow::{Context, Result};
use shin_core::format::save::Savedata;

#[derive(clap::Subcommand, Debug)]
pub enum SavedataCommand {
    /// Deobfuscates a save file
    Deobfuscate {
        /// Path to the obfuscated save file
        save_path: PathBuf,
        /// Path to the output deobfuscated file
        output_path: PathBuf,
        /// Key to use for deobfuscation (defaults to a game-specific key).
        /// Takes precedence over `--key-seed`.
        #[clap(long)]
        key: Option<u32>,
        /// Key seed to use for deobfuscation (defaults to a game-specific key)
        /// It is run through a hash function to produce the actual key
        #[clap(long)]
        key_seed: Option<String>,
    },
    /// Obfuscate a save file
    Obfuscate {
        /// Path to a deobfuscated save file to obfuscate
        save_path: PathBuf,
        /// Path to the output obfuscated file
        output_path: PathBuf,
        /// Key to use for obfuscation (defaults to a game-specific key).
        /// Takes precedence over `--key-seed`.
        #[clap(long)]
        key: Option<u32>,
        /// Key seed to use for obfuscation (defaults to a game-specific key)
        /// It is run through a hash function to produce the actual key
        #[clap(long)]
        key_seed: Option<String>,
    },
    /// Decode a save file into a human-readable format
    ///
    /// NOTE: there are no guarantees about the stability of the output format, besides it being readable by the same version of `sdu`.
    /// The format might change as more reverse-engineering progress is made.
    Decode {
        /// Path to an obfuscated save file to decode
        save_path: PathBuf,
        /// Path to the output file. Outputs to stdout if not provided
        output_path: Option<PathBuf>,
    },
}

pub fn savedata_command(command: SavedataCommand) -> Result<()> {
    match command {
        SavedataCommand::Deobfuscate {
            save_path,
            output_path,
            key,
            key_seed,
        } => {
            let savedata = std::fs::read(save_path)?;

            let key = key.or_else(|| key_seed.as_deref().map(Savedata::obfuscation_key_from_seed));

            let savedata = match key {
                None => Savedata::deobfuscate(&savedata),
                Some(key) => Savedata::deobfuscate_with_key(&savedata, key),
            }?;

            std::fs::write(output_path, savedata)?;

            Ok(())
        }
        SavedataCommand::Obfuscate {
            save_path,
            output_path,
            key,
            key_seed,
        } => {
            let savedata = std::fs::read(save_path)?;

            let key = key.or_else(|| key_seed.as_deref().map(Savedata::obfuscation_key_from_seed));

            let savedata = match key {
                None => Savedata::obfuscate(&savedata),
                Some(key) => Savedata::obfuscate_with_key(&savedata, key),
            };

            std::fs::write(output_path, savedata)?;

            Ok(())
        }

        SavedataCommand::Decode {
            save_path,
            output_path,
        } => {
            let savedata = std::fs::read(save_path)?;
            let savedata = Savedata::decode(&savedata)?;

            let pretty_config = ron::ser::PrettyConfig::default().compact_arrays(true);

            if let Some(output_path) = output_path {
                let output = File::create(output_path).context("Creating output file")?;

                ron::ser::to_writer_pretty(output, &savedata, pretty_config)
                    .context("Writing human-readable savedata")?;
            } else {
                let output = std::io::stdout().lock();

                ron::ser::to_writer_pretty(output, &savedata, pretty_config)
                    .context("Writing human-readable savedata")?;
            }

            Ok(())
        }
    }
}
