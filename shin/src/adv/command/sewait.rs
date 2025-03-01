use std::fmt::{Debug, Formatter};

use shin_core::vm::command::types::AudioWaitStatus;
use tracing::trace;

use super::prelude::*;

pub struct SEWAIT {
    token: Option<command::token::SEWAIT>,
    slot: i32,
    unwanted_statuses: AudioWaitStatus,
}

impl StartableCommand for command::runtime::SEWAIT {
    type StateInfo = ();
    fn apply_state(&self, _state: &mut VmState) {
        // nothing to do
    }

    fn start(
        self,
        _context: &mut UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        _state_info: (),
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        Yield(
            SEWAIT {
                token: Some(self.token),
                slot: self.se_slot,
                unwanted_statuses: self.unwanted_statuses,
            }
            .into(),
        )
    }
}

impl UpdatableCommand for SEWAIT {
    fn update(
        &mut self,
        _context: &mut UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
        _is_fast_forwarding: bool,
    ) -> Option<CommandResult> {
        let status = adv_state.se_player.get_wait_status(self.slot);
        let finished = (status & self.unwanted_statuses).is_empty();

        trace!(status = ?status, finished = %finished, "polling SEWAIT");

        if finished {
            Some(self.token.take().unwrap().finish())
        } else {
            None
        }
    }
}

impl Debug for SEWAIT {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SEWAIT")
            .field(&self.slot)
            .field(&self.unwanted_statuses)
            .finish()
    }
}
