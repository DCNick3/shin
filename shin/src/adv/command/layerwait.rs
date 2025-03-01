use std::fmt::{Debug, Formatter};

use shin_core::{
    format::scenario::types::SMALL_LIST_SIZE,
    vm::command::types::{LayerProperty, PlaneId},
};
use smallvec::SmallVec;

use super::prelude::*;
use crate::{adv::vm_state::layers::LayerOperationTargetList, layer::LayerProperties};

pub struct LAYERWAIT {
    plane: PlaneId,
    layer_id: VLayerId,
    affected_layers: LayerOperationTargetList,
    properties: SmallVec<LayerProperty, { SMALL_LIST_SIZE }>,
    token: Option<command::token::LAYERWAIT>,
}

impl StartableCommand for command::runtime::LAYERWAIT {
    type StateInfo = LayerOperationTargetList;
    fn apply_state(&self, state: &mut VmState) -> LayerOperationTargetList {
        let layers = &mut state.layers;

        let mut affected_layers = Default::default();

        match self.layer_id.repr() {
            VLayerIdRepr::RootLayerGroup
            | VLayerIdRepr::ScreenLayer
            | VLayerIdRepr::PageLayer
            | VLayerIdRepr::PlaneLayerGroup => {}
            VLayerIdRepr::Selected => {
                affected_layers = layers.layerbank_allocator.layers_in_range(
                    layers.current_plane,
                    layers.layer_selection.from,
                    layers.layer_selection.to,
                )
            }
            VLayerIdRepr::Layer(layer_id) => {
                affected_layers = layers.layerbank_allocator.layers_in_range(
                    layers.current_plane,
                    layer_id,
                    layer_id,
                )
            }
        };

        affected_layers
    }

    fn start(
        self,
        _context: &mut UpdateContext,
        _scenario: &Arc<Scenario>,
        vm_state: &VmState,
        affected_layers: LayerOperationTargetList,
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        Yield(
            LAYERWAIT {
                plane: vm_state.layers.current_plane,
                layer_id: self.layer_id,
                affected_layers,
                properties: self.wait_properties,
                token: Some(self.token),
            }
            .into(),
        )
    }
}

impl UpdatableCommand for LAYERWAIT {
    fn update(
        &mut self,
        _context: &mut UpdateContext,
        _scenario: &Arc<Scenario>,
        vm_state: &VmState,
        adv_state: &mut AdvState,
        #[expect(unused)] is_fast_forwarding: bool,
    ) -> Option<CommandResult> {
        let does_need_wait = |props: &LayerProperties| {
            // TODO: implement fast-forwarding of tweeners
            self.properties
                .iter()
                .any(|&prop| !props.property_tweener(prop).is_idle())
        };

        let needs_wait = match self.layer_id.repr() {
            VLayerIdRepr::RootLayerGroup => {
                does_need_wait(adv_state.root_layer_group().properties())
            }
            VLayerIdRepr::ScreenLayer => does_need_wait(adv_state.screen_layer().properties()),
            VLayerIdRepr::PageLayer => does_need_wait(adv_state.page_layer().properties()),
            VLayerIdRepr::PlaneLayerGroup => does_need_wait(
                adv_state
                    .plane_layer_group(vm_state.layers.current_plane)
                    .properties(),
            ),
            VLayerIdRepr::Selected | VLayerIdRepr::Layer(_) => {
                self.affected_layers.iter().any(|target| {
                    does_need_wait(
                        adv_state
                            .plane_layer_group(self.plane)
                            .get_layer(target.layerbank)
                            .expect("BUG: layerbank not found")
                            .properties(),
                    )
                })
            }
        };

        if needs_wait {
            None
        } else {
            Some(self.token.take().unwrap().finish())
        }
    }
}

impl Debug for LAYERWAIT {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LAYERWAIT")
            .field(&self.layer_id)
            .field(&self.properties)
            .finish()
    }
}
