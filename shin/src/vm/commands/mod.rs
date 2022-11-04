#![allow(clippy::upper_case_acronyms)]

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
        commands: &mut bevy::ecs::system::Commands,
        entity: bevy::ecs::entity::Entity,
    ) -> ExecuteCommandResult;
}

impl CommandStartResult for CommandResult {
    fn apply_result(
        self,
        _commands: &mut bevy::ecs::system::Commands,
        _entity: bevy::ecs::entity::Entity,
    ) -> ExecuteCommandResult {
        ExecuteCommandResult::Continue(self)
    }
}

pub struct CommandYield<T: bevy::ecs::component::Component>(T);

impl<T: bevy::ecs::component::Component> CommandStartResult for CommandYield<T> {
    fn apply_result(
        self,
        commands: &mut bevy::ecs::system::Commands,
        entity: bevy::ecs::entity::Entity,
    ) -> ExecuteCommandResult {
        commands.entity(entity).insert(self.0);
        ExecuteCommandResult::Yield
    }
}

pub struct CommandExit;
impl CommandStartResult for CommandExit {
    fn apply_result(
        self,
        _commands: &mut bevy::ecs::system::Commands,
        _entity: bevy::ecs::entity::Entity,
    ) -> ExecuteCommandResult {
        ExecuteCommandResult::Exit
    }
}

pub trait Command<T> {
    type Result: CommandStartResult;
    fn start(command: T, vm: &mut crate::vm::Vm) -> Self::Result;
}

pub struct CommandsPlugin;

impl bevy::app::Plugin for CommandsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_system(wait::system).add_system(msgset::system);
    }
}
