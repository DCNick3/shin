use super::prelude::*;
use crate::update::{Ticks, UpdateContext};
use crate::vm::command::UpdatableCommand;
use std::time::Duration;
use tracing::{debug, trace};

pub struct WAIT {
    token: Option<command::token::WAIT>,
    waiting_left: Duration,
}

impl super::StartableCommand for command::runtime::WAIT {
    fn apply_state(&self, _state: &mut VmState) {
        // nothing to do
    }

    fn start(self, _vm: &mut Vm) -> CommandStartResult {
        assert_eq!(self.allow_interrupt, 0);
        CommandStartResult::Yield(
            WAIT {
                token: Some(self.token),
                waiting_left: Ticks(self.wait_amount as f32).as_duration(),
            }
            .into(),
        )
    }
}

impl UpdatableCommand for WAIT {
    fn update(&mut self, context: &UpdateContext) -> Option<CommandResult> {
        trace!("WAIT: {:?} {:?}", self.waiting_left, context.delta());
        self.waiting_left = self.waiting_left.saturating_sub(context.delta());
        if self.waiting_left <= Duration::ZERO {
            debug!("WAIT: done");
            // TODO: this is kinda boilerplaty, maybe we want to have a TokenCell type?
            Some(self.token.take().unwrap().finish())
        } else {
            None
        }
    }
}
