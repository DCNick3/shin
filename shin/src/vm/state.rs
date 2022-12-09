use crate::layer::LayerPropertiesSnapshot;
use bevy_utils::hashbrown::hash_map::Entry;
use bevy_utils::StableHashMap;
use shin_core::vm::command::layer::{
    LayerId, LayerIdOpt, LayerType, MessageboxStyle, VLayerId, VLayerIdRepr, PLANES_COUNT,
};
use tracing::warn;

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

#[derive(Debug, Copy, Clone)]
pub struct LayerSelection {
    // TODO: enforce ordering?
    pub low: LayerId,
    pub high: LayerId,
}

impl LayerSelection {
    pub fn iter(&self) -> impl Iterator<Item = LayerId> {
        LayerSelectionIter {
            current: LayerIdOpt::some(self.low),
            high: self.high,
        }
    }
}

struct LayerSelectionIter {
    current: LayerIdOpt,
    high: LayerId,
}

impl Iterator for LayerSelectionIter {
    type Item = LayerId;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.opt() {
            None => None,
            Some(current) => {
                if current > self.high {
                    None
                } else {
                    if current == self.high {
                        self.current = LayerIdOpt::none();
                    } else {
                        self.current = LayerIdOpt::some(current.next());
                    }

                    Some(current)
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LayerState {
    pub layerinit_params: Option<(LayerType, [i32; 0x8])>,
    pub properties: LayerPropertiesSnapshot,
}

impl LayerState {
    pub fn new() -> Self {
        Self {
            layerinit_params: None,
            properties: LayerPropertiesSnapshot::new(),
        }
    }
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
        if self.layers.remove(&layer_id).is_none() {
            warn!("LayerState::free: layer not allocated");
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayersState {
    pub current_plane: u32,
    pub layer_selection: Option<LayerSelection>,
    pub planes: [PlaneState; PLANES_COUNT],

    pub root_layer_group: LayerState,
    pub screen_layer: LayerState,
    pub page_layer: LayerState,
    pub plane_layer_group: LayerState,
}

pub enum LayersIter<'a> {
    Single(Option<&'a LayerState>),
    Selection {
        done: bool,
        plane: &'a PlaneState,
        low: LayerId,
        high: LayerId,
    },
}

impl<'a> Iterator for LayersIter<'a> {
    type Item = &'a LayerState;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            LayersIter::Single(layer) => layer.take(),
            LayersIter::Selection {
                done,
                plane,
                low,
                high,
            } => loop {
                if *done {
                    return None;
                }

                let id = *low;
                if *low == *high {
                    *done = true;
                } else {
                    *low = low.next();
                }

                if let Some(l) = plane.layers.get(&id) {
                    return Some(l);
                }
            },
        }
    }
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
            root_layer_group: LayerState::new(),
            screen_layer: LayerState::new(),
            page_layer: LayerState::new(),
            plane_layer_group: LayerState::new(),
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
    pub fn get_vlayer(&self, vlayer_id: VLayerId) -> LayersIter {
        // if a special layer - return a single layer
        // if a normal layer id - return it if exists, otherwise print a warning and return an empty iterator
        // if a selection - return all layers in the selection and warn if the selection is empty
        match vlayer_id.repr() {
            VLayerIdRepr::RootLayerGroup => LayersIter::Single(Some(&self.root_layer_group)),
            VLayerIdRepr::ScreenLayer => LayersIter::Single(Some(&self.screen_layer)),
            VLayerIdRepr::PageLayer => LayersIter::Single(Some(&self.page_layer)),
            VLayerIdRepr::PlaneLayerGroup => LayersIter::Single(Some(&self.plane_layer_group)),
            VLayerIdRepr::Selected => {
                if let Some(selection) = self.layer_selection {
                    LayersIter::Selection {
                        done: false,
                        plane: &self.planes[self.current_plane as usize],
                        low: selection.low,
                        high: selection.high,
                    }
                } else {
                    warn!("LayersState::get_vlayer: no selection");
                    LayersIter::Single(None)
                }
            }
            VLayerIdRepr::Layer(l) => {
                let v = self.get_layer(l);
                if v.is_none() {
                    warn!("get_vlayer: layer not found: {:?}", l);
                }
                LayersIter::Single(v)
            }
        }
    }

    /// Get layer by id, handling the special layers & selection (mutable)
    pub fn for_each_vlayer_mut(&mut self, vlayer_id: VLayerId, mut f: impl FnMut(&mut LayerState)) {
        // same as get_vlayer, but mutable
        match vlayer_id.repr() {
            VLayerIdRepr::RootLayerGroup => f(&mut self.root_layer_group),
            VLayerIdRepr::ScreenLayer => f(&mut self.screen_layer),
            VLayerIdRepr::PageLayer => f(&mut self.page_layer),
            VLayerIdRepr::PlaneLayerGroup => f(&mut self.plane_layer_group),
            VLayerIdRepr::Selected => {
                if let Some(selection) = self.layer_selection {
                    let plane = &mut self.planes[self.current_plane as usize];
                    for id in selection.iter() {
                        if let Some(l) = plane.layers.get_mut(&id) {
                            f(l);
                        }
                    }
                } else {
                    warn!("LayersState::get_vlayer: no selection");
                }
            }
            VLayerIdRepr::Layer(l) => match self.get_layer_mut(l) {
                None => warn!("get_vlayer: layer not found: {:?}", l),
                Some(l) => f(l),
            },
        }
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
