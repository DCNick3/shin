use std::fmt::{Debug, Formatter};

use shin_core::vm::command::types::{
    LAYERBANKS_COUNT, LayerId, LayerLoadFlags, LayerType, PlaneId,
};
use shin_render::shaders::types::{RenderClone, RenderCloneCtx};
use shin_tasks::AsyncTask;
use tracing::error;

use super::prelude::*;
use crate::{
    adv::vm_state::layers::LayerOperationTarget,
    layer::{LayerProperties, user::UserLayer},
};

pub struct LAYERLOAD {
    token: Option<command::token::LAYERLOAD>,
    state_info: LayerLoadStateInfo,
    load_task: AsyncTask<UserLayer>,
}

struct LayerInfo {
    operation_target: LayerOperationTarget,
    // yes, this is duplicated with the field above
    // don't ask me why
    layer_id_for_properties: LayerId,
    layer_load_counter1: u32,
    keep_old_props: bool,
    already_the_same: bool,
}

pub struct LayerLoadStateInfo {
    skip_layerloader_creation: bool,
    plane: PlaneId,
    affected_layers: heapless::Vec<LayerInfo, { LAYERBANKS_COUNT }>,
}

impl StartableCommand for command::runtime::LAYERLOAD {
    type StateInfo = LayerLoadStateInfo;
    fn apply_state(&self, state: &mut VmState) -> LayerLoadStateInfo {
        let layers = &mut state.layers;

        let plane = layers.current_plane;

        // Umineko doesn't use them really, and some of them I don't understand yet
        assert_eq!(self.flags, LayerLoadFlags::empty());

        let mut state_info = LayerLoadStateInfo {
            skip_layerloader_creation: true,
            plane,
            affected_layers: heapless::Vec::new(),
        };

        let mut add_layer = |layer: LayerId| {
            let Some(layerbank) = layers.layerbank_allocator.alloc_layerbank(plane, layer) else {
                error!("Layerbank allocation failed");
                return;
            };

            let Ok(()) = state_info.affected_layers.push(LayerInfo {
                operation_target: LayerOperationTarget { layerbank, layer },
                // these will be set later
                layer_id_for_properties: LayerId::new(0),
                layer_load_counter1: 0,
                keep_old_props: false,
                already_the_same: false,
            }) else {
                unreachable!()
            };
        };

        match self.layer_id.repr() {
            VLayerIdRepr::RootLayerGroup
            | VLayerIdRepr::ScreenLayer
            | VLayerIdRepr::PageLayer
            | VLayerIdRepr::PlaneLayerGroup => {
                warn!(
                    "Attempt to load a special layer ({:?})",
                    self.layer_id.repr()
                );
                return state_info;
            }
            VLayerIdRepr::Selected => {
                for layer in layers.layer_selection.iter() {
                    add_layer(layer);
                }
            }
            VLayerIdRepr::Layer(layer) => {
                add_layer(layer);
            }
        }

        for info in &mut state_info.affected_layers {
            let state = &mut layers.layerbanks[info.operation_target.layerbank];

            let old_layer_type = state.layer_type;

            if state.layer_type == Some(self.layer_type)
                && (old_layer_type == Some(LayerType::Quiz) || state.params == self.params)
            {
                info.already_the_same = true;
            }
            state.layer_type = Some(self.layer_type);
            state.params = self.params;
            if self.layer_type == LayerType::Quiz {
                state.is_interation_completed = false;
            }
            if info.already_the_same {
                state.layer_load_counter = layers.layer_load_counter;
                layers.layer_load_counter += 1;
                state_info.skip_layerloader_creation = false;
            } else {
                state.layer_load_counter = layers.layer_load_counter;
                // no increment here!
            }
            if old_layer_type.is_some()
                && self
                    .flags
                    .contains(LayerLoadFlags::KEEP_PREVIOUS_PROPERTIES)
            {
                info.keep_old_props = true;
            } else {
                state.properties.init();
                state.layer_id = info.operation_target.layer;
                info.layer_id_for_properties = info.operation_target.layer;
                info.keep_old_props = false;
                state.plane = plane;
                layers.layer_load_with_init_counter += 1;
            }
        }

        state_info
    }

    fn start(
        self,
        context: &mut UpdateContext,
        scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        state_info: LayerLoadStateInfo,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        let asset_server = context.asset_server.clone();
        let audio_manager = adv_state.audio_manager.clone();
        let scenario = scenario.clone();

        let device = context.pre_render.device.clone();
        let load_task = shin_tasks::async_io::spawn(async move {
            UserLayer::load(
                &device,
                &asset_server,
                &audio_manager,
                &scenario,
                self.layer_type,
                self.params,
            )
            .await
        });

        if !self.flags.contains(LayerLoadFlags::DONT_BLOCK_ANIMATIONS) {
            adv_state.allow_running_animations = false;
        }

        // TODO: optimistically block for 5ms as the game does

        Yield(
            LAYERLOAD {
                token: Some(self.token),
                state_info,
                load_task,
            }
            .into(),
        )
    }
}

impl UpdatableCommand for LAYERLOAD {
    fn update(
        &mut self,
        context: &mut UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
        _is_fast_forwarding: bool,
    ) -> Option<CommandResult> {
        let Some(layer) = self.load_task.poll_naive() else {
            return None;
        };

        // NB: here the game also loads a wiper, but we don't support `LayerGroup`-level wiping

        adv_state.create_back_layer_group_if_needed(&mut context.pre_render.render_clone_ctx());

        let mut plane_layer_group = adv_state.plane_layer_group_mut(self.state_info.plane);

        for info in &self.state_info.affected_layers {
            // NOTE: the original game does not remove the previous layer here, but we can't to this because we don't Arc everything
            // this should not be a problem because we replace the layer with this layerbank id at the end of the for loop iteration
            let mut previous_layer =
                plane_layer_group.remove_layer(info.operation_target.layerbank, Ticks::ZERO);

            let mut previous_props = previous_layer
                .as_ref()
                .map(|layer| layer.properties().clone());

            let mut layer = if info.already_the_same {
                previous_layer.unwrap()
            } else {
                layer.render_clone(&mut context.pre_render.render_clone_ctx())
            };

            if let UserLayer::Bustup(layer) = &mut layer {
                // TODO: connect the BustupLayer to the lipsync machinery
            }

            let mut properties = match (previous_props, info.keep_old_props) {
                (Some(previous_props), true) => previous_props,
                _ => {
                    let mut properties = LayerProperties::new();
                    properties.set_layer_id(info.layer_id_for_properties);
                    properties
                }
            };

            properties.set_layerload_counter1(info.layer_load_counter1);
            *layer.properties_mut() = properties;

            plane_layer_group.add_layer(info.operation_target.layerbank, layer);
        }

        // TODO: call function to unlock layer-related items in CG (for PictureLayer) and Movie & BGM (for MovieLayer) modes

        adv_state.allow_running_animations = true;

        // NB: the original game waits for the wipe to end, but we don't need to do this

        Some(self.token.take().unwrap().finish())
    }
}

impl Debug for LAYERLOAD {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LAYERLOAD")
            .field(
                &self
                    .state_info
                    .affected_layers
                    .iter()
                    .map(|info| info.operation_target)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
