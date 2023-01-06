use super::manager::AudioManager;
use kira::track::TrackHandle;
use std::sync::Arc;

pub struct BgmPlayer {
    bgm_track: TrackHandle,
    current_bgm: Option<()>,
}

impl BgmPlayer {
    pub fn new(bgm_track: TrackHandle) -> Self {
        Self {
            bgm_track,
            current_bgm: None,
        }
    }
}

// TODO: make it renderable and updatable, as it can display they track name when the BGM starts
