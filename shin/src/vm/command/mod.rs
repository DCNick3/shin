#![allow(clippy::upper_case_acronyms)]

mod prelude {
    pub use crate::adv::AdvState;
    pub use crate::layer::Layer;
    pub use crate::update::Ticks;
    pub use crate::update::UpdateContext;
    pub use crate::vm::command::CommandStartResult;
    pub use crate::vm::VmState;
    pub use shin_core::format::scenario::Scenario;
    pub use shin_core::vm::command;
    pub use shin_core::vm::command::layer::VLayerIdRepr;
    pub use shin_core::vm::command::CommandResult;
    pub use tracing::warn;
    pub use CommandStartResult::Yield;
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
use shin_core::format::scenario::Scenario;

use shin_core::vm::command::{CommandResult, RuntimeCommand};

use crate::adv::AdvState;
use crate::update::UpdateContext;
use crate::vm::VmState;

#[enum_dispatch]
pub trait UpdatableCommand {
    // TODO: provide mutable access to Adv Scene state
    fn update(
        &mut self,
        _context: &UpdateContext,
        _scenario: &Scenario,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> Option<CommandResult>;
}

// all commands that yield to the game loop should have:
// - a type implementing UpdatableCommand
// - a enum variant for that type here
#[enum_dispatch(UpdatableCommand)]
pub enum ExecutingCommand {
    WAIT,
    MSGSET,
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
            // RuntimeCommand::BGMSYNC(v) => {}
            RuntimeCommand::SEPLAY(v) => v.apply_state(state),
            // RuntimeCommand::SESTOP(v) => {}
            RuntimeCommand::SESTOPALL(v) => v.apply_state(state),
            // RuntimeCommand::SEVOL(v) => {}
            // RuntimeCommand::SEPAN(v) => {}
            // RuntimeCommand::SEWAIT(v) => {}
            // RuntimeCommand::SEONCE(v) => {}
            // RuntimeCommand::VOICEPLAY(v) => {}
            // RuntimeCommand::VOICESTOP(v) => {}
            // RuntimeCommand::VOICEWAIT(v) => {}
            // RuntimeCommand::SYSSE(v) => {}
            RuntimeCommand::SAVEINFO(v) => v.apply_state(state),
            RuntimeCommand::AUTOSAVE(v) => v.apply_state(state),
            // RuntimeCommand::EVBEGIN(v) => {}
            // RuntimeCommand::EVEND(v) => {}
            // RuntimeCommand::RESUMESET(v) => {}
            // RuntimeCommand::RESUME(v) => {}
            // RuntimeCommand::SYSCALL(v) => {}
            // RuntimeCommand::TROPHY(v) => {}
            // RuntimeCommand::UNLOCK(v) => {}
            RuntimeCommand::LAYERINIT(v) => v.apply_state(state),
            RuntimeCommand::LAYERLOAD(v) => v.apply_state(state),
            RuntimeCommand::LAYERUNLOAD(v) => v.apply_state(state),
            RuntimeCommand::LAYERCTRL(v) => v.apply_state(state),
            // RuntimeCommand::LAYERWAIT(v) => {}
            // RuntimeCommand::LAYERSWAP(v) => {}
            // RuntimeCommand::LAYERSELECT(v) => {}
            // RuntimeCommand::MOVIEWAIT(v) => {}
            // RuntimeCommand::TRANSSET(v) => {}
            // RuntimeCommand::TRANSWAIT(v) => {}
            RuntimeCommand::PAGEBACK(v) => v.apply_state(state),
            // RuntimeCommand::PLANESELECT(v) => {}
            // RuntimeCommand::PLANECLEAR(v) => {}
            // RuntimeCommand::MASKLOAD(v) => {}
            // RuntimeCommand::MASKUNLOAD(v) => {}
            // RuntimeCommand::CHARS(v) => {}
            // RuntimeCommand::TIPSGET(v) => {}
            // RuntimeCommand::QUIZ(v) => {}
            // RuntimeCommand::SHOWCHARS(v) => {}
            // RuntimeCommand::NOTIFYSET(v) => {}
            // RuntimeCommand::DEBUGOUT(v) => {}
            _ => todo!(),
        }
    }

    fn start(
        self,
        context: &UpdateContext,
        scenario: &Scenario,
        vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        match self {
            // RuntimeCommand::EXIT(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::SGET(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::SSET(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::WAIT(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::MSGINIT(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::MSGSET(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::MSGWAIT(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::MSGSIGNAL(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::MSGSYNC(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::MSGCLOSE(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::SELECT(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::WIPE(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::WIPEWAIT(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::BGMPLAY(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::BGMSTOP(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::BGMVOL(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::BGMWAIT(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::BGMSYNC(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::SEPLAY(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::SESTOP(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::SESTOPALL(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::SEVOL(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::SEPAN(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::SEWAIT(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::SEONCE(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::VOICEPLAY(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::VOICESTOP(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::VOICEWAIT(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::SYSSE(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::SAVEINFO(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::AUTOSAVE(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::EVBEGIN(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::EVEND(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::RESUMESET(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::RESUME(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::SYSCALL(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::TROPHY(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::UNLOCK(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::LAYERINIT(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::LAYERLOAD(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::LAYERUNLOAD(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::LAYERCTRL(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::LAYERWAIT(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::LAYERSWAP(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::LAYERSELECT(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::MOVIEWAIT(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::TRANSSET(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::TRANSWAIT(v) => v.start(context, scenario, vm_state, adv_state),
            RuntimeCommand::PAGEBACK(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::PLANESELECT(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::PLANECLEAR(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::MASKLOAD(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::MASKUNLOAD(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::CHARS(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::TIPSGET(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::QUIZ(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::SHOWCHARS(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::NOTIFYSET(v) => v.start(context, scenario, vm_state, adv_state),
            // RuntimeCommand::DEBUGOUT(v) => v.start(context, scenario, vm_state, adv_state),
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
    fn start(
        self,
        context: &UpdateContext,
        scenario: &Scenario,
        vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> CommandStartResult;
}
