use crate::vm::Vm;
use bevy::prelude::*;
use shin_core::vm::command;
use shin_core::vm::command::CommandResult;

#[derive(Component)]
pub struct BGMSTOP;

impl super::Command<command::runtime::BGMSTOP> for BGMSTOP {
    type Result = CommandResult;

    fn start(command: command::runtime::BGMSTOP, vm: &mut Vm) -> Self::Result {
        warn!("TODO: BGMSTOP: {:?}", command);
        command.token.finish()
    }
}
