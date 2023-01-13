pub mod assets;
mod command;
mod vm_state;

pub use command::{CommandStartResult, ExecutingCommand, StartableCommand, UpdatableCommand};
use std::borrow::Cow;
pub use vm_state::VmState;

use crate::adv::assets::AdvAssets;
use crate::audio::{AudioManager, BgmPlayer, SePlayer};
use crate::input::actions::AdvMessageAction;
use crate::input::ActionState;
use crate::layer::{AnyLayer, AnyLayerMut, LayerGroup, MessageLayer, RootLayerGroup};
use crate::render::overlay::{OverlayCollector, OverlayVisitable};
use crate::render::{GpuCommonResources, Renderable};
use crate::update::{Updatable, UpdateContext};
use cgmath::Matrix4;
use egui::Window;
use shin_core::format::scenario::Scenario;
use shin_core::vm::command::layer::{VLayerId, VLayerIdRepr};
use shin_core::vm::command::CommandResult;
use shin_core::vm::Scripter;
use std::sync::Arc;
use tracing::warn;

pub struct Adv {
    scenario: Arc<Scenario>,
    scripter: Scripter,
    vm_state: VmState,
    adv_state: AdvState,
    action_state: ActionState<AdvMessageAction>,
    current_command: Option<ExecutingCommand>,
}

impl Adv {
    pub fn new(
        resources: &GpuCommonResources,
        audio_manager: Arc<AudioManager>,
        assets: AdvAssets,
        init_val: i32,
        random_seed: u32,
    ) -> Self {
        let scenario = assets.scenario.clone();
        let scripter = Scripter::new(&scenario, init_val, random_seed);
        let vm_state = VmState::new();
        let adv_state = AdvState::new(resources, audio_manager, assets);

        Self {
            scenario,
            scripter,
            vm_state,
            adv_state,
            action_state: ActionState::new(),
            current_command: None,
        }
    }
}

impl Updatable for Adv {
    fn update(&mut self, context: &UpdateContext) {
        self.action_state.update(context.raw_input_state);

        let is_fast_forwarding = self
            .action_state
            .is_pressed(AdvMessageAction::HoldFastForward);

        if self.action_state.is_just_pressed(AdvMessageAction::Advance) || is_fast_forwarding {
            self.adv_state
                .root_layer_group
                .message_layer_mut()
                .advance();
        }

        let mut result = CommandResult::None;
        loop {
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

impl Renderable for Adv {
    fn render<'enc>(
        &'enc self,
        resources: &'enc GpuCommonResources,
        render_pass: &mut wgpu::RenderPass<'enc>,
        transform: Matrix4<f32>,
    ) {
        self.adv_state.render(resources, render_pass, transform);
    }

    fn resize(&mut self, resources: &GpuCommonResources) {
        self.adv_state.resize(resources);
    }
}

impl OverlayVisitable for Adv {
    fn visit_overlay(&self, collector: &mut OverlayCollector) {
        collector.subgroup(
            "ADV",
            |collector| {
                collector.overlay(
                    "Command",
                    |_ctx, top_left| {
                        let command = if let Some(command) = &self.current_command {
                            Cow::Owned(format!("{:?}", command))
                        } else {
                            Cow::Borrowed("None")
                        };
                        top_left.label(format!("Command: {}", command));
                    },
                    true,
                );
                self.adv_state
                    .root_layer_group
                    .message_layer()
                    .visit_overlay(collector);
                collector.overlay(
                    "User Layers",
                    |ctx, _top_left| {
                        let mut layer_ids = self
                            .adv_state
                            .root_layer_group
                            .screen_layer()
                            .get_layer_ids()
                            .cloned()
                            .collect::<Vec<_>>();
                        layer_ids.sort();
                        Window::new("User Layers").show(ctx, |ui| {
                            for layer_id in layer_ids {
                                let layer_ty: &'static str = self
                                    .adv_state
                                    .root_layer_group
                                    .screen_layer()
                                    .get_layer(layer_id)
                                    .unwrap()
                                    .into();
                                ui.label(format!("{}: {}", layer_id.raw(), layer_ty));
                            }
                        });
                    },
                    false,
                );
            },
            true,
        );
    }
}

pub struct AdvState {
    pub root_layer_group: RootLayerGroup,
    pub bgm_player: BgmPlayer,
    pub se_player: SePlayer,
}

impl AdvState {
    pub fn new(
        resources: &GpuCommonResources,
        audio_manager: Arc<AudioManager>,
        assets: AdvAssets,
    ) -> Self {
        Self {
            root_layer_group: RootLayerGroup::new(
                resources,
                LayerGroup::new(resources),
                MessageLayer::new(resources, assets.fonts, assets.messagebox_textures),
            ),
            bgm_player: BgmPlayer::new(audio_manager.clone()),
            se_player: SePlayer::new(audio_manager),
        }
    }

    pub fn current_layer_group(&self, _vm_state: &VmState) -> &LayerGroup {
        self.root_layer_group.screen_layer()
    }

    pub fn current_layer_group_mut(&mut self, _vm_state: &VmState) -> &mut LayerGroup {
        self.root_layer_group.screen_layer_mut()
    }

    #[allow(unused)]
    pub fn iter_vlayer(&self, vm_state: &VmState, id: VLayerId) -> impl Iterator<Item = AnyLayer> {
        // TODO: we actually can do this without vectors
        match id.repr() {
            VLayerIdRepr::RootLayerGroup => vec![(&self.root_layer_group).into()],
            VLayerIdRepr::ScreenLayer => vec![self.root_layer_group.screen_layer().into()],
            VLayerIdRepr::PageLayer => {
                warn!("Returning ScreenLayer for PageLayer");
                vec![(&self.root_layer_group).into()]
            }
            VLayerIdRepr::PlaneLayerGroup => {
                warn!("Returning ScreenLayer for PlaneLayerGroup");
                vec![self.root_layer_group.screen_layer().into()]
            }
            VLayerIdRepr::Selected => {
                if let Some(selection) = vm_state.layers.layer_selection {
                    selection
                        .iter()
                        .filter_map(|id| {
                            if let Some(layer) = self.current_layer_group(vm_state).get_layer(id) {
                                Some(layer.into())
                            } else {
                                warn!("AdvState::iter_vlayer: Selected layer not found: {:?}", id);
                                None
                            }
                        })
                        .collect()
                } else {
                    warn!("AdvState::iter_vlayer: no layer selected");
                    vec![]
                }
            }
            VLayerIdRepr::Layer(l) => {
                let layer = self.current_layer_group(vm_state).get_layer(l);
                if let Some(layer) = layer {
                    vec![layer.into()]
                } else {
                    warn!("AdvState::iter_vlayer: layer not found: {:?}", l);
                    vec![]
                }
            }
        }
        .into_iter()
    }

    pub fn for_each_vlayer_mut(
        &mut self,
        vm_state: &VmState,
        id: VLayerId,
        mut f: impl FnMut(AnyLayerMut),
    ) {
        match id.repr() {
            VLayerIdRepr::RootLayerGroup => f((&mut self.root_layer_group).into()),
            VLayerIdRepr::ScreenLayer => f(self.root_layer_group.screen_layer_mut().into()),
            VLayerIdRepr::PageLayer => {
                warn!("Returning ScreenLayer for PageLayer");
                f((&mut self.root_layer_group).into())
            }
            VLayerIdRepr::PlaneLayerGroup => {
                warn!("Returning ScreenLayer for PlaneLayerGroup");
                f(self.root_layer_group.screen_layer_mut().into())
            }
            VLayerIdRepr::Selected => {
                if let Some(selection) = vm_state.layers.layer_selection {
                    for id in selection.iter() {
                        if let Some(layer) =
                            self.current_layer_group_mut(vm_state).get_layer_mut(id)
                        {
                            f(layer.into());
                        } else {
                            warn!(
                                "AdvState::for_each_vlayer_mut: Selected layer not found: {:?}",
                                id
                            );
                        }
                    }
                } else {
                    warn!("AdvState::for_each_vlayer_mut: no layer selected");
                }
            }
            VLayerIdRepr::Layer(l) => {
                let layer = self.current_layer_group_mut(vm_state).get_layer_mut(l);
                if let Some(layer) = layer {
                    f(layer.into());
                } else {
                    warn!("AdvState::for_each_vlayer_mut: layer not found: {:?}", l);
                }
            }
        }
    }
}

// TODO: this could be derived...
impl Updatable for AdvState {
    fn update(&mut self, context: &UpdateContext) {
        self.root_layer_group.update(context);
    }
}

impl Renderable for AdvState {
    fn render<'enc>(
        &'enc self,
        resources: &'enc GpuCommonResources,
        render_pass: &mut wgpu::RenderPass<'enc>,
        transform: Matrix4<f32>,
    ) {
        self.root_layer_group
            .render(resources, render_pass, transform);
    }

    fn resize(&mut self, resources: &GpuCommonResources) {
        self.root_layer_group.resize(resources);
    }
}
