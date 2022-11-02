use arrayvec::ArrayVec;
use shin_core::vm::command::layer_id::{
    LayerId, LayerIdOpt, LayerbankId, LayerbankIdOpt, LAYERBANKS_COUNT, LAYERS_COUNT,
};

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

pub struct MsgInfo {
    pub msginit: Option<i32>,
}

pub struct GlobalsInfo {
    globals: [i32; 0x100],
}

impl GlobalsInfo {
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

/// Manages mapping between layer IDs and layer bank IDs, as well as allocation
pub struct LayerbankInfo {
    free_layerbanks: ArrayVec<LayerbankId, { LAYERBANKS_COUNT as usize }>,

    // TODO: handle layer planes
    layerbank_id_to_layer_id: [LayerIdOpt; LAYERBANKS_COUNT as usize],
    layer_id_to_layerbank_id: [LayerbankIdOpt; 0x100],
}

impl LayerbankInfo {
    pub fn new() -> Self {
        Self {
            free_layerbanks: (0..LAYERBANKS_COUNT).map(LayerbankId::new).collect(),
            layerbank_id_to_layer_id: [LayerIdOpt::none(); LAYERBANKS_COUNT as usize],
            layer_id_to_layerbank_id: [LayerbankIdOpt::none(); LAYERS_COUNT as usize],
        }
    }

    pub fn get_layerbank_id(&self, layer_id: LayerId) -> Option<LayerbankId> {
        self.layer_id_to_layerbank_id[layer_id.raw() as usize].opt()
    }

    fn alloc_layerbank(&mut self) -> Option<LayerbankId> {
        self.free_layerbanks.pop()
    }

    pub fn get_or_allocate_layerbank_id(&mut self, layer_id: LayerId) -> Option<LayerbankId> {
        if let Some(id) = self.layer_id_to_layerbank_id[layer_id.raw() as usize].opt() {
            Some(id)
        } else if let Some(id) = self.alloc_layerbank() {
            self.layer_id_to_layerbank_id[layer_id.raw() as usize] = LayerbankIdOpt::some(id);
            Some(id)
        } else {
            None
        }
    }
}

pub struct VmState {
    pub save_info: SaveInfo,
    pub msg_info: MsgInfo,
    pub globals_info: GlobalsInfo,
    pub layerbank_info: LayerbankInfo,
}

impl VmState {
    pub fn new() -> Self {
        Self {
            save_info: SaveInfo {
                info: ["", "", "", ""].map(|v| v.to_string()),
            },
            msg_info: MsgInfo { msginit: None },
            globals_info: GlobalsInfo::new(),
            layerbank_info: LayerbankInfo::new(),
        }
    }
}
