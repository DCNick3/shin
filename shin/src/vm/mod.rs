use bevy::prelude::*;

use crate::vm::commands::{Command, CommandStartResult};
use shin_core::format::scenario::Scenario;
use shin_core::vm::command::{CommandResult, RuntimeCommand};
use shin_core::vm::AdvVm;
use state::VmState;
use std::sync::Arc;

mod commands;
mod state;

#[derive(Component)]
pub struct Vm {
    scenario: Arc<Scenario>,
    vm: AdvVm,
    state: VmState,
}

impl Vm {
    pub fn new(scenario: Arc<Scenario>, init_val: i32, random_seed: u32) -> Self {
        Self {
            vm: AdvVm::new(&scenario, init_val, random_seed),
            state: VmState::new(),
            scenario,
        }
    }
}

#[derive(Component)]
pub struct VmContinuation {
    command_result: CommandResult,
}

impl VmContinuation {
    pub fn new(command_result: CommandResult) -> Self {
        Self { command_result }
    }
}

enum ExecuteCommandResult {
    Continue(CommandResult),
    Yield,
    Exit,
}

#[allow(clippy::unit_arg)]
fn execute_command(
    commands: &mut Commands,
    vm: &mut Vm,
    entity: Entity,
    command: RuntimeCommand,
) -> ExecuteCommandResult {
    match command {
        RuntimeCommand::EXIT(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::SGET(cmd) => commands::SGET::start(cmd, vm).apply_result(commands, entity),
        RuntimeCommand::SSET(cmd) => commands::SSET::start(cmd, vm).apply_result(commands, entity),
        RuntimeCommand::WAIT(cmd) => commands::WAIT::start(cmd, vm).apply_result(commands, entity),
        RuntimeCommand::MSGINIT(cmd) => {
            commands::MSGINIT::start(cmd, vm).apply_result(commands, entity)
        }
        RuntimeCommand::MSGSET(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::MSGWAIT(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::MSGSIGNAL(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::MSGSYNC(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::MSGCLOSE(cmd) => {
            commands::MSGCLOSE::start(cmd, vm).apply_result(commands, entity)
        }
        RuntimeCommand::SELECT(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::WIPE(cmd) => commands::WIPE::start(cmd, vm).apply_result(commands, entity),
        RuntimeCommand::WIPEWAIT(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::BGMPLAY(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::BGMSTOP(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::BGMVOL(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::BGMWAIT(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::BGMSYNC(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::SEPLAY(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::SESTOP(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::SESTOPALL(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::SEVOL(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::SEPAN(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::SEWAIT(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::SEONCE(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::VOICEPLAY(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::VOICESTOP(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::VOICEWAIT(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::SYSSE(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::SAVEINFO(cmd) => {
            commands::SAVEINFO::start(cmd, vm).apply_result(commands, entity)
        }
        RuntimeCommand::AUTOSAVE(cmd) => {
            commands::AUTOSAVE::start(cmd, vm).apply_result(commands, entity)
        }
        RuntimeCommand::EVBEGIN(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::EVEND(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::RESUMESET(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::RESUME(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::SYSCALL(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::TROPHY(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::UNLOCK(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::LAYERINIT(cmd) => {
            commands::LAYERINIT::start(cmd, vm).apply_result(commands, entity)
        }
        RuntimeCommand::LAYERLOAD(cmd) => {
            commands::LAYERLOAD::start(cmd, vm).apply_result(commands, entity)
        }
        RuntimeCommand::LAYERUNLOAD(cmd) => {
            commands::LAYERUNLOAD::start(cmd, vm).apply_result(commands, entity)
        }
        RuntimeCommand::LAYERCTRL(cmd) => {
            commands::LAYERCTRL::start(cmd, vm).apply_result(commands, entity)
        }
        RuntimeCommand::LAYERWAIT(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::LAYERSWAP(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::LAYERSELECT(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::MOVIEWAIT(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::TRANSSET(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::TRANSWAIT(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::PAGEBACK(cmd) => {
            commands::PAGEBACK::start(cmd, vm).apply_result(commands, entity)
        }
        RuntimeCommand::PLANESELECT(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::PLANECLEAR(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::MASKLOAD(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::MASKUNLOAD(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::CHARS(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::TIPSGET(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::QUIZ(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::SHOWCHARS(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::NOTIFYSET(cmd) => todo!("Execute command {:?}", cmd),
        RuntimeCommand::DEBUGOUT(cmd) => todo!("Execute command {:?}", cmd),
    }
}

fn adv_vm_system(mut commands: Commands, mut q: Query<(Entity, &mut Vm, &VmContinuation)>) {
    // let commands = Arc::new(RefCell::new(commands));

    for (entity, mut vm, cont) in &mut q {
        trace!("Updating a VM");

        commands.entity(entity).remove::<VmContinuation>();

        let mut command_result = cont.command_result.clone();

        loop {
            let command = vm.vm.run(command_result).expect("VM error");
            match execute_command(&mut commands, &mut vm, entity, command) {
                ExecuteCommandResult::Continue(new_command_result) => {
                    command_result = new_command_result
                }
                ExecuteCommandResult::Yield => break,
                ExecuteCommandResult::Exit => {
                    todo!("Exit the VM");
                }
            }
        }
    }
}

pub struct VmPlugin;

impl Plugin for VmPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(adv_vm_system)
            .add_plugin(commands::CommandsPlugin);
    }
}
