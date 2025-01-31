use shin_core::vm::command::types::Volume;
use shin_derive::RenderClone;

#[derive(Debug, Clone, RenderClone)]
pub struct Block {
    pub time: f32, // Ticks?
    pub ty: BlockType,
}

#[derive(Debug, Clone, RenderClone)]
pub enum BlockType {
    Voice(Voice),
    Wait(Wait),
    Section(Section),
    Sync(Sync),
    VoiceSync(VoiceSync),
    VoiceWait(VoiceWait),
}

#[derive(Debug, Clone, RenderClone)]
pub struct Voice {
    pub filename: String,
    pub volume: Volume,
    pub lipsync_enabled: bool,
    pub segment_duration: u32, // newtype?
}
#[derive(Debug, Clone, RenderClone)]
pub struct Wait {
    pub wait_auto_delay: f32,
    pub is_last_wait: bool,
    pub is_auto_click: bool,
}
#[derive(Debug, Clone, RenderClone)]
pub struct Section {
    pub index: u32,
}
#[derive(Debug, Clone, RenderClone)]
pub struct Sync {
    pub index: u32,
}
#[derive(Debug, Clone, RenderClone)]
pub struct VoiceSync {
    pub segment_start: u32,
    pub segment_duration: u32,
}
#[derive(Debug, Clone, RenderClone)]
pub struct VoiceWait;
