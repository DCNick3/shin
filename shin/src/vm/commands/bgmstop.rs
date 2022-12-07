use super::prelude::*;

pub struct BGMSTOP;

impl super::Command<command::runtime::BGMSTOP> for BGMSTOP {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::BGMSTOP, _state: &mut VmState) {
        warn!("TODO: BGMSTOP state: {:?}", command);
    }

    fn start(command: command::runtime::BGMSTOP, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: BGMSTOP: {:?}", command);
        command.token.finish()
    }
}
