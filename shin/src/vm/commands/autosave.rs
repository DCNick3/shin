use crate::vm::Vm;
use bevy::prelude::*;
use shin_core::vm::command;
use shin_core::vm::command::CommandResult;

#[derive(Component)]
pub struct AUTOSAVE;

impl super::Command<command::runtime::AUTOSAVE> for AUTOSAVE {
    type Result = CommandResult;

    fn start(command: command::runtime::AUTOSAVE, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: AUTOSAVE: {:?}", command);
        command.token.finish()
    }
}
