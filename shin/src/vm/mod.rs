mod command;
mod state;

pub use command::{ExecutingCommand, UpdatableCommand};
pub use state::VmState;

// fn adv_vm_system(mut commands: Commands, mut q: Query<(Entity, &mut Vm, &VmContinuation)>) {
//     // let commands = Arc::new(RefCell::new(commands));
//
//     for (entity, mut vm, cont) in &mut q {
//         trace!("Updating a VM");
//
//         commands.entity(entity).remove::<VmContinuation>();
//
//         let mut command_result = cont.command_result.clone();
//
//         loop {
//             let command = vm.vm.run(command_result).expect("VM error");
//             match execute_command(&mut commands, &mut vm, entity, command) {
//                 ExecuteCommandResult::Continue(new_command_result) => {
//                     command_result = new_command_result
//                 }
//                 ExecuteCommandResult::Yield => break,
//                 ExecuteCommandResult::Exit => {
//                     todo!("Exit the VM");
//                 }
//             }
//         }
//     }
// }
//
// pub struct VmPlugin;
//
// impl Plugin for VmPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_system(adv_vm_system)
//             .add_plugin(commands::CommandsPlugin);
//     }
// }
