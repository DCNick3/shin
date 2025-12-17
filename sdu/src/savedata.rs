use std::{fs::File, path::PathBuf};

use anyhow::{Context, Result};
use shin_core::format::save::SaveData;

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
    /// The format might change as more reverse-engineering progress is made. Currently, it's a RON-serialized object (https://github.com/ron-rs/ron).
    Decode {
        /// Path to an obfuscated save file to decode
        save_path: PathBuf,
        /// Path to the output file. Outputs to stdout if not provided
        output_path: Option<PathBuf>,
    },
    /// Encodes a save file from a human-readable format to the game format.
    ///
    /// The human-readable formatted-file can be created with `sdu save decode`.
    Encode {
        /// Path to the input text file.
        text_save_path: PathBuf,
        /// Path to the output encoded save file.
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

            let key = key.or_else(|| key_seed.as_deref().map(SaveData::obfuscation_key_from_seed));

            let savedata = match key {
                None => SaveData::deobfuscate(&savedata),
                Some(key) => SaveData::deobfuscate_with_key(&savedata, key),
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

            let key = key.or_else(|| key_seed.as_deref().map(SaveData::obfuscation_key_from_seed));

            let savedata = match key {
                None => SaveData::obfuscate(&savedata),
                Some(key) => SaveData::obfuscate_with_key(&savedata, key),
            };

            std::fs::write(output_path, savedata)?;

            Ok(())
        }

        SavedataCommand::Decode {
            save_path,
            output_path,
        } => {
            let savedata_bytes = std::fs::read(&save_path).context("Reading the savefile")?;
            let savedata = SaveData::decode(&savedata_bytes).context("Decoding the savefile")?;

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
        SavedataCommand::Encode {
            text_save_path,
            output_path,
        } => {
            let ron_bytes = std::fs::read(&text_save_path).context("Reading the text savefile")?;
            let savedata = ron::de::from_bytes::<SaveData>(&ron_bytes)
                .context("Deserializing the text savefile")?;

            let savedata_bytes = savedata.encode();

            std::fs::write(&output_path, &savedata_bytes).context("Writing the savefile")?;

            Ok(())
        }
    }
}
