use kira::track::{TrackBuilder, TrackHandle, TrackId, TrackRoutes};
use std::sync::Arc;

use super::manager::AudioManager;
use crate::asset::audio::{Audio, AudioHandle, AudioParams};

pub struct BgmPlayer {
    audio_manager: Arc<AudioManager>,
    bgm_track: TrackHandle,
    // TODO: async track loading?
    current_bgm: Option<AudioHandle>,
}

impl BgmPlayer {
    pub fn new(audio_manager: Arc<AudioManager>) -> Self {
        let mut manager = audio_manager.manager().lock().unwrap();

        let bgm_track = manager
            .add_sub_track(TrackBuilder::new().routes(TrackRoutes::parent(TrackId::Main)))
            .expect("Failed to create bgm track");

        drop(manager);

        Self {
            audio_manager,
            bgm_track,
            current_bgm: None,
        }
    }

    pub fn play(&mut self, bgm: Arc<Audio>) {
        let kira_data = bgm.to_kira_data(AudioParams {
            track: self.bgm_track.id(),
        });

        let handle = self.audio_manager.play(kira_data);

        assert!(self.current_bgm.is_none());

        self.current_bgm = Some(handle);
    }
}

// TODO: make it renderable and updatable, as it can display they track name when the BGM starts
