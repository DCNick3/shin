use bevy::prelude::*;
use std::cell::RefCell;

use crate::vm::layer::LayerbankInfo;
use crate::vm::listener::ListenerCtx;
use shin_core::format::scenario::Scenario;
use shin_core::vm::command::CommandPoll;
use shin_core::vm::AdvVm;
use std::sync::Arc;

mod commands;
mod layer;
mod listener;

struct SaveInfo {
    pub info: [String; 4],
}

impl SaveInfo {
    pub fn set_save_info(&mut self, level: i32, info: &str) {
        assert!(
            (0..=4).contains(&level),
            "SaveInfo::set_save_info: level out of range"
        );

        self.info[level as usize] = info.to_string();
    }
}

struct VmState {
    pub save_info: SaveInfo,
    pub layerbank_info: LayerbankInfo,
    // TODO: store ADV globals somewhere (maybe use a bevy resource?)
}

impl VmState {
    pub fn new() -> Self {
        Self {
            save_info: SaveInfo {
                info: ["", "", "", ""].map(|v| v.to_string()),
            },
            layerbank_info: LayerbankInfo::new(),
        }
    }
}

struct VmImpl {
    state: VmState,
}

#[derive(Component)]
pub struct Vm {
    scenario: Arc<Scenario>,
    vm: AdvVm<VmImpl>,
}

impl Vm {
    pub fn new(scenario: Arc<Scenario>, init_val: i32, random_seed: u32) -> Self {
        Self {
            vm: AdvVm::new(
                &scenario,
                init_val,
                random_seed,
                VmImpl {
                    state: VmState::new(),
                },
            ),
            scenario,
        }
    }
}

pub fn adv_vm_system(commands: Commands, mut q: Query<&mut Vm>, time: Res<Time>) {
    let commands = Arc::new(RefCell::new(commands));

    for mut vm in &mut q {
        trace!("Updating a VM");

        let ctx = ListenerCtx {
            time: time.clone(),
            commands: commands.clone(),
        };

        match vm.vm.run(&ctx).expect("VM error") {
            CommandPoll::Ready(result) => {
                panic!("VM finished with exit code {}", result);
            }
            CommandPoll::Pending => {}
        }
    }
}

pub struct VmPlugin;

impl Plugin for VmPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(adv_vm_system);
    }
}
