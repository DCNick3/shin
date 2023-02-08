mod kira_data;
mod resampler;

use crate::asset::Asset;
use anyhow::Context;
use anyhow::Result;
use kira::track::TrackId;
use shin_core::format::audio::{read_audio, AudioFile};
use shin_core::time::Tween;
use shin_core::vm::command::types::{Pan, Volume};

pub use kira_data::{AudioData, AudioHandle};

pub struct Audio(AudioFile);

pub struct AudioParams {
    pub track: TrackId,
    pub fade_in: Tween,
    pub repeat: bool,
    pub volume: Volume,
    pub pan: Pan,
    // TODO: support play speed (needs research)
}

impl Asset for Audio {
    fn load_from_bytes(data: Vec<u8>) -> Result<Self> {
        read_audio(&data).map(Self).context("Parsing audio file")
    }
}
