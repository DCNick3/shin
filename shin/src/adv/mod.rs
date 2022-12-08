mod layers;

use crate::render::GpuCommonResources;
use crate::render::Renderable;
use crate::update::{Updatable, UpdateContext};
use crate::vm::{ExecutingCommand, UpdatableCommand, VmState};
use shin_core::format::scenario::Scenario;
use shin_core::vm::Scripter;
use std::sync::Arc;

pub struct Adv {
    scenario: Arc<Scenario>,
    scripter: Scripter,
    vm_state: VmState,
    adv_state: AdvState,
    current_command: Option<ExecutingCommand>,
}

impl Updatable for Adv {
    fn update(&mut self, context: &UpdateContext) {
        if let Some(command) = &mut self.current_command {
            command.update(context, &self.vm_state, &mut self.adv_state);
        }
        self.adv_state.update(context);
    }
}

impl Renderable for Adv {
    fn render<'enc>(
        &'enc self,
        resources: &'enc GpuCommonResources,
        render_pass: &mut wgpu::RenderPass<'enc>,
    ) {
        self.adv_state.render(resources, render_pass);
    }

    fn resize(&mut self, resources: &GpuCommonResources) {
        self.adv_state.resize(resources);
    }
}

pub struct AdvState {}

impl Updatable for AdvState {
    fn update(&mut self, _context: &UpdateContext) {
        todo!()
    }
}

impl Renderable for AdvState {
    fn render<'enc>(
        &'enc self,
        _resources: &'enc GpuCommonResources,
        _render_pass: &mut wgpu::RenderPass<'enc>,
    ) {
        todo!()
    }

    fn resize(&mut self, _resources: &GpuCommonResources) {
        todo!()
    }
}
