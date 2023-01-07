use super::manager::AudioManager;
use crate::asset::audio::{Audio, AudioHandle, AudioParams};
use kira::track::{TrackBuilder, TrackHandle, TrackId, TrackRoutes};
use kira::tween::Tween;
use kira::StartTime;
use shin_core::vm::command::time::Ticks;
use std::sync::Arc;
use tracing::warn;

pub const SE_SLOT_COUNT: usize = 32;

pub struct SePlayer {
    audio_manager: Arc<AudioManager>,
    all_se_track: TrackHandle,
    se_tracks: [TrackHandle; SE_SLOT_COUNT],
    se_slots: [Option<AudioHandle>; SE_SLOT_COUNT],
}

impl SePlayer {
    pub fn new(audio_manager: Arc<AudioManager>) -> Self {
        let mut manager = audio_manager.manager().lock().unwrap();

        let all_se_track = manager
            .add_sub_track(TrackBuilder::new().routes(TrackRoutes::parent(TrackId::Main)))
            .expect("Failed to create all_se track");

        let se_tracks = [(); SE_SLOT_COUNT].map(|_| {
            manager
                .add_sub_track(TrackBuilder::new().routes(TrackRoutes::parent(all_se_track.id())))
                .expect("Failed to create se track")
        });

        drop(manager);

        Self {
            audio_manager,
            all_se_track,
            se_tracks,
            se_slots: [(); SE_SLOT_COUNT].map(|_| None),
        }
    }

    pub fn play(&mut self, slot: i32, se: Arc<Audio>, volume: f32) {
        let slot = slot as usize;

        let kira_data = se.to_kira_data(AudioParams {
            track: self.se_tracks[slot].id(),
            volume,
        });

        let handle = self.audio_manager.play(kira_data);

        assert!(self.se_slots[slot].is_none());

        self.se_slots[slot] = Some(handle);
    }

    pub fn stop(&mut self, slot: i32, fade_out_time: Ticks) {
        let slot = slot as usize;

        if let Some(mut se) = self.se_slots[slot].take() {
            se.stop(Tween {
                start_time: StartTime::Immediate,
                duration: fade_out_time.as_duration(),
                easing: Default::default(),
            })
            .unwrap();
        } else {
            warn!("Tried to stop a SE that was not playing");
        }
    }

    pub fn stop_all(&mut self, fade_out_time: Ticks) {
        for slot in 0..SE_SLOT_COUNT {
            if self.se_slots[slot].is_some() {
                self.stop(slot as i32, fade_out_time);
            }
        }
    }
}
