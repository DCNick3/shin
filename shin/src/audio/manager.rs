use crate::asset::audio::{AudioData, AudioHandle};
use kira::manager::backend::cpal::CpalBackend;
use kira::manager::AudioManagerSettings;
use std::sync::Mutex;

// TODO: we want some more generic (?) audio manager, as this one is only suited for ADV
pub struct AudioManager {
    manager: Mutex<kira::manager::AudioManager<CpalBackend>>,
}

impl AudioManager {
    pub fn new() -> Self {
        let manager = kira::manager::AudioManager::new(AudioManagerSettings::default())
            .expect("Failed to create kira audio manager");

        // let all_se_track = manager
        //     .add_sub_track(TrackBuilder::new().routes(TrackRoutes::parent(TrackId::Main)))
        //     .expect("Failed to create all_se track");
        //
        // let se_tracks = [(); SE_SLOT_COUNT].map(|_| {
        //     manager
        //         .add_sub_track(TrackBuilder::new().routes(TrackRoutes::parent(all_se_track.id())))
        //         .expect("Failed to create se track")
        // });
        //
        // let se_player = SePlayer::new(all_se_track, se_tracks);

        Self {
            manager: Mutex::new(manager),
        }
    }

    pub(super) fn play(&self, data: AudioData) -> AudioHandle {
        let mut manager = self.manager.lock().unwrap();

        manager.play(data).expect("Failed to start playing audio")
    }

    pub(super) fn manager(&self) -> &Mutex<kira::manager::AudioManager<CpalBackend>> {
        &self.manager
    }
}
