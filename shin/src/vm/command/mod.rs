#![allow(clippy::upper_case_acronyms)]

mod prelude {
    pub use crate::adv::AdvState;
    pub use crate::update::UpdateContext;
    pub use crate::vm::command::CommandStartResult;
    pub use crate::vm::VmState;
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

use msgset::MSGSET;
use wait::WAIT;

use enum_dispatch::enum_dispatch;

use shin_core::vm::command::{CommandResult, RuntimeCommand};

use crate::adv::AdvState;
use crate::update::UpdateContext;
use crate::vm::VmState;

#[enum_dispatch]
pub trait UpdatableCommand {
    // TODO: provide mutable access to Adv Scene state
    fn update(
        &mut self,
        context: &UpdateContext,
        vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> Option<CommandResult>;
}

// all commands that yield to the game loop should have:
// - a type implementing UpdatableCommand
// - a enum variant for that type here
#[enum_dispatch(UpdatableCommand)]
pub enum ExecutingCommand {
    WAIT(WAIT),
    MSGSET(MSGSET),
}

impl StartableCommand for RuntimeCommand {
    fn apply_state(&self, state: &mut VmState) {
        match self {
            // RuntimeCommand::EXIT(v) => v.apply_state(state),
            RuntimeCommand::SGET(v) => v.apply_state(state),
            RuntimeCommand::SSET(v) => v.apply_state(state),
            RuntimeCommand::WAIT(v) => v.apply_state(state),
            RuntimeCommand::MSGINIT(v) => v.apply_state(state),
            RuntimeCommand::MSGSET(v) => v.apply_state(state),
            // RuntimeCommand::MSGWAIT(v) => v.apply_state(state),
            // RuntimeCommand::MSGSIGNAL(v) => v.apply_state(state),
            // RuntimeCommand::MSGSYNC(v) => v.apply_state(state),
            RuntimeCommand::MSGCLOSE(v) => v.apply_state(state),
            // RuntimeCommand::SELECT(v) => v.apply_state(state),
            RuntimeCommand::WIPE(v) => v.apply_state(state),
            // RuntimeCommand::WIPEWAIT(v) => v.apply_state(state),
            RuntimeCommand::BGMPLAY(v) => v.apply_state(state),
            RuntimeCommand::BGMSTOP(v) => v.apply_state(state),
            // RuntimeCommand::BGMVOL(v) => v.apply_state(state),
            // RuntimeCommand::BGMWAIT(v) => v.apply_state(state),
            _ => todo!(),
            // RuntimeCommand::BGMSYNC(_) => {}
            // RuntimeCommand::SEPLAY(_) => {}
            // RuntimeCommand::SESTOP(_) => {}
            // RuntimeCommand::SESTOPALL(_) => {}
            // RuntimeCommand::SEVOL(_) => {}
            // RuntimeCommand::SEPAN(_) => {}
            // RuntimeCommand::SEWAIT(_) => {}
            // RuntimeCommand::SEONCE(_) => {}
            // RuntimeCommand::VOICEPLAY(_) => {}
            // RuntimeCommand::VOICESTOP(_) => {}
            // RuntimeCommand::VOICEWAIT(_) => {}
            // RuntimeCommand::SYSSE(_) => {}
            // RuntimeCommand::SAVEINFO(_) => {}
            // RuntimeCommand::AUTOSAVE(_) => {}
            // RuntimeCommand::EVBEGIN(_) => {}
            // RuntimeCommand::EVEND(_) => {}
            // RuntimeCommand::RESUMESET(_) => {}
            // RuntimeCommand::RESUME(_) => {}
            // RuntimeCommand::SYSCALL(_) => {}
            // RuntimeCommand::TROPHY(_) => {}
            // RuntimeCommand::UNLOCK(_) => {}
            // RuntimeCommand::LAYERINIT(_) => {}
            // RuntimeCommand::LAYERLOAD(_) => {}
            // RuntimeCommand::LAYERUNLOAD(_) => {}
            // RuntimeCommand::LAYERCTRL(_) => {}
            // RuntimeCommand::LAYERWAIT(_) => {}
            // RuntimeCommand::LAYERSWAP(_) => {}
            // RuntimeCommand::LAYERSELECT(_) => {}
            // RuntimeCommand::MOVIEWAIT(_) => {}
            // RuntimeCommand::TRANSSET(_) => {}
            // RuntimeCommand::TRANSWAIT(_) => {}
            // RuntimeCommand::PAGEBACK(_) => {}
            // RuntimeCommand::PLANESELECT(_) => {}
            // RuntimeCommand::PLANECLEAR(_) => {}
            // RuntimeCommand::MASKLOAD(_) => {}
            // RuntimeCommand::MASKUNLOAD(_) => {}
            // RuntimeCommand::CHARS(_) => {}
            // RuntimeCommand::TIPSGET(_) => {}
            // RuntimeCommand::QUIZ(_) => {}
            // RuntimeCommand::SHOWCHARS(_) => {}
            // RuntimeCommand::NOTIFYSET(_) => {}
            // RuntimeCommand::DEBUGOUT(_) => {}
        }
    }

    fn start(self, vm_state: &VmState, adv_state: &mut AdvState) -> CommandStartResult {
        match self {
            // RuntimeCommand::EXIT(v) => v.start(vm),
            RuntimeCommand::SGET(v) => v.start(vm_state, adv_state),
            _ => todo!(),
        }
    }
}

pub enum CommandStartResult {
    /// Continue VM execution
    Continue(CommandResult),
    /// Yield to the game loop, run the command to completion, execution continued with the result
    Yield(ExecutingCommand),
    Exit,
}

impl From<CommandResult> for CommandStartResult {
    fn from(result: CommandResult) -> Self {
        CommandStartResult::Continue(result)
    }
}

impl From<ExecutingCommand> for CommandStartResult {
    fn from(command: ExecutingCommand) -> Self {
        CommandStartResult::Yield(command)
    }
}

pub trait StartableCommand {
    fn apply_state(&self, state: &mut VmState);
    // TODO: this should have a constant access to the VmState, but mutable access to layers, music players, etc.
    fn start(self, vm_state: &VmState, adv_state: &mut AdvState) -> CommandStartResult;
}
