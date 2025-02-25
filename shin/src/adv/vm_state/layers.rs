use bevy_utils::hashbrown::hash_map::Entry;
use enum_map::{Enum, EnumMap};
use shin_core::{
    format::scenario::{
        info::{MaskId, MaskIdOpt},
        instruction_elements::UntypedNumberArray,
    },
    vm::command::types::{
        LAYERBANKS_COUNT, LAYERS_COUNT, LayerId, LayerIdOpt, LayerType, LayerbankId,
        LayerbankIdOpt, MaskFlags, PLANES_COUNT, PlaneId, PlaneIdOpt, VLayerId, VLayerIdRepr,
    },
};
use smallvec::{SmallVec, smallvec};
use tracing::{trace, warn};

use crate::layer::LayerPropertiesState;

#[derive(Debug, Copy, Clone)]
pub struct FullLayerId {
    pub plane: PlaneId,
    pub layer: LayerId,
}

impl Enum for FullLayerId {
    type Array<V> = [V; LAYERS_COUNT * PLANES_COUNT];

    fn from_usize(value: usize) -> Self {
        assert!(value < LAYERS_COUNT * PLANES_COUNT);
        let plane = value / LAYERS_COUNT;
        let layer = value % LAYERS_COUNT;
        Self {
            plane: PlaneId::new(plane as u8),
            layer: LayerId::new(layer as u16),
        }
    }

    fn into_usize(self) -> usize {
        self.plane.raw() as usize * LAYERS_COUNT + self.layer.raw() as usize
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FullLayerIdOpt {
    pub plane: PlaneIdOpt,
    pub layer: LayerIdOpt,
}
impl FullLayerIdOpt {
    pub fn none() -> Self {
        Self {
            plane: PlaneIdOpt::none(),
            layer: LayerIdOpt::none(),
        }
    }
    fn some(plane: PlaneId, layer: LayerId) -> FullLayerIdOpt {
        FullLayerIdOpt {
            plane: PlaneIdOpt::some(plane),
            layer: LayerIdOpt::some(layer),
        }
    }
    fn unwrap(self) -> FullLayerId {
        FullLayerId {
            plane: self.plane.unwrap(),
            layer: self.layer.unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
struct LayerRangeCache {
    plane: PlaneIdOpt,
    from: LayerId,
    to: LayerId,
    affected_layerbanks: [(LayerId, LayerbankId); LAYERBANKS_COUNT],
    affected_layerbank_count: u32,
}

impl LayerRangeCache {
    fn new() -> Self {
        Self {
            plane: PlaneIdOpt::none(),
            from: LayerId::new(0),
            to: LayerId::new(0),
            affected_layerbanks: [(LayerId::new(0), LayerbankId::new(0)); LAYERBANKS_COUNT],
            affected_layerbank_count: 0,
        }
    }

    fn invalidate(&mut self) {
        self.plane = PlaneIdOpt::none();
    }

    fn is_hit(&self, plane: PlaneId, from: LayerId, to: LayerId) -> bool {
        self.plane == PlaneIdOpt::some(plane) && self.from == from && self.to == to
    }

    fn clear(&mut self) {
        self.affected_layerbank_count = 0;
    }

    fn push(&mut self, layer: LayerId, layerbank: LayerbankId) {
        self.affected_layerbanks[self.affected_layerbank_count as usize] = (layer, layerbank);
        self.affected_layerbank_count += 1;
    }

    fn iter(&self) -> impl Iterator<Item = (LayerId, LayerbankId)> + '_ {
        self.affected_layerbanks[..self.affected_layerbank_count as usize]
            .iter()
            .copied()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LayerOperationTarget {
    pub layer: LayerId,
    pub layerbank: LayerbankId,
}

pub type LayerOperationTargetList = heapless::Vec<LayerOperationTarget, LAYERBANKS_COUNT>;

#[derive(Debug, Clone)]
pub struct LayerbankAllocator {
    free_layerbank_ids: [LayerbankId; LAYERBANKS_COUNT],
    allocated_layerbanks: u32,
    layer_id_to_layerbank: EnumMap<FullLayerId, LayerbankIdOpt>,
    layerbank_id_to_layer_id: EnumMap<LayerbankId, FullLayerIdOpt>,

    range_cache: LayerRangeCache,
}

#[expect(unused)] // for future stuff
impl LayerbankAllocator {
    pub fn new() -> Self {
        Self {
            free_layerbank_ids: core::array::from_fn(|i| LayerbankId::new(i as u8)),
            allocated_layerbanks: 0,
            layer_id_to_layerbank: EnumMap::from_fn(|_| LayerbankIdOpt::none()),
            layerbank_id_to_layer_id: EnumMap::from_fn(|_| FullLayerIdOpt::none()),
            range_cache: LayerRangeCache::new(),
        }
    }

    fn layer_id_index(plane: PlaneId, layer: LayerId) -> usize {
        plane.raw() as usize * LAYERS_COUNT + layer.raw() as usize
    }

    pub fn get_layerbank_id(&self, plane: PlaneId, layer: LayerId) -> Option<LayerbankId> {
        self.layer_id_to_layerbank[FullLayerId { plane, layer }].into_option()
    }

    pub fn alloc_layerbank(&mut self, plane: PlaneId, layer: LayerId) -> Option<LayerbankId> {
        let full_id = FullLayerId { plane, layer };

        // double allocation is fine
        if let Some(layerbank_id) = self.layer_id_to_layerbank[full_id].into_option() {
            return Some(layerbank_id);
        }

        if self.allocated_layerbanks >= LAYERBANKS_COUNT as u32 {
            // no more layerbanks to allocate :(
            return None;
        }

        let new_layerbank_id = self.free_layerbank_ids[self.allocated_layerbanks as usize];
        self.allocated_layerbanks += 1;

        self.layer_id_to_layerbank[full_id] = LayerbankIdOpt::some(new_layerbank_id);
        self.layerbank_id_to_layer_id[new_layerbank_id] = FullLayerIdOpt::some(plane, layer);

        self.range_cache.invalidate();

        Some(new_layerbank_id)
    }

    pub fn free_layerbank(&mut self, plane: PlaneId, layer: LayerId) {
        let full_id = FullLayerId { plane, layer };

        let Some(layerbank_id) = self.layer_id_to_layerbank[full_id].into_option() else {
            // layerbank not allocated
            return;
        };

        self.layer_id_to_layerbank[full_id] = LayerbankIdOpt::none();
        self.layerbank_id_to_layer_id[layerbank_id] = FullLayerIdOpt::none();
        self.allocated_layerbanks -= 1;
        self.free_layerbank_ids[self.allocated_layerbanks as usize] = layerbank_id;

        self.range_cache.invalidate();
    }

    pub fn for_layer_in_range(
        &mut self,
        plane: PlaneId,
        from: LayerId,
        to: LayerId,
        mut f: impl FnMut(LayerId, LayerbankId),
    ) {
        if from == to {
            let Some(layerbank_id) = self.get_layerbank_id(plane, from) else {
                return;
            };
            let layer_id = self.layerbank_id_to_layer_id[layerbank_id].unwrap().layer;
            f(layer_id, layerbank_id);
        }

        if !self.range_cache.is_hit(plane, from, to) {
            self.range_cache.clear();

            let mut position = from;
            while position <= to {
                if let Some(layerbank_id) = self.get_layerbank_id(plane, position) {
                    self.range_cache.push(position, layerbank_id);
                }

                let Some(next_position) = position.try_next() else {
                    // assuming this can only happen if to is at max
                    break;
                };
                position = next_position;
            }
        }

        for (layer_id, layerbank_id) in self.range_cache.iter() {
            f(layer_id, layerbank_id);
        }
    }

    pub fn layers_in_range(
        &mut self,
        plane: PlaneId,
        from: LayerId,
        to: LayerId,
    ) -> LayerOperationTargetList {
        let mut result = LayerOperationTargetList::new();

        self.for_layer_in_range(plane, from, to, |layer, layerbank| {
            let Ok(()) = result.push(LayerOperationTarget { layer, layerbank }) else {
                // the list must be big enough, we can't have more that `LAYERBANKS_COUNT` layers with assigned layerbanks
                unreachable!()
            };
        });

        trace!("Operating on layer range {:?}", result);

        result
    }

    pub fn swap_layerbanks(&mut self, plane: PlaneId, layer_1: LayerId, layer_2: LayerId) {
        if layer_1 == layer_2 {
            return;
        }

        let full_id_1 = FullLayerId {
            plane,
            layer: layer_1,
        };
        let full_id_2 = FullLayerId {
            plane,
            layer: layer_2,
        };

        let layerbank_1 = self.layer_id_to_layerbank[full_id_1];
        let layerbank_2 = self.layer_id_to_layerbank[full_id_2];
        self.layer_id_to_layerbank[full_id_1] = layerbank_2;
        self.layer_id_to_layerbank[full_id_2] = layerbank_1;

        if let Some(layerbank_1) = layerbank_1.into_option() {
            self.layerbank_id_to_layer_id[layerbank_1] = FullLayerIdOpt::some(plane, layer_2);
        }
        if let Some(layerbank_2) = layerbank_2.into_option() {
            self.layerbank_id_to_layer_id[layerbank_2] = FullLayerIdOpt::some(plane, layer_1);
        }

        self.range_cache.invalidate();
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LayerSelection {
    pub from: LayerId,
    pub to: LayerId,
}

impl LayerSelection {
    pub fn new() -> Self {
        Self {
            from: LayerId::new(0),
            to: LayerId::new(0),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = LayerId> + use<> {
        LayerSelectionIter {
            current: LayerIdOpt::some(self.from),
            high: self.to,
        }
    }

    pub fn contains(&self, id: LayerId) -> bool {
        self.from <= id && id <= self.to
    }
}

struct LayerSelectionIter {
    current: LayerIdOpt,
    high: LayerId,
}

impl Iterator for LayerSelectionIter {
    type Item = LayerId;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.into_option() {
            None => None,
            Some(current) => {
                if current > self.high {
                    None
                } else {
                    if current == self.high {
                        self.current = LayerIdOpt::none();
                    } else {
                        // NB: current.next() will panic if the layer id is out of range
                        // but this shouldn't happen because `high` is always in range
                        self.current = LayerIdOpt::some(current.next());
                    }

                    Some(current)
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayerState {
    pub layerinit_params: Option<(LayerType, UntypedNumberArray)>,
    pub properties: LayerPropertiesState,
}

impl LayerState {
    pub fn new() -> Self {
        Self {
            layerinit_params: None,
            properties: LayerPropertiesState::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlaneLayerGroupState {
    pub properties: LayerPropertiesState,
    pub mask_id: MaskIdOpt,
    pub mask_flags: MaskFlags,
}

impl PlaneLayerGroupState {
    pub fn new() -> Self {
        Self {
            properties: LayerPropertiesState::new(),
            mask_id: MaskIdOpt::none(),
            mask_flags: MaskFlags::empty(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayerbankState {
    // None -> the layer is unloaded and most other info is stale
    pub layer_type: Option<LayerType>,
    pub plane: PlaneId,
    pub layer_id: LayerId,
    pub layer_load_counter: u32,
    // TODO: there are some fields which don't seem used?
    pub params: UntypedNumberArray,
    pub properties: LayerPropertiesState,
    pub is_interation_completed: bool,
}

impl LayerbankState {
    pub fn new() -> Self {
        Self {
            layer_type: None,
            plane: PlaneId::new(0),
            layer_id: LayerId::new(0),
            layer_load_counter: 0,
            params: (0, 0, 0, 0, 0, 0, 0, 0),
            properties: LayerPropertiesState::new(),
            is_interation_completed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayersState {
    pub root_layer_group: LayerPropertiesState,
    pub screen_layer: LayerPropertiesState,
    pub page_layer: LayerPropertiesState,
    pub plane_layergroups: EnumMap<PlaneId, PlaneLayerGroupState>,
    // NB: missing wiper state set by TRANSSET here
    // probably not a problem with umineko not utilizing this system
    pub layerbank_allocator: LayerbankAllocator,
    pub layer_selection: LayerSelection,
    pub current_plane: PlaneId,
    pub is_page_back_started: bool,
    pub layer_load_with_init_counter: u32,
    pub layer_load_counter: u32,
    pub layerbanks: EnumMap<LayerbankId, LayerbankState>,
}

/// can be whatever, just an optimization. Ideally, most selections made by the script should fit in
pub const ITER_VLAYER_SMALL_VECTOR_SIZE: usize = 0x10;

impl LayersState {
    pub fn new() -> Self {
        Self {
            root_layer_group: LayerPropertiesState::new(),
            screen_layer: LayerPropertiesState::new(),
            page_layer: LayerPropertiesState::new(),
            plane_layergroups: EnumMap::from_fn(|_| PlaneLayerGroupState::new()),
            layerbank_allocator: LayerbankAllocator::new(),
            layer_selection: LayerSelection::new(),
            current_plane: PlaneId::new(0),
            is_page_back_started: false,
            layer_load_with_init_counter: 0,
            layer_load_counter: 0,
            layerbanks: EnumMap::from_fn(|_| LayerbankState::new()),
        }
    }
}
