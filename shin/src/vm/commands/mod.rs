#![allow(clippy::upper_case_acronyms)]

mod autosave;
mod layerinit;
mod msginit;
mod saveinfo;
mod sget;
mod sset;
mod wait;

use crate::vm::ExecuteCommandResult;
use shin_core::vm::command::CommandResult;

pub use autosave::AUTOSAVE;
pub use layerinit::LAYERINIT;
pub use msginit::MSGINIT;
pub use saveinfo::SAVEINFO;
pub use sget::SGET;
pub use sset::SSET;
pub use wait::WAIT;

pub trait CommandStartResult {
    fn apply_result(self, commands: &mut bevy::ecs::system::Commands) -> ExecuteCommandResult;
}

impl CommandStartResult for CommandResult {
    fn apply_result(self, _commands: &mut bevy::ecs::system::Commands) -> ExecuteCommandResult {
        ExecuteCommandResult::Continue(self)
    }
}

pub struct CommandYield<T: bevy::ecs::component::Component>(T);

impl<T: bevy::ecs::component::Component> CommandStartResult for CommandYield<T> {
    fn apply_result(self, commands: &mut bevy::ecs::system::Commands) -> ExecuteCommandResult {
        commands.spawn().insert(self.0);
        ExecuteCommandResult::Yield
    }
}

pub struct CommandExit;
impl CommandStartResult for CommandExit {
    fn apply_result(self, _commands: &mut bevy::ecs::system::Commands) -> ExecuteCommandResult {
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
        app.add_system(wait::system);
    }
}
