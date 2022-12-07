use crate::render::{RenderContext, Renderable};
use crate::update::{Updatable, UpdateContext};
use crate::vm::{ExecutingCommand, Vm};

pub struct Adv {
    vm: Vm,
    current_command: Option<ExecutingCommand>,
}

impl Updatable for Adv {
    fn update(&mut self, context: &UpdateContext) {
        todo!()
    }
}

impl Renderable for Adv {
    fn render<'a>(&'a self, context: &mut RenderContext<'a, '_>) {
        todo!()
    }

    fn resize(&mut self, size: (u32, u32)) {
        todo!()
    }
}
