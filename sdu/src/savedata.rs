use anyhow::{Context, Result};
use shin_core::format::save::Savedata;
use std::fs::File;
use std::path::PathBuf;

#[derive(clap::Subcommand, Debug)]
pub enum SavedataCommand {
    /// Deobfuscates the save file
    Deobfuscate {
        /// Path to the save file
        save_path: PathBuf,
        /// Path to the output decrypted file
        output_path: PathBuf,
        /// Key to use for deobfuscation (defaults to a game-specific key)
        #[clap(long)]
        key: Option<u32>,
        /// Key seed to use for deobfuscation (defaults to a game-specific key)
        /// It is run through a hash function to produce the actual key
        #[clap(long)]
        key_seed: Option<String>,
    },
    /// Obfuscate the save file
    Obfuscate {
        /// Path to the save file
        save_path: PathBuf,
        /// Path to the output encrypted file
        output_path: PathBuf,
        /// Key to use for obfuscation (defaults to a game-specific key)
        #[clap(long)]
        key: Option<u32>,
        /// Key seed to use for obfuscation (defaults to a game-specific key)
        /// It is run through a hash function to produce the actual key
        #[clap(long)]
        key_seed: Option<String>,
    },
    /// Decode the save file into a human-readable format
    Decode {
        /// Path to the save file
        save_path: PathBuf,
        /// Path to the output yaml file
        output_path: PathBuf,
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

            ron::ser::to_writer_pretty(
                File::create(output_path).context("Creating output file")?,
                &savedata,
                ron::ser::PrettyConfig::default().compact_arrays(true),
            )
            .context("Writing human-readable savedata")?;

            Ok(())
        }
    }
}
