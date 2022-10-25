use crate::vm::listener::ListenerCtx;
use crate::vm::VmImpl;
use shin_core::vm::command::{AdvCommand, CommandPoll};
use std::time::Duration;

pub struct Wait {
    waiting_left: Duration,
}

impl Wait {
    pub fn new(time: Duration) -> Self {
        Self { waiting_left: time }
    }
}

impl AdvCommand<VmImpl> for Wait {
    type Output = ();

    fn poll(&mut self, ctx: &ListenerCtx, _listener: &mut VmImpl) -> CommandPoll<Self::Output> {
        self.waiting_left = self.waiting_left.saturating_sub(ctx.time.delta());

        if self.waiting_left == Duration::from_secs(0) {
            CommandPoll::Ready(())
        } else {
            CommandPoll::Pending
        }
    }
}
