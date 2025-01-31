use super::prelude::*;

impl StartableCommand for command::runtime::MSGINIT {
    type StateInfo = ();
    fn apply_state(&self, state: &mut VmState) {
        state.messagebox_state.msginit = self.messagebox_style;
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        _state_info: (),
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        self.token.finish().into()

        // adv_state
        //     .root_layer_group
        //     .message_layer_mut()
        //     .set_style(self.messagebox_style);
        // self.token.finish().into()
    }
}
