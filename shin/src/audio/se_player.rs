use super::manager::AudioManager;
use kira::track::TrackHandle;
use std::sync::Arc;

pub const SE_SLOT_COUNT: usize = 32;

pub struct SePlayer {
    all_se_track: TrackHandle,
    se_tracks: [TrackHandle; SE_SLOT_COUNT],
}

impl SePlayer {
    pub fn new(all_se_track: TrackHandle, se_tracks: [TrackHandle; SE_SLOT_COUNT]) -> Self {
        Self {
            all_se_track,
            se_tracks,
        }
    }
}
