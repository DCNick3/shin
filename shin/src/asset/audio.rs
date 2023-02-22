use crate::asset::Asset;
use anyhow::Context;
use anyhow::Result;
use shin_core::format::audio::{read_audio, AudioFile};

impl Asset for AudioFile {
    fn load_from_bytes(data: Vec<u8>) -> Result<Self> {
        read_audio(&data).context("Parsing audio file")
    }
}
