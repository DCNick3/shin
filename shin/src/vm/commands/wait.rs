use crate::vm::commands::CommandYield;
use crate::vm::Vm;
use crate::VmContinuation;
use bevy::prelude::*;
use shin_core::vm::command;
use std::time::Duration;

#[derive(Component)]
pub struct WAIT {
    token: Option<command::token::WAIT>,
    waiting_left: Duration,
}

impl super::Command<command::runtime::WAIT> for WAIT {
    type Result = CommandYield<WAIT>;

    fn start(command: command::runtime::WAIT, _vm: &mut Vm) -> Self::Result {
        assert_eq!(command.wait_kind, 0);
        CommandYield(Self {
            token: Some(command.token),
            waiting_left: Duration::from_millis(command.wait_amount as u64),
        })
    }
}

pub fn system(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut WAIT)>) {
    for (entity, mut wait) in query.iter_mut() {
        trace!("WAIT: {:?} {:?}", wait.waiting_left, time.delta());
        wait.waiting_left = wait.waiting_left.saturating_sub(time.delta());
        if wait.waiting_left <= Duration::ZERO {
            debug!(
                "WAIT: done; removing the WAIT component from {:?}, adding a continuation",
                entity
            );
            commands
                .entity(entity)
                .remove::<WAIT>()
                .insert(VmContinuation::new(wait.token.take().unwrap().finish()));
        }
    }
}
