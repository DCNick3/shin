use shin_core::{format::scenario::instructions::Instruction, vm::command::CompiletimeCommand};

use crate::compile::hir::lower::LowerResult;

pub trait IntoInstructionResult {
    fn into_instruction_result(self) -> LowerResult<Instruction>;
}

impl IntoInstructionResult for Instruction {
    #[inline]
    fn into_instruction_result(self) -> LowerResult<Instruction> {
        Ok(self)
    }
}

impl IntoInstructionResult for LowerResult<Instruction> {
    #[inline]
    fn into_instruction_result(self) -> LowerResult<Instruction> {
        self
    }
}

impl IntoInstructionResult for CompiletimeCommand {
    #[inline]
    fn into_instruction_result(self) -> LowerResult<Instruction> {
        Ok(Instruction::Command(self))
    }
}

impl IntoInstructionResult for LowerResult<CompiletimeCommand> {
    #[inline]
    fn into_instruction_result(self) -> LowerResult<Instruction> {
        self.map(Instruction::Command)
    }
}
