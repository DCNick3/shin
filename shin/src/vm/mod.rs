use bevy::prelude::*;
use std::cell::RefCell;

use crate::vm::layer::LayerbankInfo;
use crate::vm::listener::ListenerCtx;
use shin_core::format::scenario::Scenario;
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

fn run_one_vm<'w, 's, 'vm>(commands: &'vm mut Commands<'w, 's>, vm: &'vm mut Vm, time: Time)
where
    'w: 'vm,
    's: 'vm,
{
    let Vm { vm, state, .. } = vm;

    let mut ctx = ListenerCtx {
        time,
        commands,
        vm_state: state,
    };

    match vm.run::<'vm>(&mut ctx).expect("VM error") {
        CommandPoll::Ready(result) => {
            panic!("VM finished with exit code {}", result);
        }
        CommandPoll::Pending => {}
    }

    todo!()
}

pub fn adv_vm_system(mut commands: Commands, mut q: Query<&mut Vm>, time: Res<Time>) {
    // let commands = Arc::new(RefCell::new(commands));

    for mut vm in &mut q {
        trace!("Updating a VM");

        run_one_vm(&mut commands, &mut vm, time.clone());

        // let commands = commands.clone();
    }
}

pub struct VmPlugin;

impl Plugin for VmPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(adv_vm_system);
    }
}
