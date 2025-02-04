use super::prelude::*;

impl StartableCommand for command::runtime::MSGSIGNAL {
    type StateInfo = ();
    fn apply_state(&self, _state: &mut VmState) {
        // nothing to do
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        _state_info: (),
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        adv_state.root_layer_group.message_layer_mut().send_sync();
        self.token.finish().into()
    }
}
