use super::prelude::*;

pub struct MSGCLOSE;

impl super::Command<command::runtime::MSGCLOSE> for MSGCLOSE {
    type Result = CommandResult;

    fn apply_state(command: &command::runtime::MSGCLOSE, state: &mut VmState) {
        todo!()
    }

    fn start(command: command::runtime::MSGCLOSE, _vm: &mut Vm) -> Self::Result {
        assert_eq!(command.wait_for_close, 0);
        command.token.finish()
    }
}
