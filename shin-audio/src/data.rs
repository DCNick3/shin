//! Implements the SoundData trait for the Kira audio library.

use crate::handle::AudioHandle;
use crate::sound::{AudioSound, COMMAND_BUFFER_CAPACITY};
use anyhow::Result;
use kira::sound::{Sound, SoundData};
use ringbuf::HeapRb;
use shin_core::format::audio::AudioFile;
use std::sync::Arc;

use super::AudioSettings;

pub struct AudioData {
    pub file: Arc<AudioFile>,
    pub settings: AudioSettings,
}

impl AudioData {
    pub fn new(audio: Arc<AudioFile>, settings: AudioSettings) -> Self {
        Self {
            file: audio,
            settings,
        }
    }
}

impl SoundData for AudioData {
    type Error = anyhow::Error;
    type Handle = AudioHandle;

    fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error> {
        let (sound, handle) = self.split();
        Ok((Box::new(sound), handle))
    }
}

impl AudioData {
    fn split(self) -> (AudioSound, AudioHandle) {
        let (command_producer, command_consumer) = HeapRb::new(COMMAND_BUFFER_CAPACITY).split();

        let sound = AudioSound::new(self, command_consumer);
        let shared = sound.shared();

        (
            sound,
            AudioHandle {
                command_producer,
                shared,
            },
        )
    }
}
