use crate::layer::{AnyLayer, AnyLayerMut, LayerGroup, MessageLayer, RootLayerGroup};
use crate::render::GpuCommonResources;
use crate::render::Renderable;
use crate::update::{Updatable, UpdateContext};
use crate::vm::{
    CommandStartResult, ExecutingCommand, StartableCommand, UpdatableCommand, VmState,
};
use cgmath::Matrix4;
use shin_core::format::scenario::Scenario;
use shin_core::vm::command::layer::{VLayerId, VLayerIdRepr};
use shin_core::vm::command::CommandResult;
use shin_core::vm::Scripter;
use tracing::warn;

pub struct Adv {
    scenario: Scenario,
    scripter: Scripter,
    vm_state: VmState,
    adv_state: AdvState,
    current_command: Option<ExecutingCommand>,
}

impl Adv {
    pub fn new(
        resources: &GpuCommonResources,
        scenario: Scenario,
        init_val: i32,
        random_seed: u32,
    ) -> Self {
        let scripter = Scripter::new(&scenario, init_val, random_seed);
        let vm_state = VmState::new();
        let adv_state = AdvState::new(resources);

        Self {
            scenario,
            scripter,
            vm_state,
            adv_state,
            current_command: None,
        }
    }
}

impl Updatable for Adv {
    fn update(&mut self, context: &UpdateContext) {
        let mut result = CommandResult::None;
        loop {
            // TODO: maybe yield if spent too much time in this loop?
            let runtime_command = if let Some(command) = &mut self.current_command {
                match command.update(context, &self.scenario, &self.vm_state, &mut self.adv_state) {
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

pub struct AdvState {
    pub root_layer_group: RootLayerGroup,
}

impl AdvState {
    pub fn new(resources: &GpuCommonResources) -> Self {
        Self {
            root_layer_group: RootLayerGroup::new(
                resources,
                LayerGroup::new(resources),
                MessageLayer::new(resources),
            ),
        }
    }

    pub fn current_layer_group_mut(&mut self, _vm_state: &VmState) -> &mut LayerGroup {
        self.root_layer_group.screen_layer_mut()
    }

    pub fn get_vlayer(&self, _vm_state: &VmState, _id: VLayerId) -> impl Iterator<Item = AnyLayer> {
        todo!() as std::iter::Once<_>
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
