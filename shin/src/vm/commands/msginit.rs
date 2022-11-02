use crate::vm::Vm;
use bevy::prelude::*;
use shin_core::vm::command;
use shin_core::vm::command::CommandResult;

#[derive(Component)]
pub struct MSGINIT;

impl super::Command<command::runtime::MSGINIT> for MSGINIT {
    type Result = CommandResult;

    fn start(command: command::runtime::MSGINIT, vm: &mut Vm) -> Self::Result {
        vm.state.msg_info.msginit = Some(command.arg);
        command.token.finish()
    }
}
