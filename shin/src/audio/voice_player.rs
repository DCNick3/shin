use std::sync::Arc;

use bitflags::bitflags;
use shin_audio::AudioManager;
use shin_core::{
    format::scenario::Scenario,
    vm::command::types::{AudioWaitStatus, Volume},
};

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct VoicePlayFlags: i32 {
        const ENABLE_CHARACTER_LIPSYNC = 1;
        const ENABLE_CHARACTER_MUTING = 2;
    }
}

pub struct VoicePlayer {
    audio_manager: Arc<AudioManager>,
}

impl VoicePlayer {
    pub fn new(audio_manager: Arc<AudioManager>) -> Self {
        Self { audio_manager }
    }

    pub fn play(
        &mut self,
        _scenario: &Scenario,
        _voicefiles_spec: &str,
        _segment_start: u32,
        _segment_duration: u32,
        _flags: VoicePlayFlags,
        _volume: Volume,
    ) -> bool {
        false
    }

    pub fn get_wait_status(&self) -> AudioWaitStatus {
        AudioWaitStatus::empty()
    }
}
