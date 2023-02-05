use super::manager::AudioManager;
use crate::asset::audio::{Audio, AudioHandle, AudioParams, AudioWaitStatus};
use kira::track::{TrackBuilder, TrackHandle, TrackId, TrackRoutes};
use shin_core::time::Tween;
use shin_core::vm::command::types::{Pan, Volume};
use std::sync::Arc;
use tracing::warn;

pub const SE_SLOT_COUNT: usize = 32;

pub struct SePlayer {
    audio_manager: Arc<AudioManager>,
    se_tracks: [TrackHandle; SE_SLOT_COUNT],
    se_slots: [Option<AudioHandle>; SE_SLOT_COUNT],
}

impl SePlayer {
    pub fn new(audio_manager: Arc<AudioManager>) -> Self {
        let mut manager = audio_manager.manager().lock().unwrap();

        let se_tracks = [(); SE_SLOT_COUNT].map(|_| {
            manager
                .add_sub_track(TrackBuilder::new().routes(TrackRoutes::parent(TrackId::Main)))
                .expect("Failed to create se track")
        });

        drop(manager);

        Self {
            audio_manager,
            se_tracks,
            se_slots: [(); SE_SLOT_COUNT].map(|_| None),
        }
    }

    pub fn play(
        &mut self,
        slot: i32,
        se: Arc<Audio>,
        repeat: bool,
        volume: Volume,
        pan: Pan,
        fade_in: Tween,
    ) {
        let slot = slot as usize;

        let kira_data = se.to_kira_data(AudioParams {
            track: self.se_tracks[slot].id(),
            fade_in,
            repeat,
            volume,
            pan,
        });

        let handle = self.audio_manager.play(kira_data);

        if let Some(mut old_handle) = self.se_slots[slot].take() {
            old_handle.stop(Tween::MS_15).unwrap();
        }

        self.se_slots[slot] = Some(handle);
    }

    pub fn set_volume(&mut self, slot: i32, volume: Volume, tween: Tween) {
        let slot = slot as usize;

        if let Some(handle) = self.se_slots[slot].as_mut() {
            handle.set_volume(volume, tween).unwrap();
        } else {
            warn!(
                "Tried to set volume of se slot {}, but there was no se playing",
                slot
            );
        }
    }

    pub fn set_panning(&mut self, slot: i32, pan: Pan, tween: Tween) {
        let slot = slot as usize;

        if let Some(handle) = self.se_slots[slot].as_mut() {
            handle.set_panning(pan, tween).unwrap();
        } else {
            warn!(
                "Tried to set pan of se slot {}, but there was no se playing",
                slot
            );
        }
    }

    pub fn stop(&mut self, slot: i32, fade_out: Tween) {
        let slot = slot as usize;

        if let Some(mut se) = self.se_slots[slot].take() {
            se.stop(fade_out).unwrap();
        } else {
            warn!("Tried to stop a SE that was not playing");
        }
    }

    pub fn stop_all(&mut self, fade_out: Tween) {
        for slot in 0..SE_SLOT_COUNT {
            if self.se_slots[slot].is_some() {
                self.stop(slot as i32, fade_out);
            }
        }
    }

    pub fn get_wait_status(&self, slot: i32) -> AudioWaitStatus {
        let slot = slot as usize;

        if let Some(handle) = self.se_slots[slot].as_ref() {
            handle.get_wait_status()
        } else {
            AudioWaitStatus::STOPPED
        }
    }
}
