//! This module implements the virtual machine that can run the scenario.
//!
//! # Execution environment
//!
//! The virtual machine is a register-based machine with a stack (two stacks, actually). The VM state is stored in the [`VmCtx`] struct.
//!
//! # Execution model
//!
//! The VM executes instructions from a scenario. They are represented as [`Instruction`] enum.
//!
//! There are instructions for integer arithmetic, control flow, and more.
//!
//! A special kind of instruction is the [`Instruction::Command`]. Those are not executed by the VM, but instead are passed to the game engine.
//!
//! Most commands do not have any feedback to the VM, except for [SGET](command::runtime::SGET), [SELECT](command::runtime::SELECT) and [QUIZ](command::runtime::QUIZ).
//!
//! # Usage
//!
//! The [`Scripter`] struct is the main entry point for the VM. It reads a scenario, executes the instructions and returns commands for engine to execute.
//!

pub mod breakpoint;
pub mod command;
mod ctx;

pub use ctx::*;

use crate::format::scenario::instructions::{
    BinaryOperation, CodeAddress, Instruction, UnaryOperation, UnaryOperationType,
};
use crate::format::scenario::{InstructionReader, Scenario};
use crate::vm::breakpoint::{BreakpointHandle, CodeBreakpointSet};
use crate::vm::command::{CommandResult, RuntimeCommand};
use anyhow::Result;
use smallvec::SmallVec;
use tracing::{instrument, trace};

// TODO: add a listener trait that can be used to get notified of commands
/// The scripter reads scenarios and issues commands.
/// Those are usually handled by the Adv scene in the game (but you can do other stuff if you want to).
///
/// Example:
///
/// ```
/// use shin_core::format::scenario::Scenario;
/// use shin_core::vm::Scripter;
/// use shin_core::vm::command::CommandResult;
///
/// let min_scenario = b"SNR \xd8\x00\x00\x00\x00\x00\x00\x00\x06\x00\x00\x00\x13\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xbc\x00\x00\x00X\x00\x00\x00`\x00\x00\x00h\x00\x00\x00p\x00\x00\x00x\x00\x00\x00\x80\x00\x00\x00\x88\x00\x00\x00\x90\x00\x00\x00\x94\x00\x00\x00\x98\x00\x00\x00\x9c\x00\x00\x00\xa4\x00\x00\x00\xa8\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00F\x02\xb0\x00\xc4\x00\x00\x00\xff\r\x00Hello world!\x00\x00\x00\x00\x00";
/// let scenario = bytes::Bytes::from_static(min_scenario);
/// let scenario = Scenario::new(scenario).unwrap();
///
/// let mut scripter = Scripter::new(&scenario, 0, 42);
///
/// // Execute the scenario
/// let mut prev_command_result = CommandResult::None;
/// loop {
///    let command = scripter.run(prev_command_result).unwrap();
///    println!("Command: {:?}", command);
///    // use execute_dummy() to "execute" the command without the game engine
///    if let Some(result) = command.execute_dummy() {
///        prev_command_result = result;
///    } else {
///        break;
///    }     
/// }
/// ```
pub struct Scripter {
    /// Vm execution context
    ctx: VmCtx,
    instruction_reader: InstructionReader,
    position: CodeAddress,
    breakpoints: CodeBreakpointSet,
}

impl Scripter {
    /// Create a scripter for the given scenario
    ///
    /// # Arguments
    ///
    /// * `scenario` - The scenario to run
    /// * `init_val` - The initial value of the memory cell at address 0, used to dispatch different episodes
    /// * `random_seed` - The initial value of the PRNG
    pub fn new(scenario: &Scenario, init_val: i32, random_seed: u32) -> Self {
        Self {
            ctx: VmCtx::new(init_val, random_seed),
            instruction_reader: scenario.instruction_reader(scenario.entrypoint_address()),
            position: scenario.entrypoint_address(),
            breakpoints: CodeBreakpointSet::new(),
        }
    }

    /// Execute one instruction
    /// pc is the program counter before the instruction was read
    #[instrument(skip(self), level = "trace")]
    #[inline]
    fn run_instruction(
        &mut self,
        instruction: Instruction,
        pc: CodeAddress,
    ) -> Option<RuntimeCommand> {
        self.ctx.update_prng();
        self.position = pc;

        match instruction {
            Instruction::uo(UnaryOperation {
                ty,
                destination,
                source,
            }) => {
                let source = self.ctx.get_number(source);
                let result = match ty {
                    UnaryOperationType::Zero => 0,
                    UnaryOperationType::Negate => -source,
                    _ => todo!(),
                };

                trace!(?pc, ?ty, ?destination, ?source, ?result, "uo");

                self.ctx.write_register(destination, result);
            }
            Instruction::bo(BinaryOperation {
                ty,
                left,
                right,
                destination,
            }) => {
                let left = self.ctx.get_number(left);
                let right = self.ctx.get_number(right);
                let result = self.ctx.evaluate_binary_operation(ty, left, right);

                trace!(?pc, ?ty, ?destination, ?left, ?right, ?result, "bo");

                self.ctx.write_register(destination, result);
            }

            Instruction::exp { dest, expr } => {
                let result = self.ctx.evaluate_expression(&expr);
                trace!(?pc, ?dest, ?result, ?expr, "exp");
                self.ctx.write_register(dest, result);
            }
            Instruction::gt { dest, index, table } => {
                let index = self.ctx.get_number(index);

                let result = if index >= 0 && index < table.0.len() as i32 {
                    self.ctx.get_number(table.0[index as usize].0)
                } else {
                    0
                };
                trace!(?pc, ?index, ?result, ?dest, table_len = ?table.0.len(), "gt");
                self.ctx.write_register(dest, result);
            }
            Instruction::jc {
                cond,
                left,
                right,
                target,
            } => {
                let left = self.ctx.get_number(left);
                let right = self.ctx.get_number(right);
                let cond = self.ctx.compute_jump_condition(cond, left, right);

                trace!(?pc, ?cond, ?left, ?right, ?target, "jc");
                if cond {
                    self.instruction_reader.set_position(target);
                }
            }
            Instruction::j { target } => {
                trace!(?pc, ?target, "j");
                self.instruction_reader.set_position(target);
            }
            Instruction::gosub { target } => {
                trace!(?pc, ?target, "gosub");
                self.ctx.push_code_stack(self.instruction_reader.position());
                self.instruction_reader.set_position(target);
            }
            Instruction::retsub {} => {
                let target = self.ctx.pop_code_stack();
                trace!(?pc, ?target, "retsub");
                self.instruction_reader.set_position(target);
            }
            Instruction::jt { index, table } => {
                let index = self.ctx.get_number(index);

                let target =
                    (index >= 0 && index < table.0.len() as i32).then(|| table.0[index as usize]);

                trace!(?pc, ?index, ?target, table_len = ?table.0.len(), "jt");
                if let Some(target) = target {
                    self.instruction_reader.set_position(target);
                }
            }
            Instruction::rnd { dest, min, max } => {
                let min = self.ctx.get_number(min);
                let max = self.ctx.get_number(max);
                let result = self.ctx.run_prng(min, max);
                trace!(?pc, ?dest, ?min, ?max, ?result, prng_state = ?self.ctx.get_prng_state(), "rnd");
                self.ctx.write_register(dest, result);
            }
            Instruction::call { target, args } => {
                let args = args
                    .0
                    .into_iter()
                    .map(|v| self.ctx.get_number(v))
                    .collect::<SmallVec<[i32; 6]>>();
                trace!(?pc, ?target, ?args, "call");

                self.ctx.push_code_stack(self.instruction_reader.position());
                self.ctx.push_data_stack_frame(&args);
                self.instruction_reader.set_position(target);
            }
            Instruction::push { values } => {
                // unfortunately the game uses the call stack for both code addresses and sometimes data...
                // we just cast the data provided to CodeOffset and hope for the best
                // what could go wrong?
                let values = values
                    .0
                    .into_iter()
                    .map(|v| CodeAddress(self.ctx.get_number(v).try_into().unwrap()))
                    .collect::<SmallVec<[CodeAddress; 6]>>();
                trace!(?pc, ?values, "push");

                for value in values {
                    self.ctx.push_code_stack(value)
                }
            }
            Instruction::pop { dest } => {
                let values = (0..dest.0.len())
                    .map(|_| self.ctx.pop_code_stack().0.try_into().unwrap())
                    .collect::<SmallVec<[i32; 6]>>();
                trace!(?pc, ?values, "pop");

                for (dest, value) in dest.0.iter().zip(values) {
                    self.ctx.write_register(*dest, value);
                }
            }
            Instruction::r#return {} => {
                self.ctx.pop_data_stack_frame();
                let target = self.ctx.pop_code_stack();
                trace!(?pc, ?target, "return");
                self.instruction_reader.set_position(target);
            }
            Instruction::Command(command) => {
                let command = RuntimeCommand::from_vm_ctx(&self.ctx, command);
                trace!(?pc, ?command, "command");
                return Some(command);
            }
        }

        None
    }

    /// Get the current position of the VM
    ///
    /// This is the address of the next instruction to be executed
    #[inline]
    pub fn position(&self) -> CodeAddress {
        self.position
    }

    /// Run the VM until a command is encountered
    ///
    /// You should pass the result of the previous command to this function (use `CommandResult::None` if the VM is just starting)
    #[inline]
    pub fn run(&mut self, prev_command_result: CommandResult) -> Result<RuntimeCommand> {
        match prev_command_result {
            CommandResult::None => {}
            CommandResult::WriteMemory(addr, value) => {
                self.ctx.write_register(addr, value);
            }
        }

        loop {
            let pc = self.instruction_reader.position();
            let instruction = self.instruction_reader.read()?;
            self.breakpoints.visit_address(pc);
            if let Some(command) = self.run_instruction(instruction, pc) {
                return Ok(command);
            }
        }
    }

    /// Install a breakpoint at the given code address
    pub fn add_breakpoint(&mut self, address: CodeAddress) -> BreakpointHandle {
        self.breakpoints.add_breakpoint(address)
    }
}
