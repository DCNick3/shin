mod kira_data;
mod resampler;

use crate::asset::Asset;
use anyhow::Context;
use anyhow::Result;
use bitflags::bitflags;
use kira::track::TrackId;
use shin_core::format::audio::{read_audio, AudioFile};
use shin_core::time::Tween;

pub use kira_data::{AudioData, AudioHandle};

pub struct Audio(AudioFile);

pub struct AudioParams {
    pub track: TrackId,
    pub fade_in: Tween,
    pub repeat: bool,
    pub volume: f32,
    pub pan: f32,
    // TODO: support play speed (needs research)
}

impl Asset for Audio {
    fn load_from_bytes(data: Vec<u8>) -> Result<Self> {
        read_audio(&data).map(Self).context("Parsing audio file")
    }
}

bitflags! {
    pub struct AudioWaitStatus: u32 {
        const PLAYING = 1;
        const STOPPED = 2;
        const VOLUME_TWEENER_IDLE = 4;
        const PANNING_TWEENER_IDLE = 8;
        const PLAY_SPEED_TWEENER_IDLE = 16;
    }
}
