#![allow(clippy::upper_case_acronyms)]

mod prelude {
    pub use crate::vm::commands::CommandYield;
    pub use crate::vm::state::VmState;
    pub use crate::vm::Vm;
    pub use shin_core::vm::command;
    pub use shin_core::vm::command::layer::VLayerIdRepr;
    pub use shin_core::vm::command::CommandResult;
    pub use tracing::warn;
}

mod autosave;
mod bgmplay;
mod bgmstop;
mod layerctrl;
mod layerinit;
mod layerload;
mod layerunload;
mod msgclose;
mod msginit;
mod msgset;
mod pageback;
mod saveinfo;
mod seplay;
mod sestopall;
mod sget;
mod sset;
mod wait;
mod wipe;

use crate::vm::ExecuteCommandResult;
use shin_core::vm::command::CommandResult;

pub use autosave::AUTOSAVE;
pub use bgmplay::BGMPLAY;
pub use bgmstop::BGMSTOP;
pub use layerctrl::LAYERCTRL;
pub use layerinit::LAYERINIT;
pub use layerload::LAYERLOAD;
pub use layerunload::LAYERUNLOAD;
pub use msgclose::MSGCLOSE;
pub use msginit::MSGINIT;
pub use msgset::MSGSET;
pub use pageback::PAGEBACK;
pub use saveinfo::SAVEINFO;
pub use seplay::SEPLAY;
pub use sestopall::SESTOPALL;
pub use sget::SGET;
pub use sset::SSET;
pub use wait::WAIT;
pub use wipe::WIPE;

pub trait CommandStartResult {
    fn apply_result(
        self,
        // commands: &mut bevy::ecs::system::Commands,
        // entity: bevy::ecs::entity::Entity,
    ) -> ExecuteCommandResult;
}

impl CommandStartResult for CommandResult {
    fn apply_result(self) -> ExecuteCommandResult {
        ExecuteCommandResult::Continue(self)
    }
}

// TODO: add trait bound for T
// smth like "can be executed in a game loop repeatedly"
pub struct CommandYield<T>(T);

impl<T> CommandStartResult for CommandYield<T> {
    fn apply_result(self) -> ExecuteCommandResult {
        // commands.entity(entity).insert(self.0);
        todo!("CommandYield")
        // ExecuteCommandResult::Yield
    }
}

pub struct CommandExit;
impl CommandStartResult for CommandExit {
    fn apply_result(self) -> ExecuteCommandResult {
        ExecuteCommandResult::Exit
    }
}
pub trait Command<T> {
    type Result: CommandStartResult;

    fn apply_state(command: &T, state: &mut crate::vm::VmState);
    fn start(command: T, vm: &mut crate::vm::Vm) -> Self::Result;
}
