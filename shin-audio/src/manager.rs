use kira::manager::AudioManagerSettings;
use kira::sound::SoundData;
use std::sync::Mutex;

type Backend = kira::manager::backend::cpal::CpalBackend;

pub struct AudioManager {
    manager: Mutex<kira::manager::AudioManager<Backend>>,
}

impl AudioManager {
    pub fn new() -> Self {
        let manager = kira::manager::AudioManager::new(AudioManagerSettings::default())
            .expect("Failed to create kira audio manager");

        Self {
            manager: Mutex::new(manager),
        }
    }

    pub fn play<S: SoundData>(&self, data: S) -> S::Handle
    where
        S::Error: std::fmt::Debug,
    {
        let mut manager = self.manager.lock().unwrap();

        manager.play(data).expect("Failed to start playing audio")
    }

    pub fn kira_manager(&self) -> &Mutex<kira::manager::AudioManager<Backend>> {
        &self.manager
    }
}
