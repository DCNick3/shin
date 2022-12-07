use crate::layer::LayerPropertiesSnapshot;
use bevy_utils::hashbrown::hash_map::Entry;
use bevy_utils::StableHashMap;
use shin_core::vm::command::layer::{LayerId, LayerType, VLayerId, PLANES_COUNT};

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

pub struct MessageboxState {
    pub msginit: Option<i32>,
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

#[derive(Debug, Copy, Clone)]
pub struct LayerSelection {
    // TODO: enforce ordering?
    pub low: LayerId,
    pub high: LayerId,
}

#[derive(Debug, Copy, Clone)]
pub struct LayerState {
    pub layerinit_params: Option<(LayerType, [i32; 0x8])>,
    pub properties: LayerPropertiesSnapshot,
}

#[derive(Debug, Clone)]
pub struct PlaneState {
    // TODO: allocations - bad?
    pub layers: StableHashMap<LayerId, LayerState>,
}

impl PlaneState {
    pub fn new() -> Self {
        Self {
            layers: StableHashMap::default(),
        }
    }

    pub fn get_layer(&self, layer_id: LayerId) -> Option<&LayerState> {
        self.layers.get(&layer_id)
    }

    pub fn get_layer_mut(&mut self, layer_id: LayerId) -> Option<&mut LayerState> {
        self.layers.get_mut(&layer_id)
    }

    pub fn alloc(&mut self, layer_id: LayerId) -> &mut LayerState {
        match self.layers.entry(layer_id) {
            // TODO: downgrade to a warning?
            Entry::Occupied(_) => panic!("LayerState::alloc: layer already allocated"),
            Entry::Vacant(v) => v.insert(LayerState {
                layerinit_params: None,
                properties: LayerPropertiesSnapshot::new(),
            }),
        }
    }

    pub fn free(&mut self, layer_id: LayerId) {
        self.layers
            .remove(&layer_id)
            // TODO: downgrade to a warning?
            .expect("LayerState::free: layer not allocated");
    }
}

#[derive(Debug, Clone)]
pub struct LayersState {
    pub current_plane: u32,
    pub layer_selection: Option<LayerSelection>,
    pub planes: [PlaneState; PLANES_COUNT],
}

impl LayersState {
    pub fn new() -> Self {
        Self {
            current_plane: 0,
            layer_selection: None,
            planes: [
                PlaneState::new(),
                PlaneState::new(),
                PlaneState::new(),
                PlaneState::new(),
            ],
        }
    }

    /// Get user layer by id
    pub fn get_layer(&self, layer_id: LayerId) -> Option<&LayerState> {
        self.planes[self.current_plane as usize].get_layer(layer_id)
    }

    /// Get user layer by id (mutable)
    pub fn get_layer_mut(&mut self, layer_id: LayerId) -> Option<&mut LayerState> {
        self.planes[self.current_plane as usize].get_layer_mut(layer_id)
    }

    /// Get layer by id, handling the special layers & selection
    pub fn get_vlayer(&self, _vlayer_id: VLayerId) -> impl Iterator<Item = &LayerState> {
        // TODO: implement
        // if a special layer - return a single layer
        // if a normal layer id - return it if exists, otherwise print a warning and return an empty iterator
        // if a selection - return all layers in the selection and warn if the selection is empty
        std::iter::once(todo!())
    }

    /// Get layer by id, handling the special layers & selection (mutable)
    pub fn get_vlayer_mut(
        &mut self,
        _vlayer_id: VLayerId,
    ) -> impl Iterator<Item = &mut LayerState> {
        // TODO: same as get_many, but mutable
        std::iter::once(todo!())
    }

    pub fn alloc(&mut self, layer_id: LayerId) -> &mut LayerState {
        self.planes[self.current_plane as usize].alloc(layer_id)
    }

    pub fn free(&mut self, layer_id: LayerId) {
        self.planes[self.current_plane as usize].free(layer_id)
    }
}

pub struct VmState {
    pub save_info: SaveInfo,
    pub messagebox_state: MessageboxState,
    pub globals: Globals,
    pub layers: LayersState,
}

impl VmState {
    pub fn new() -> Self {
        Self {
            save_info: SaveInfo {
                info: ["", "", "", ""].map(|v| v.to_string()),
            },
            messagebox_state: MessageboxState { msginit: None },
            globals: Globals::new(),
            layers: LayersState::new(),
        }
    }
}
