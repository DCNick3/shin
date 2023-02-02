pub mod layers;

use layers::LayersState;
use shin_core::vm::command::layer::{MessageboxStyle, VLayerId, VLayerIdRepr};

pub struct SaveInfo {
    pub info: [String; 4],
}

impl SaveInfo {
    pub fn set_save_info(&mut self, level: i32, info: String) {
        assert!(
            (0..=4).contains(&level),
            "SaveInfo::set_save_info: level out of range"
        );

        self.info[level as usize] = info;
    }
}

#[derive(Debug)]
pub struct MessageState {
    pub msginit: MessageboxStyle,
    pub messagebox_shown: bool,
    pub text: Option<String>,
}

impl MessageState {
    pub fn new() -> Self {
        Self {
            msginit: MessageboxStyle::default(),
            messagebox_shown: false,
            text: None,
        }
    }
}

pub struct Globals {
    globals: [i32; 0x100],
}

impl Globals {
    pub fn new() -> Self {
        Self {
            globals: [0; 0x100],
        }
    }

    pub fn get(&self, id: i32) -> i32 {
        assert!(
            (0x0..0x100).contains(&id),
            "GlobalsInfo::get: id out of range"
        );
        self.globals[id as usize]
    }

    pub fn set(&mut self, id: i32, value: i32) {
        assert!(
            (0x0..0x100).contains(&id),
            "GlobalsInfo::set: id out of range"
        );
        self.globals[id as usize] = value;
    }
}

pub struct VmState {
    pub save_info: SaveInfo,
    pub messagebox_state: MessageState,
    pub globals: Globals,
    pub layers: LayersState,
}

impl VmState {
    pub fn new() -> Self {
        Self {
            save_info: SaveInfo {
                info: ["", "", "", ""].map(|v| v.to_string()),
            },
            messagebox_state: MessageState::new(),
            globals: Globals::new(),
            layers: LayersState::new(),
        }
    }
}
