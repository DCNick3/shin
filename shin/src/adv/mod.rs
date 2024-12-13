pub mod assets;
mod command;
mod vm_state;

use std::{borrow::Cow, sync::Arc};

pub use command::{CommandStartResult, ExecutingCommand, StartableCommand, UpdatableCommand};
use egui::Window;
use enum_map::{enum_map, Enum};
use glam::Mat4;
use itertools::Itertools;
use shin_audio::AudioManager;
use shin_core::{
    format::scenario::{instruction_elements::CodeAddress, Scenario},
    vm::{
        breakpoint::BreakpointObserver,
        command::{
            types::{LayerId, VLayerId, VLayerIdRepr, PLANES_COUNT},
            CommandResult,
        },
        Scripter,
    },
};
use shin_input::{inputs::MouseButton, Action, ActionState};
use shin_render::render_pass::RenderPass;
use smallvec::{smallvec, SmallVec};
use tracing::{debug, warn};
use vm_state::layers::ITER_VLAYER_SMALL_VECTOR_SIZE;
pub use vm_state::{layers::LayerSelection, VmState};
use winit::keyboard::KeyCode;

use crate::{
    adv::assets::AdvAssets,
    audio::{BgmPlayer, SePlayer},
    layer::{
        render_layer_without_bg, render_params::TransformParams, user::UserLayer, AnyLayer,
        AnyLayerMut, Layer as _, LayerGroup, MessageLayer, PreRenderContext, RootLayerGroup,
        ScreenLayer,
    },
    render::overlay::{OverlayCollector, OverlayVisitable},
    update::{AdvUpdatable, AdvUpdateContext, Updatable, UpdateContext},
};

/// Actions available in all ADV contexts
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Enum)]
pub enum AdvMessageAction {
    Advance,
    HoldFastForward,
    Backlog,
    Rollback,
}

// impl Action for AdvMessageAction {
//     fn default_action_map() -> ActionMap<Self> {
//         fn map(v: AdvMessageAction) -> InputSet {
//             match v {
//                 AdvMessageAction::Advance => [
//                     MouseButton::Left.into(),
//                     KeyCode::Enter.into(),
//                     KeyCode::Space.into(),
//                 ]
//                 .into_iter()
//                 .collect(),
//                 AdvMessageAction::HoldFastForward => {
//                     [KeyCode::ControlLeft.into()].into_iter().collect()
//                 }
//                 AdvMessageAction::Backlog => [].into_iter().collect(),
//                 AdvMessageAction::Rollback => [].into_iter().collect(),
//             }
//         }
//
//         ActionMap::new(enum_map! { v => map(v) })
//     }
// }

pub struct Adv {
    scenario: Arc<Scenario>,
    scripter: Scripter,
    vm_state: VmState,
    adv_state: AdvState,
    // action_state: ActionState<AdvMessageAction>,
    current_command: Option<ExecutingCommand>,
    fast_forward_to_bp: Option<BreakpointObserver>,
}

impl Adv {
    pub fn new(
        audio_manager: Arc<AudioManager>,
        assets: AdvAssets,
        init_val: i32,
        random_seed: u32,
    ) -> Self {
        let scenario = assets.scenario.clone();
        let scripter = Scripter::new(&scenario, init_val, random_seed);
        let vm_state = VmState::new();
        let adv_state = AdvState::new(audio_manager, assets);

        Self {
            scenario,
            scripter,
            vm_state,
            adv_state,
            current_command: None,
            fast_forward_to_bp: None,
        }
    }

    pub fn fast_forward_to(&mut self, addr: CodeAddress) {
        assert!(self.fast_forward_to_bp.is_none());
        self.fast_forward_to_bp = Some(self.scripter.add_breakpoint(addr).into());
    }

    // TODO: impl Scene for Adv
    pub fn render(&self, pass: &mut RenderPass) {
        self.adv_state.render(pass);
    }
}

impl Updatable for Adv {
    fn update(&mut self, context: &mut UpdateContext) {
        // self.action_state.update(context.raw_input_state);

        let fast_forward_button_held = false;
        // self
        //     .action_state
        //     .is_pressed(AdvMessageAction::HoldFastForward);

        // if self.action_state.is_just_pressed(AdvMessageAction::Advance) {
        //     self.adv_state
        //         .root_layer_group
        //         .message_layer_mut()
        //         .advance();
        // }

        if fast_forward_button_held || self.fast_forward_to_bp.is_some() {
            self.adv_state
                .root_layer_group
                .message_layer_mut()
                .fast_forward();
        }

        let mut result = CommandResult::None;
        loop {
            // check the fast forward breakpoint; delete if hit
            if self
                .fast_forward_to_bp
                .as_mut()
                .map_or(false, |bp| bp.update())
            {
                debug!("Fast-forward breakpoint hit, stopping the FF");
                self.fast_forward_to_bp = None;
            }

            let is_fast_forwarding = fast_forward_button_held || self.fast_forward_to_bp.is_some();

            // TODO: maybe yield if spent too much time in this loop?
            let runtime_command = if let Some(command) = &mut self.current_command {
                match command.update(
                    context,
                    &self.scenario,
                    &self.vm_state,
                    &mut self.adv_state,
                    is_fast_forwarding,
                ) {
                    None => break,
                    Some(result) => {
                        self.current_command = None;
                        self.scripter.run(result).expect("scripter run failed")
                    }
                }
            } else {
                self.scripter.run(result).expect("scripter run failed")
            };

            runtime_command.apply_state(&mut self.vm_state);

            match runtime_command.start(
                context,
                &self.scenario,
                &self.vm_state,
                &mut self.adv_state,
            ) {
                CommandStartResult::Continue(r) => result = r,
                CommandStartResult::Yield(executing_command) => {
                    self.current_command = Some(executing_command);
                }
                CommandStartResult::Exit => {
                    todo!("adv exit");
                }
            }
        }

        self.adv_state.update(context);
    }
}

// impl Renderable for Adv {
//     fn render<'enc>(
//         &'enc self,
//         resources: &'enc GpuCommonResources,
//         render_pass: &mut wgpu::RenderPass<'enc>,
//         transform: Mat4,
//         projection: Mat4,
//     ) {
//         self.adv_state
//             .render(resources, render_pass, transform, projection);
//     }
//
//     fn resize(&mut self, resources: &GpuCommonResources) {
//         self.adv_state.resize(resources);
//     }
// }

// impl OverlayVisitable for Adv {
//     fn visit_overlay(&self, collector: &mut OverlayCollector) {
//         collector.subgroup(
//             "ADV",
//             |collector| {
//                 collector.overlay(
//                     "Command",
//                     |_ctx, top_left| {
//                         let command = if let Some(command) = &self.current_command {
//                             Cow::Owned(format!("{:?}", command))
//                         } else {
//                             Cow::Borrowed("None")
//                         };
//                         top_left.label(format!(
//                             "Command: {:08x} {}",
//                             self.scripter.position().0,
//                             command
//                         ));
//                     },
//                     true,
//                 );
//                 self.adv_state
//                     .root_layer_group
//                     .message_layer()
//                     .visit_overlay(collector);
//                 collector.overlay(
//                     "User Layers",
//                     |ctx, _top_left| {
//                         let page_layer =
//                             self.adv_state.root_layer_group.screen_layer().page_layer();
//                         Window::new("User Layers").show(ctx, |ui| {
//                             for plane in 0..PLANES_COUNT {
//                                 let layer_group = page_layer.plane(plane as u32);
//                                 let layer_ids =
//                                     layer_group.get_layer_ids().sorted().collect::<Vec<_>>();
//                                 if !layer_ids.is_empty() {
//                                     ui.monospace(format!("Plane {}:", plane));
//                                     for layer_id in layer_ids {
//                                         let layer = layer_group.get_layer(layer_id).unwrap();
//                                         ui.monospace(format!(
//                                             "  {:>2}: {:?}",
//                                             layer_id.raw(),
//                                             layer
//                                         ));
//                                     }
//                                 }
//                             }
//                         });
//                     },
//                     false,
//                 );
//             },
//             true,
//         );
//     }
// }

/// Contains all the state of the ADV system NOT pertaining to the VM
///
/// This is the object the VM manipulates
pub struct AdvState {
    pub root_layer_group: RootLayerGroup,
    pub audio_manager: Arc<AudioManager>,
    pub bgm_player: BgmPlayer,
    pub se_player: SePlayer,
    pub allow_running_animations: bool,
}

impl AdvState {
    pub fn new(audio_manager: Arc<AudioManager>, _assets: AdvAssets) -> Self {
        Self {
            root_layer_group: RootLayerGroup::new(),
            audio_manager: audio_manager.clone(),
            bgm_player: BgmPlayer::new(audio_manager.clone()),
            se_player: SePlayer::new(audio_manager),
            allow_running_animations: true,
        }
    }

    pub fn current_plane_layer_group(&self, vm_state: &VmState) -> &LayerGroup {
        self.root_layer_group
            .screen_layer()
            .page_layer()
            .get_plane(vm_state.layers.current_plane)
    }

    pub fn current_plane_layer_group_mut(&mut self, vm_state: &VmState) -> &mut LayerGroup {
        self.root_layer_group
            .screen_layer_mut()
            .page_layer_mut()
            .get_plane_mut(vm_state.layers.current_plane)
    }

    pub fn get_layer(&self, vm_state: &VmState, id: LayerId) -> Option<&UserLayer> {
        todo!()

        // let layer = self.current_plane_layer_group(vm_state).get_layer(id);
        // if let Some(layer) = layer {
        //     Some(layer)
        // } else {
        //     None
        // }
    }

    pub fn get_layer_mut(&mut self, vm_state: &VmState, id: LayerId) -> Option<&mut UserLayer> {
        todo!()

        // let layer = self
        //     .current_plane_layer_group_mut(vm_state)
        //     .get_layer_mut(id);
        // if let Some(layer) = layer {
        //     Some(layer)
        // } else {
        //     None
        // }
    }

    #[allow(unused)]
    pub fn get_vlayer(&self, vm_state: &VmState, id: VLayerId) -> impl Iterator<Item = AnyLayer> {
        // I could implement a special iterator for this, but it's not really worth it IMO
        // small vector will save A LOT of complexity
        let result: SmallVec<AnyLayer, { ITER_VLAYER_SMALL_VECTOR_SIZE }> = match id.repr() {
            VLayerIdRepr::RootLayerGroup => smallvec![(&self.root_layer_group).into()],
            VLayerIdRepr::ScreenLayer => smallvec![self.root_layer_group.screen_layer().into()],
            VLayerIdRepr::PageLayer => {
                smallvec![self.root_layer_group.screen_layer().page_layer().into()]
            }
            VLayerIdRepr::PlaneLayerGroup => {
                smallvec![self.current_plane_layer_group(vm_state).into()]
            }
            VLayerIdRepr::Selected => {
                if let Some(selection) = vm_state.layers.layer_selection {
                    todo!()
                    // self.current_plane_layer_group(vm_state)
                    //     .get_layers(selection)
                    //     .map(|v| v.into())
                    //     .collect::<SmallVec<AnyLayer, { ITER_VLAYER_SMALL_VECTOR_SIZE }>>()
                } else {
                    warn!("AdvState::iter_vlayer: no layer selected");
                    smallvec![]
                }
            }
            VLayerIdRepr::Layer(l) => {
                let layer = self.get_layer(vm_state, l);
                if let Some(layer) = layer {
                    smallvec![layer.into()]
                } else {
                    warn!("AdvState::iter_vlayer: layer not found: {:?}", l);
                    smallvec![]
                }
            }
        };

        result.into_iter()
    }

    pub fn get_vlayer_mut(
        &mut self,
        vm_state: &VmState,
        id: VLayerId,
    ) -> impl Iterator<Item = AnyLayerMut> {
        let result: SmallVec<AnyLayerMut, { ITER_VLAYER_SMALL_VECTOR_SIZE }> = match id.repr() {
            VLayerIdRepr::RootLayerGroup => smallvec![(&mut self.root_layer_group).into()],
            VLayerIdRepr::ScreenLayer => smallvec![self.root_layer_group.screen_layer_mut().into()],
            VLayerIdRepr::PageLayer => {
                smallvec![self
                    .root_layer_group
                    .screen_layer_mut()
                    .page_layer_mut()
                    .into()]
            }
            VLayerIdRepr::PlaneLayerGroup => {
                smallvec![self
                    .root_layer_group
                    .screen_layer_mut()
                    .page_layer_mut()
                    .get_plane_mut(vm_state.layers.current_plane)
                    .into()]
            }
            VLayerIdRepr::Selected => {
                if let Some(selection) = vm_state.layers.layer_selection {
                    todo!()
                    // self.current_plane_layer_group_mut(vm_state)
                    //     .get_layers_mut(selection)
                    //     .map(|v| v.into())
                    //     .collect::<SmallVec<AnyLayerMut, { ITER_VLAYER_SMALL_VECTOR_SIZE }>>()
                } else {
                    warn!("AdvState::get_vlayer_mut: no layer selected");
                    smallvec![]
                }
            }
            VLayerIdRepr::Layer(l) => {
                let layer = self.get_layer_mut(vm_state, l);
                if let Some(layer) = layer {
                    smallvec![layer.into()]
                } else {
                    warn!("AdvState::get_vlayer_mut: layer not found: {:?}", l);
                    smallvec![]
                }
            }
        };

        result.into_iter()
    }

    pub fn render(&self, pass: &mut RenderPass) {
        render_layer_without_bg(pass, &TransformParams::default(), &self.root_layer_group, 0)
    }
}

impl Updatable for AdvState {
    fn update(&mut self, context: &mut UpdateContext) {
        let adv_update_context = AdvUpdateContext {
            delta_time: context.delta_time,
            asset_server: context.asset_server,
            are_animations_allowed: self.allow_running_animations,
        };

        self.root_layer_group.update(&adv_update_context);

        let transform = TransformParams::default();

        self.root_layer_group
            .pre_render(context.pre_render, &transform);
    }
}
