use super::prelude::*;
use crate::update::UpdateContext;
use shin_core::vm::command::time::Ticks;
use std::time::Duration;
use tracing::debug;

pub struct WAIT {
    token: Option<command::token::WAIT>,
    waiting_left: Duration,
}

impl super::StartableCommand for command::runtime::WAIT {
    fn apply_state(&self, _state: &mut VmState) {
        // nothing to do
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        _adv_state: &mut AdvState,
    ) -> CommandStartResult {
        assert_eq!(self.allow_interrupt, 0);
        Yield(
            WAIT {
                token: Some(self.token),
                waiting_left: Ticks(self.wait_amount as f32).as_duration(),
            }
            .into(),
        )
    }
}

impl UpdatableCommand for WAIT {
    fn update(
        &mut self,
        context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        _adv_state: &mut AdvState,
        is_fast_forwarding: bool,
    ) -> Option<CommandResult> {
        // trace!("WAIT: {:?} {:?}", self.waiting_left, context.time_delta());
        self.waiting_left = self.waiting_left.saturating_sub(context.time_delta());
        // TODO: short circuit the wait for now
        if self.waiting_left <= Duration::ZERO || is_fast_forwarding {
            debug!("WAIT: done");
            // TODO: this is kinda boilerplaty, maybe we want to have a TokenCell type?
            Some(self.token.take().unwrap().finish())
        } else {
            None
        }
    }
}
