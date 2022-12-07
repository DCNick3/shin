use crate::render::{RenderContext, Renderable};
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
    fn render<'a>(&'a self, context: &mut RenderContext<'a, '_>) {
        self.adv_state.render(context);
    }

    fn resize(&mut self, size: (u32, u32)) {
        self.adv_state.resize(size);
    }
}

pub struct AdvState {}

impl Updatable for AdvState {
    fn update(&mut self, context: &UpdateContext) {
        todo!()
    }
}

impl Renderable for AdvState {
    fn render<'a>(&'a self, context: &mut RenderContext<'a, '_>) {
        todo!()
    }

    fn resize(&mut self, size: (u32, u32)) {
        todo!()
    }
}
