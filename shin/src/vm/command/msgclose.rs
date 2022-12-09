use super::prelude::*;

impl super::StartableCommand for command::runtime::MSGCLOSE {
    fn apply_state(&self, _state: &mut VmState) {
        // TODO: how to mark the closed messagebox in the state>
        todo!()
    }

    fn start(
        self,
        _context: &UpdateContext,
        _vm_state: &VmState,
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        assert_eq!(self.wait_for_close, 0);
        self.token.finish().into()
    }
}
