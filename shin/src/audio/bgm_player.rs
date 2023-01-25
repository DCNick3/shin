use kira::track::{TrackBuilder, TrackHandle, TrackId, TrackRoutes};
use shin_core::time::Tween;
use std::sync::Arc;
use tracing::warn;

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

    pub fn play(
        &mut self,
        bgm: Arc<Audio>,
        _display_name: &str,
        repeat: bool,
        volume: f32,
        fade_in: Tween,
    ) {
        let kira_data = bgm.to_kira_data(AudioParams {
            track: self.bgm_track.id(),
            fade_in,
            repeat,
            volume,
            pan: 0.0,
        });

        let handle = self.audio_manager.play(kira_data);

        if let Some(mut old_handle) = self.current_bgm.take() {
            old_handle.stop(Tween::MS_15).unwrap();
        }

        self.current_bgm = Some(handle);
    }

    pub fn set_volume(&mut self, volume: f32, tween: Tween) {
        if let Some(handle) = self.current_bgm.as_mut() {
            handle.set_volume(volume, tween).unwrap();
        } else {
            warn!("Tried to set volume of BGM, but no BGM is currently playing");
        }
    }

    pub fn stop(&mut self, fade_out: Tween) {
        if let Some(mut handle) = self.current_bgm.take() {
            handle.stop(fade_out).unwrap();
        } else {
            warn!("Tried to stop BGM, but no BGM is currently playing");
        }
    }
}

// TODO: make it renderable and updatable, as it can display they track name when the BGM starts
