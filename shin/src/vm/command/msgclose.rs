use super::prelude::*;

impl super::StartableCommand for command::runtime::MSGCLOSE {
    fn apply_state(&self, state: &mut VmState) {
        // TODO: how to mark the closed messagebox in the state>
        todo!()
    }

    fn start(self, _vm: &mut Vm) -> CommandStartResult {
        assert_eq!(self.wait_for_close, 0);
        self.token.finish().into()
    }
}
