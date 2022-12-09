use crate::layer::LayerGroup;
use crate::render::GpuCommonResources;
use crate::render::Renderable;
use crate::update::{Updatable, UpdateContext};
use crate::vm::{
    CommandStartResult, ExecutingCommand, StartableCommand, UpdatableCommand, VmState,
};
use cgmath::Matrix4;
use shin_core::format::scenario::Scenario;
use shin_core::vm::command::CommandResult;
use shin_core::vm::Scripter;

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
                match command.update(context, &self.vm_state, &mut self.adv_state) {
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

            match runtime_command.start(context, &self.vm_state, &mut self.adv_state) {
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
    pub root_layer_group: LayerGroup,
}

impl AdvState {
    pub fn new(resources: &GpuCommonResources) -> Self {
        Self {
            root_layer_group: LayerGroup::new(resources),
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
