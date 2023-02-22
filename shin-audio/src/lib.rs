mod data;
mod handle;
mod resampler;
mod sound;

use kira::track::TrackId;
use shin_core::time::Tween;
use shin_core::vm::command::types::{Pan, Volume};

pub use data::AudioData;
pub use handle::AudioHandle;
pub use shin_core::format::audio::AudioFile;

pub struct AudioSettings {
    pub track: TrackId,
    pub fade_in: Tween,
    pub repeat: bool,
    pub volume: Volume,
    pub pan: Pan,
    // TODO: support play speed (needs research)
}
