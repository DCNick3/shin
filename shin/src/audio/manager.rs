use kira::manager::backend::cpal::CpalBackend;
use kira::manager::AudioManagerSettings;
use kira::track::{TrackBuilder, TrackId, TrackRoutes};
use std::sync::Mutex;

use crate::audio::bgm_player::BgmPlayer;
use crate::audio::se_player::{SePlayer, SE_SLOT_COUNT};

pub struct AudioManager {
    manager: Mutex<kira::manager::AudioManager<CpalBackend>>,
    bgm_player: BgmPlayer,
    se_player: SePlayer,
}

impl AudioManager {
    pub fn new() -> Self {
        let mut manager = kira::manager::AudioManager::new(AudioManagerSettings::default())
            .expect("Failed to create kira audio manager");

        let bgm_track = manager
            .add_sub_track(TrackBuilder::new().routes(TrackRoutes::parent(TrackId::Main)))
            .expect("Failed to create bgm track");

        let bgm_player = BgmPlayer::new(bgm_track);

        let all_se_track = manager
            .add_sub_track(TrackBuilder::new().routes(TrackRoutes::parent(TrackId::Main)))
            .expect("Failed to create all_se track");

        let se_tracks = [(); SE_SLOT_COUNT].map(|_| {
            manager
                .add_sub_track(TrackBuilder::new().routes(TrackRoutes::parent(all_se_track.id())))
                .expect("Failed to create se track")
        });

        let se_player = SePlayer::new(all_se_track, se_tracks);

        Self {
            manager: Mutex::new(manager),
            bgm_player,
            se_player,
        }
    }
}
