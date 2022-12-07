use super::prelude::*;

pub struct MSGSET {
    #[allow(unused)]
    token: Option<command::token::MSGSET>,
}

impl super::StartableCommand for command::runtime::MSGSET {
    fn apply_state(&self, state: &mut VmState) {
        todo!("Add MSGSET")
    }

    fn start(self, _vm: &mut Vm) -> CommandStartResult {
        warn!("TODO: MSGSET: {:?}", self);

        todo!("Make a enum variant in ExecutingCommand for MSGSET")
        // CommandStartResult::Yield(
        //     Self {
        //         token: Some(command.token),
        //     }
        //     .into(),
        // )
    }
}

// pub fn system(mut _commands: Commands, mut query: Query<(Entity, &mut MSGSET)>) {
//     for (_entity, mut _wait) in query.iter_mut() {
//         // TODO: here we do not finish the command, making the VM wait forever
//     }
// }
