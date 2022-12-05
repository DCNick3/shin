use crate::vm::commands::CommandYield;
use crate::vm::Vm;
use shin_core::vm::command;
use tracing::warn;

pub struct MSGSET {
    #[allow(unused)]
    token: Option<command::token::MSGSET>,
}

impl super::Command<command::runtime::MSGSET> for MSGSET {
    type Result = CommandYield<MSGSET>;

    fn start(command: command::runtime::MSGSET, _vm: &mut Vm) -> Self::Result {
        warn!("TODO: MSGSET: {:?}", command);
        CommandYield(Self {
            token: Some(command.token),
        })
    }
}

// pub fn system(mut _commands: Commands, mut query: Query<(Entity, &mut MSGSET)>) {
//     for (_entity, mut _wait) in query.iter_mut() {
//         // TODO: here we do not finish the command, making the VM wait forever
//     }
// }
