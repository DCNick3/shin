use super::prelude::*;
use std::time::Duration;

pub struct WAIT {
    token: Option<command::token::WAIT>,
    waiting_left: Duration,
}

impl super::Command<command::runtime::WAIT> for WAIT {
    type Result = CommandYield<WAIT>;

    fn apply_state(_command: &command::runtime::WAIT, _state: &mut VmState) {
        // nothing to do
    }

    fn start(command: command::runtime::WAIT, _vm: &mut Vm) -> Self::Result {
        assert_eq!(command.allow_interrupt, 0);
        CommandYield(Self {
            token: Some(command.token),
            waiting_left: Duration::from_millis(command.wait_amount as u64),
        })
    }
}

// TODO: WAIT
// pub fn system(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut WAIT)>) {
//     for (entity, mut wait) in query.iter_mut() {
//         trace!("WAIT: {:?} {:?}", wait.waiting_left, time.delta());
//         wait.waiting_left = wait.waiting_left.saturating_sub(time.delta());
//         if wait.waiting_left <= Duration::ZERO {
//             debug!(
//                 "WAIT: done; removing the WAIT component from {:?}, adding a continuation",
//                 entity
//             );
//             commands
//                 .entity(entity)
//                 .remove::<WAIT>()
//                 .insert(VmContinuation::new(wait.token.take().unwrap().finish()));
//         }
//     }
// }
