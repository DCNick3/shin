use std::fmt::{Debug, Formatter};

use super::prelude::*;

pub struct MSGWAIT {
    token: Option<command::token::MSGWAIT>,
    signal_num: i32,
}

impl StartableCommand for command::runtime::MSGWAIT {
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
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        Yield(
            MSGWAIT {
                token: Some(self.token),
                signal_num: self.signal_num,
            }
            .into(),
        )
    }
}

impl UpdatableCommand for MSGWAIT {
    fn update(
        &mut self,
        _context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
        _is_fast_forwarding: bool,
    ) -> Option<CommandResult> {
        let message_layer = adv_state.root_layer_group.message_layer();

        if message_layer.is_waiting(self.signal_num) {
            None
        } else {
            Some(self.token.take().unwrap().finish())
        }
    }
}

impl Debug for MSGWAIT {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MSGWAIT").field(&self.signal_num).finish()
    }
}
