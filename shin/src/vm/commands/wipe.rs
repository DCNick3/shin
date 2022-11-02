use crate::vm::Vm;
use bevy::prelude::*;
use shin_core::vm::command;
use shin_core::vm::command::CommandResult;

#[derive(Component)]
pub struct WIPE;

impl super::Command<command::runtime::WIPE> for WIPE {
    type Result = CommandResult;

    fn start(command: command::runtime::WIPE, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: WIPE: {:?}", command);
        command.token.finish()
    }
}
