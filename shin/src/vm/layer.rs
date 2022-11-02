pub const LAYERBANKS_COUNT: u8 = 0x30;
pub const LAYERS_COUNT: u32 = 0x100;
pub const PLANES_COUNT: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerbankId(u8);
impl LayerbankId {
    pub const NONE: Self = Self(LAYERBANKS_COUNT);

    pub fn new(id: u8) -> Self {
        assert!(id < LAYERBANKS_COUNT, "LayerbankId::new: id out of range");
        Self(id)
    }

    pub fn ok(self) -> Option<Self> {
        if self.0 < LAYERBANKS_COUNT {
            Some(self)
        } else {
            None
        }
    }

    pub fn to_raw(self) -> Option<u32> {
        if self.0 == LAYERBANKS_COUNT {
            None
        } else {
            Some(self.0 as u32)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerId(u32);
impl LayerId {
    pub const NONE: Self = Self(LAYERS_COUNT);

    pub fn new(id: u32) -> Self {
        assert!(id < LAYERS_COUNT, "LayerId::new: id out of range");
        Self(id)
    }

    pub fn ok(self) -> Option<Self> {
        if self.0 < LAYERS_COUNT {
            Some(self)
        } else {
            None
        }
    }

    pub fn to_raw(self) -> Option<u32> {
        if self.0 == LAYERS_COUNT {
            None
        } else {
            Some(self.0)
        }
    }
}

impl From<shin_core::vm::command::LayerId> for LayerId {
    fn from(id: shin_core::vm::command::LayerId) -> Self {
        Self::new(id.0)
    }
}
