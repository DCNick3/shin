use glam::Vec2;
use shin_primitives::color::UnormColor;

#[derive(Debug, PartialEq)]
pub struct Char {
    pub time: f32,
    pub line_index: usize,
    // TODO: font
    pub codepoint: char,
    pub is_rubi: bool,
    pub cant_be_at_line_start: bool,
    pub cant_be_at_line_end: bool,
    pub has_rubi: bool,
    pub width: f32,
    pub height: f32,
    pub position: Vec2,
    pub horizontal_scale: f32,
    pub scale: f32,
    pub color: UnormColor,
    pub fade: f32,
}

impl Char {
    pub fn right_border(&self) -> f32 {
        self.position.x + self.width
    }
}

#[derive(Debug, PartialEq)]
pub struct Section {
    pub time: f32,
    pub line_index: usize,
    pub index: u32,
}

#[derive(Debug, PartialEq)]
pub struct Sync {
    pub time: f32,
    pub line_index: usize,
    pub index: u32,
}

#[derive(Debug, PartialEq)]
pub struct Voice {
    pub time: f32,
    pub line_index: usize,
    pub filename: String,
    pub volume: f32,
    pub lipsync_enabled: bool,
    pub time_to_first_sync: i32, // TODO: integer ticks?
}

#[derive(Debug, PartialEq)]
pub struct VoiceSync {
    pub time: f32,
    pub line_index: usize,
    pub target_instant: i32,
    pub time_to_next_sync: i32,
}

#[derive(Debug, PartialEq)]
pub struct VoiceWait {
    pub time: f32,
    pub line_index: usize,
}

#[derive(Debug, PartialEq)]
pub struct Wait {
    pub time: f32,
    pub line_index: usize,
    pub is_last_wait: bool,
    pub is_auto_click: bool,
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Char(Char),
    Section(Section),
    Sync(Sync),
    Voice(Voice),
    VoiceSync(VoiceSync),
    VoiceWait(VoiceWait),
    Wait(Wait),
}

impl Command {
    pub fn time(&self) -> f32 {
        match self {
            Command::Char(char) => char.time,
            Command::Section(section) => section.time,
            Command::Sync(sync) => sync.time,
            Command::Voice(voice) => voice.time,
            Command::VoiceSync(sync) => sync.time,
            Command::VoiceWait(wait) => wait.time,
            Command::Wait(wait) => wait.time,
        }
    }

    pub fn set_line_index(&mut self, index: usize) {
        match self {
            Command::Char(char) => char.line_index = index,
            Command::Section(section) => section.line_index = index,
            Command::Sync(sync) => sync.line_index = index,
            Command::Voice(voice) => voice.line_index = index,
            Command::VoiceSync(sync) => sync.line_index = index,
            Command::VoiceWait(wait) => wait.line_index = index,
            Command::Wait(wait) => wait.line_index = index,
        }
    }
}
