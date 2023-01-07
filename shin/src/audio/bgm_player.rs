use kira::track::{TrackBuilder, TrackHandle, TrackId, TrackRoutes};
use kira::tween::Tween;
use kira::StartTime;
use shin_core::vm::command::time::Ticks;
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

    pub fn play(&mut self, bgm: Arc<Audio>, volume: f32) {
        let kira_data = bgm.to_kira_data(AudioParams {
            track: self.bgm_track.id(),
            volume,
        });

        let handle = self.audio_manager.play(kira_data);

        assert!(self.current_bgm.is_none());

        self.current_bgm = Some(handle);
    }

    pub fn stop(&mut self, fade_out_time: Ticks) {
        if let Some(mut handle) = self.current_bgm.take() {
            handle
                .stop(Tween {
                    start_time: StartTime::Immediate,
                    duration: fade_out_time.as_duration(),
                    easing: Default::default(),
                })
                .unwrap();
        } else {
            warn!("Tried to stop bgm, but there was no bgm playing");
        }
    }
}

// TODO: make it renderable and updatable, as it can display they track name when the BGM starts
