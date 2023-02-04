//! Support for decrypting and decoding save files.

use anyhow::Result;
use once_cell::sync::Lazy;

mod crc32;
mod obfuscation;

pub struct Savedata(());

static GAME_KEY: Lazy<u32> = Lazy::new(|| crc32::crc32("うみねこのなく頃に咲".as_bytes(), 0));

impl Savedata {
    pub fn obfuscation_key_from_seed(seed: &str) -> u32 {
        crc32::crc32(seed.as_bytes(), 0)
    }

    /// Same as [Savedata::deobfuscate_with_key], but with fixed game key.
    pub fn deobfuscate(data: &[u8]) -> Result<Vec<u8>> {
        Self::deobfuscate_with_key(data, *GAME_KEY)
    }

    /// Decrypts the game data, returning the raw decrypted bytes.
    /// Can fail if the CRC check fails.
    pub fn deobfuscate_with_key(data: &[u8], key: u32) -> Result<Vec<u8>> {
        obfuscation::decode(data, key)
    }
}
