use crate::vm::commands::CommandYield;
use crate::vm::Vm;
use bevy::prelude::*;
use shin_core::vm::command;
use std::time::Duration;

#[derive(Component)]
pub struct WAIT {
    waiting_left: Duration,
}

impl super::Command<command::runtime::WAIT> for WAIT {
    type Result = CommandYield<WAIT>;

    fn start(command: command::runtime::WAIT, _vm: &mut Vm) -> Self::Result {
        assert_eq!(command.wait_kind, 0);
        CommandYield(Self {
            waiting_left: Duration::from_millis(command.wait_amount as u64),
        })
    }
}

pub fn system(mut commands: Commands, time: Res<Time>, mut query: Query<(&mut WAIT,)>) {
    for (mut wait,) in query.iter_mut() {
        wait.waiting_left -= time.delta();
        if wait.waiting_left <= Duration::ZERO {
            todo!("Finish WAIT");
        }
    }
}
