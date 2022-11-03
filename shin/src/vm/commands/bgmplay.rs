use crate::vm::Vm;
use bevy::prelude::*;
use shin_core::vm::command;
use shin_core::vm::command::CommandResult;

#[derive(Component)]
pub struct BGMPLAY;

impl super::Command<command::runtime::BGMPLAY> for BGMPLAY {
    type Result = CommandResult;

    fn start(command: command::runtime::BGMPLAY, vm: &mut Vm) -> Self::Result {
        warn!("TODO: BGMPLAY: {:?}", command);
        command.token.finish()
    }
}
