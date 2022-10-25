use arrayvec::ArrayVec;

const LAYERBANKS_COUNT: u8 = 0x30;
const LAYERS_COUNT: u32 = 0x100;
const PLANES_COUNT: u32 = 4;

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

/// Manages mapping between layer IDs and layer bank IDs, as well as allocation
pub struct LayerbankInfo {
    free_layerbanks: ArrayVec<LayerbankId, { LAYERBANKS_COUNT as usize }>,

    // TODO: handle layer planes
    layerbank_id_to_layer_id: [LayerId; LAYERBANKS_COUNT as usize],
    layer_id_to_layerbank_id: [LayerbankId; 0x100],
}

impl LayerbankInfo {
    pub fn new() -> Self {
        Self {
            free_layerbanks: (0..LAYERBANKS_COUNT).map(LayerbankId).collect(),
            layerbank_id_to_layer_id: [LayerId::NONE; LAYERBANKS_COUNT as usize],
            layer_id_to_layerbank_id: [LayerbankId::NONE; LAYERS_COUNT as usize],
        }
    }

    pub fn get_layerbank_id(&self, layer_id: LayerId) -> Option<LayerbankId> {
        self.layer_id_to_layerbank_id[layer_id.0 as usize].ok()
    }

    fn alloc_layerbank(&mut self) -> Option<LayerbankId> {
        self.free_layerbanks.pop()
    }

    pub fn get_or_allocate_layerbank_id(&mut self, layer_id: LayerId) -> Option<LayerbankId> {
        if let Some(id) = self.layer_id_to_layerbank_id[layer_id.0 as usize].ok() {
            Some(id)
        } else if let Some(id) = self.alloc_layerbank() {
            self.layer_id_to_layerbank_id[layer_id.0 as usize] = id;
            Some(id)
        } else {
            None
        }
    }
}
