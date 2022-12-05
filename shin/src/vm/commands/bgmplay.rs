use super::prelude::*;

pub struct BGMPLAY;

impl super::Command<command::runtime::BGMPLAY> for BGMPLAY {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::BGMPLAY, state: &mut VmState) {
        todo!()
    }

    fn start(command: command::runtime::BGMPLAY, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: BGMPLAY: {:?}", command);
        command.token.finish()
    }
}
