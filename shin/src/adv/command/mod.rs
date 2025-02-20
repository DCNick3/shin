#![allow(clippy::upper_case_acronyms)]

mod prelude {
    pub use std::sync::Arc;

    pub use shin_core::{
        format::scenario::Scenario,
        time::Ticks,
        vm::{
            command,
            command::{
                types::{VLayerId, VLayerIdRepr},
                CommandResult,
            },
        },
    };
    pub use tracing::warn;
    pub use CommandStartResult::Yield;

    pub use crate::{
        adv::{AdvState, CommandStartResult, StartableCommand, UpdatableCommand, VmState},
        layer::DrawableLayer,
        update::UpdateContext,
    };
}

mod autosave;
mod bgmplay;
mod bgmstop;
mod bgmvol;
mod chars;
mod debugout;
mod evbegin;
mod evend;
mod layerctrl;
mod layerinit;
mod layerload;
mod layerselect;
mod layerunload;
mod layerwait;
mod moviewait;
mod msgclose;
mod msginit;
mod msgset;
mod msgsignal;
mod msgwait;
mod notifyset;
mod pageback;
mod planeclear;
mod planeselect;
mod saveinfo;
mod sepan;
mod seplay;
mod sestop;
mod sestopall;
mod sevol;
mod sewait;
mod sget;
mod showchars;
mod sset;
mod tipsget;
mod trophy;
mod unlock;
mod voiceplay;
mod wait;
mod wipe;

use std::sync::Arc;

use derivative::Derivative;
use enum_dispatch::enum_dispatch;
use shin_core::{
    format::scenario::Scenario,
    vm::command::{CommandResult, RuntimeCommand},
};

use self::{
    layerload::LAYERLOAD, layerwait::LAYERWAIT, moviewait::MOVIEWAIT, msgset::MSGSET,
    msgwait::MSGWAIT, sewait::SEWAIT, wait::WAIT, wipe::WIPE,
};
use crate::{
    adv::{AdvState, VmState},
    update::UpdateContext,
};

#[enum_dispatch]
pub trait UpdatableCommand {
    // TODO: provide mutable access to Adv Scene state
    fn update(
        &mut self,
        _context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        adv_state: &mut AdvState,
        is_fast_forwarding: bool,
    ) -> Option<CommandResult>;
}

// all commands that yield to the game loop should have:
// - a type implementing UpdatableCommand
// - a enum variant for that type here
#[enum_dispatch(UpdatableCommand)]
#[derive(Derivative)]
#[derivative(Debug)]
pub enum ExecutingCommand {
    #[derivative(Debug = "transparent")]
    WAIT,
    #[derivative(Debug = "transparent")]
    MSGSET,
    #[derivative(Debug = "transparent")]
    MSGWAIT,
    #[derivative(Debug = "transparent")]
    LAYERLOAD,
    #[derivative(Debug = "transparent")]
    LAYERWAIT,
    #[derivative(Debug = "transparent")]
    SEWAIT,
    #[derivative(Debug = "transparent")]
    MOVIEWAIT,
    #[derivative(Debug = "transparent")]
    WIPE,
}

pub fn apply_command_state(command: RuntimeCommand, state: &mut VmState) {
    macro_rules! impl_apply_state {
        ($($cmd:ident),*) => {
            match command {
                $(RuntimeCommand::$cmd(v) => {v.apply_state(state);},)*
                _ => todo!(),
            }
        };
    }

    impl_apply_state!(
        // EXIT,
        SGET,
        SSET,
        WAIT,
        MSGINIT,
        MSGSET,
        MSGWAIT,
        MSGSIGNAL,
        // MSGSYNC,
        MSGCLOSE,
        // SELECT,
        WIPE,
        // WIPEWAIT,
        BGMPLAY,
        BGMSTOP,
        BGMVOL,
        // BGMWAIT,
        // BGMSYNC,
        SEPLAY,
        SESTOP,
        SESTOPALL,
        SEVOL,
        SEPAN,
        SEWAIT,
        // SEONCE,
        VOICEPLAY,
        // VOICESTOP,
        // VOICEWAIT,
        // SYSSE,
        SAVEINFO,
        AUTOSAVE,
        EVBEGIN,
        EVEND,
        // RESUMESET,
        // RESUME,
        // SYSCALL,
        TROPHY,
        UNLOCK,
        LAYERINIT,
        LAYERLOAD,
        LAYERUNLOAD,
        LAYERCTRL,
        LAYERWAIT,
        // LAYERSWAP,
        LAYERSELECT,
        MOVIEWAIT,
        // TRANSSET,
        // TRANSWAIT,
        PAGEBACK,
        PLANESELECT,
        PLANECLEAR,
        // MASKLOAD,
        // MASKUNLOAD,
        CHARS,
        TIPSGET,
        // QUIZ,
        SHOWCHARS,
        NOTIFYSET,
        DEBUGOUT
    );
}

pub fn apply_command_state_and_start(
    command: RuntimeCommand,
    context: &UpdateContext,
    scenario: &Arc<Scenario>,
    vm_state: &mut VmState,
    adv_state: &mut AdvState,
) -> CommandStartResult {
    macro_rules! impl_apply_state {
        ($($cmd:ident),*) => {
            match command {
                $(RuntimeCommand::$cmd(v) => {let info = v.apply_state(vm_state); v.start(context, scenario, vm_state, info, adv_state)},)*
                _ => todo!(),
            }
        };
        () => {};
    }

    impl_apply_state!(
        // EXIT,
        SGET,
        SSET,
        WAIT,
        MSGINIT,
        MSGSET,
        MSGWAIT,
        MSGSIGNAL,
        // MSGSYNC,
        MSGCLOSE,
        // SELECT,
        WIPE,
        // WIPEWAIT,
        BGMPLAY,
        BGMSTOP,
        BGMVOL,
        // BGMWAIT,
        // BGMSYNC,
        SEPLAY,
        SESTOP,
        SESTOPALL,
        SEVOL,
        SEPAN,
        SEWAIT,
        // SEONCE,
        VOICEPLAY,
        // VOICESTOP,
        // VOICEWAIT,
        // SYSSE,
        SAVEINFO,
        AUTOSAVE,
        EVBEGIN,
        EVEND,
        // RESUMESET,
        // RESUME,
        // SYSCALL,
        TROPHY,
        UNLOCK,
        LAYERINIT,
        LAYERLOAD,
        LAYERUNLOAD,
        LAYERCTRL,
        LAYERWAIT,
        // LAYERSWAP,
        LAYERSELECT,
        MOVIEWAIT,
        // TRANSSET,
        // TRANSWAIT,
        PAGEBACK,
        PLANESELECT,
        PLANECLEAR,
        // MASKLOAD,
        // MASKUNLOAD,
        CHARS,
        TIPSGET,
        // QUIZ,
        SHOWCHARS,
        NOTIFYSET,
        DEBUGOUT
    )
}

pub enum CommandStartResult {
    /// Continue VM execution
    Continue(CommandResult),
    /// Yield to the game loop, run the command to completion, execution continued with the result
    Yield(ExecutingCommand),
    #[allow(unused)] // TODO: it will be used for implementing the "EXIT" command
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
    /// Type of the information the commands needs smuggled from the `apply_state` to the `start` method
    ///
    /// When doing a command replay for scene loading, this information is ignored
    ///
    /// This usually corresponds to non-param data set in the command constructor
    // TODO: for scene loading, this info can actually be paseed to the `RunState` function (or whatever it would be)
    type StateInfo;

    fn apply_state(&self, state: &mut VmState) -> Self::StateInfo;
    fn start(
        self,
        context: &UpdateContext,
        scenario: &Arc<Scenario>,
        vm_state: &VmState,
        state_info: Self::StateInfo,
        adv_state: &mut AdvState,
    ) -> CommandStartResult;
}
