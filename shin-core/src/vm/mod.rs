pub mod command;
mod ctx;

pub use ctx::*;

use crate::format::scenario::instructions::{
    BinaryOperation, BinaryOperationType, CodeAddress, Instruction, UnaryOperation,
    UnaryOperationType,
};
use crate::format::scenario::{InstructionReader, Scenario};
use crate::vm::command::{CommandResult, RuntimeCommand};
use anyhow::Result;
use smallvec::SmallVec;
use tracing::{instrument, trace};

// TODO: add a listener trait that can be used to get notified of commands
pub struct AdvVm {
    /// Vm execution context
    ctx: VmCtx,
    instruction_reader: InstructionReader,
}

impl AdvVm {
    pub fn new(scenario: &Scenario, init_val: i32, random_seed: u32) -> Self {
        Self {
            ctx: VmCtx::new(init_val, random_seed),
            instruction_reader: scenario.instruction_reader(scenario.entrypoint_address()),
        }
    }

    /// Execute one instruction
    /// pc is the program counter before the instruction was read
    #[instrument(skip(self), level = "trace")]
    fn run_instruction(
        &mut self,
        instruction: Instruction,
        pc: CodeAddress,
    ) -> Option<RuntimeCommand> {
        self.ctx.update_prng();

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

                self.ctx.set_memory(destination, result);
            }
            Instruction::bo(BinaryOperation {
                ty,
                left,
                right,
                destination,
            }) => {
                let left = self.ctx.get_number(left);
                let right = self.ctx.get_number(right);
                let result = match ty {
                    BinaryOperationType::MovRight => right,
                    BinaryOperationType::Zero => 0,
                    BinaryOperationType::Add => left + right,
                    BinaryOperationType::Subtract => left - right,
                    BinaryOperationType::Multiply => left * right,
                    BinaryOperationType::Divide => {
                        if right != 0 {
                            left / right
                        } else {
                            0
                        }
                    }
                    BinaryOperationType::Remainder => {
                        let div = if right != 0 { left / right } else { 0 };
                        left - div * right
                    }
                    BinaryOperationType::BitwiseAnd => left & right,
                    BinaryOperationType::BitwiseOr => left | right,
                    BinaryOperationType::BitwiseXor => left ^ right,
                    BinaryOperationType::LeftShift => left << right,
                    BinaryOperationType::RightShift => left >> right,
                    BinaryOperationType::MultiplyReal => todo!(),
                    BinaryOperationType::DivideReal => todo!(),
                };

                trace!(?pc, ?ty, ?destination, ?left, ?right, ?result, "bo");

                self.ctx.set_memory(destination, result);
            }

            Instruction::exp { dest, expr } => {
                let result = self.ctx.evaluate_expression(&expr);
                trace!(?pc, ?dest, ?result, ?expr, "exp");
                self.ctx.set_memory(dest, result);
            }
            Instruction::gt { dest, value, table } => {
                let value = self.ctx.get_number(value);

                let result = if value >= 0 && value < table.0.len() as i32 {
                    self.ctx.get_number(table.0[value as usize])
                } else {
                    0
                };
                trace!(?pc, ?value, ?result, ?dest, table_len = ?table.0.len(), "gt");
                self.ctx.set_memory(dest, result);
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
            Instruction::jt { value, table } => {
                let value = self.ctx.get_number(value);

                trace!(?pc, ?value, table_len = ?table.0.len(), "jt");

                // if value < 0 {
                //     panic!("jump table command with negative value");
                // }
                if value >= 0 && value < table.0.len() as i32 {
                    self.instruction_reader
                        .set_position(table.0[value as usize]);
                }
            }
            Instruction::rnd { dest, min, max } => {
                let min = self.ctx.get_number(min);
                let max = self.ctx.get_number(max);
                let result = self.ctx.run_prng(min, max);
                trace!(?pc, ?dest, ?min, ?max, ?result, prng_state = ?self.ctx.get_prng_state(), "rnd");
                self.ctx.set_memory(dest, result);
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
                    self.ctx.set_memory(*dest, value);
                }
            }
            Instruction::r#return {} => {
                self.ctx.pop_data_stack_frame();
                let target = self.ctx.pop_code_stack();
                trace!(?pc, ?target, "return");
                self.instruction_reader.set_position(target);
            }
            Instruction::Command(command) => {
                trace!(?pc, ?command, "command");
                return Some(RuntimeCommand::from_vm_ctx(&self.ctx, command));
            }
        }

        None
    }

    pub fn run(&mut self, prev_command_result: CommandResult) -> Result<RuntimeCommand> {
        match prev_command_result {
            CommandResult::None => {}
            CommandResult::WriteMemory(addr, value) => {
                self.ctx.set_memory(addr, value);
            }
        }

        loop {
            let pc = self.instruction_reader.position();
            let instruction = self.instruction_reader.read()?;
            if let Some(command) = self.run_instruction(instruction, pc) {
                return Ok(command);
            }
        }
    }
}
