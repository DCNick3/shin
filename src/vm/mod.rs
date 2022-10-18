use crate::format::scenario::instructions::{
    BinaryOperation, BinaryOperationType, CodeAddress, Command, Expression, ExpressionTerm,
    Instruction, JumpCond, JumpCondType, MemoryAddress, NumberSpec, UnaryOperation,
    UnaryOperationType,
};
use crate::format::scenario::{InstructionReader, Scenario};
use anyhow::Result;
use smallvec::SmallVec;
use tracing::{debug, instrument, trace, warn};

// TODO: add a listener trait that can be used to get notified of commands
pub struct AdvVm<'a> {
    scenario: &'a Scenario,
    memory: [i32; 0x1000],
    call_stack: Vec<CodeAddress>,
    data_stack: Vec<i32>,
    instruction_reader: InstructionReader<'a>,
}

impl<'a> AdvVm<'a> {
    pub fn new(scenario: &'a Scenario, init_val: i32) -> Self {
        let mut memory = [0; 0x1000];
        memory[0] = init_val;

        Self {
            scenario,
            memory,
            call_stack: Vec::new(),
            data_stack: vec![0; 0x16], // Umineko scenario writes out of bounds of the stack so we add some extra space
            instruction_reader: scenario.instruction_reader(scenario.entrypoint_address()),
        }
    }

    #[inline]
    fn get_memory(&self, addr: MemoryAddress) -> i32 {
        if let Some(offset) = addr.as_stack_offset() {
            self.data_stack[self.data_stack.len() - 1 - (offset + 1) as usize]
        } else {
            self.memory[addr.0 as usize]
        }
    }

    #[inline]
    fn set_memory(&mut self, addr: MemoryAddress, val: i32) {
        if let Some(offset) = addr.as_stack_offset() {
            let len = self.data_stack.len();
            // the top of the data stack is always the frame size
            // so we need to subtract 1 to get the actual top of the stack
            self.data_stack[len - 1 - (offset + 1) as usize] = val;
        } else {
            self.memory[addr.0 as usize] = val;
        }
    }

    #[inline]
    fn get_number(&self, number: NumberSpec) -> i32 {
        match number {
            NumberSpec::Constant(c) => c,
            NumberSpec::Memory(addr) => self.get_memory(addr),
        }
    }

    fn compute_jump_condition(&self, cond: JumpCond, left: i32, right: i32) -> bool {
        let result = match cond.condition {
            JumpCondType::Equal => left == right,
            JumpCondType::NotEqual => left != right,
            JumpCondType::GreaterOrEqual => left >= right,
            JumpCondType::Greater => left > right,
            JumpCondType::LessOrEqual => left <= right,
            JumpCondType::Less => left < right,
            JumpCondType::BitwiseAndNotZero => (left & right) != 0,
            JumpCondType::BitSet => todo!(),
        };

        if cond.is_negated {
            !result
        } else {
            result
        }
    }

    fn push_code_stack(&mut self, addr: CodeAddress) {
        self.call_stack.push(addr);
    }

    fn pop_code_stack(&mut self) -> CodeAddress {
        self.call_stack.pop().unwrap()
    }

    fn push_data_stack_frame(&mut self, val: &[i32]) {
        for &v in val.iter().rev() {
            self.data_stack.push(v);
        }
        self.data_stack.push(val.len() as i32);
    }

    fn pop_data_stack_frame(&mut self) {
        let count = self.data_stack.pop().unwrap() as usize;
        for _ in 0..count {
            self.data_stack.pop().unwrap();
        }
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> i32 {
        let mut stack = SmallVec::<[i32; 16]>::new();
        for term in expr.0.iter() {
            match term {
                &ExpressionTerm::Push(v) => stack.push(self.get_number(v)),
                ExpressionTerm::Add => {
                    let left = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    stack.push(left + right);
                }
                ExpressionTerm::Subtract => {
                    let left = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    stack.push(left - right);
                }
                ExpressionTerm::Multiply => {
                    let left = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    stack.push(left * right);
                }
                ExpressionTerm::Divide => {
                    let left = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    stack.push(if right != 0 { left / right } else { 0 });
                }
                ExpressionTerm::Remainder => {
                    let left = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    let div = if right != 0 { left / right } else { 0 };
                    stack.push(left - div * right);
                }
                ExpressionTerm::MultiplyReal => {
                    let left = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    assert!(left >= 0 && right >= 0); // not sure if this will behave correctly otherwise
                    stack.push(left * right / 1000);
                }
                ExpressionTerm::Min => {
                    let left = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    stack.push(left.min(right));
                }
                ExpressionTerm::Max => {
                    let left = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    stack.push(left.max(right));
                }
            }
        }
        if stack.len() != 1 {
            warn!("Expression did not evaluate to a single value");
        }

        stack.pop().unwrap()
    }

    /// Execute one instruction
    /// pc is the program counter before the instruction was read
    #[instrument(skip(self), level = "trace")]
    fn run_instruction(&mut self, instruction: Instruction, pc: CodeAddress) -> Result<()> {
        match instruction {
            Instruction::uo(UnaryOperation {
                ty,
                destination,
                source,
            }) => {
                let source = self.get_number(source);
                let result = match ty {
                    UnaryOperationType::Zero => 0,
                    UnaryOperationType::Negate => -source,
                    _ => todo!(),
                };

                trace!(?pc, ?ty, ?destination, ?source, ?result, "uo");

                self.set_memory(destination, result);
            }
            Instruction::bo(BinaryOperation {
                ty,
                left,
                right,
                destination,
            }) => {
                let left = self.get_number(left);
                let right = self.get_number(right);
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

                self.set_memory(destination, result);
            }

            Instruction::exp { dest, expr } => {
                let result = self.evaluate_expression(&expr);
                trace!(?pc, ?dest, ?result, ?expr, "exp");
                self.set_memory(dest, result);
            }
            Instruction::jc {
                cond,
                left,
                right,
                target,
            } => {
                let left = self.get_number(left);
                let right = self.get_number(right);
                let cond = self.compute_jump_condition(cond, left, right);

                trace!(?pc, ?cond, ?left, ?right, ?target, "jc");
                if cond {
                    self.instruction_reader.set_position(target);
                }
            }
            Instruction::j { target } => {
                trace!(?pc, ?target, "j");
                self.instruction_reader.set_position(target);
            }
            Instruction::jt { value, table } => {
                let value = self.get_number(value);

                trace!(?pc, ?value, table_len = ?table.0.len(), "jt");

                // if value < 0 {
                //     panic!("jump table command with negative value");
                // }
                if value >= 0 && value < table.0.len() as i32 {
                    self.instruction_reader
                        .set_position(table.0[value as usize]);
                }
            }
            Instruction::call { target, args } => {
                let args = args
                    .0
                    .into_iter()
                    .map(|v| self.get_number(v))
                    .collect::<SmallVec<[i32; 6]>>();
                trace!(?pc, ?target, ?args, "call");

                self.push_code_stack(self.instruction_reader.position());
                self.push_data_stack_frame(&args);
                self.instruction_reader.set_position(target);
            }
            Instruction::push { values } => {
                // unfortunately the game uses the call stack for both code addresses and sometimes data...
                // we just cast the data provided to CodeOffset and hope for the best
                // what could go wrong?
                let values = values
                    .0
                    .into_iter()
                    .map(|v| CodeAddress(self.get_number(v).try_into().unwrap()))
                    .collect::<SmallVec<[CodeAddress; 6]>>();
                trace!(?pc, ?values, "push");

                for value in values {
                    self.push_code_stack(value)
                }
            }
            Instruction::pop { dest } => {
                let values = (0..dest.0.len())
                    .map(|_| self.pop_code_stack().0.try_into().unwrap())
                    .collect::<SmallVec<[i32; 6]>>();
                trace!(?pc, ?values, "pop");

                for (dest, value) in dest.0.iter().zip(values) {
                    self.set_memory(*dest, value);
                }
            }
            Instruction::r#return {} => {
                self.pop_data_stack_frame();
                let target = self.pop_code_stack();
                trace!(?pc, ?target, "return");
                self.instruction_reader.set_position(target);
            }
            Instruction::Command(command) => {
                match command {
                    Command::DEBUGOUT { format, args } => {
                        let args = args
                            .0
                            .into_iter()
                            .map(|v| self.get_number(v))
                            .collect::<SmallVec<[i32; 6]>>();

                        debug!(?pc, ?format, ?args, "DEBUGOUT");
                    }
                    _ => {}
                }
                // TODO: commands a no-op for now (not actually accurate!)
            }
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let pc = self.instruction_reader.position();
            let instruction = self.instruction_reader.read()?;
            self.run_instruction(instruction, pc)?;
        }
    }
}
