//! Defines the [`Instruction`] type, along with some helper types used for their encoding.

mod binary_operation;
mod expression;
mod jump_cond;
mod unary_operation;

use std::fmt::Debug;

use binrw::{BinRead, BinWrite};

pub use self::{
    binary_operation::{BinaryOperation, BinaryOperationType},
    expression::{Expression, ExpressionTerm},
    jump_cond::{JumpCond, JumpCondType},
    unary_operation::{UnaryOperation, UnaryOperationType},
};
use crate::{
    format::scenario::{
        instruction_elements::{CodeAddress, NumberSpec, Register},
        types::{Pad4, U16SmallList, U8SmallList, U8SmallNumberList},
    },
    vm::command::CompiletimeCommand,
};

// NOTE: theoretically, it might have sense to use the same macro as we do for Command to create separate runtime and compile-time instruction representations
// But I believe this not really necessary.
// First of all, there aren't a lot of instructions. It doesn't hurt that much to repeat the `IntoRuntimeForm` invokactions for some of types (numbers).
// Second, the types we would need to convert are not like the ones used in commands, and they are sometimes nasty (think about expressions for example). It's just easier to convert & execute them in one go.
// Finally, unlike commands, instruction don't have to live for a long time in a game loop, but are always executed immediately without yielding control to the game engine.

/// Represents an instruction read from a script.
#[allow(non_camel_case_types)]
#[derive(BinRead, BinWrite, PartialEq, Eq, Debug, Clone)]
#[brw(little)]
pub enum Instruction {
    /// Unary operation
    ///
    /// Loads one argument, computes a single result and stores the result at the destination address.
    #[brw(magic(0x40u8))]
    uo(UnaryOperation),
    /// Binary operation
    ///
    /// Loads two arguments, computes a single result and stores the result at the destination address.
    #[brw(magic(0x41u8))]
    bo(BinaryOperation),

    /// Complex expression
    ///
    /// This can load multiple arguments, compute a single result and store the result at the destination address.
    ///
    /// The expression itself is encoded as a reverse polish notation expression.
    #[brw(magic(0x42u8))]
    exp { dest: Register, expr: Expression },

    /// Get Table
    ///
    /// This selects a number from a table based on the value of the index and stores the result at the destination address.
    #[brw(magic(0x44u8))]
    gt {
        dest: Register,
        index: NumberSpec,
        table: U16SmallList<Pad4<NumberSpec>, 32>,
    },
    /// Jump Conditional
    ///
    /// Compares two numbers and jumps to a target address if the condition is true.
    #[brw(magic(0x46u8))]
    jc {
        cond: JumpCond,
        left: NumberSpec,
        right: NumberSpec,
        target: CodeAddress,
    },

    /// Jump Unconditional
    #[brw(magic(0x47u8))]
    j { target: CodeAddress },
    // ShinDataUtil is using names "call" and "return" for opcodes 0x48 and 0x49
    // while this is kinda true, there are instructions that are much more like "call" and "return"
    // I think I will rename these to gosub or smth, because they do not pass any parameters
    // (Higurashi does not use mem3 aka data stack at all, maybe because the script was converted)
    /// Call a Subroutine without Parameters (legacy call?)
    ///
    /// It appears that this is the older way of calling functions (before the introduction of [call](Instruction::call)).
    ///
    /// The umi scenario still uses this (a bit).
    ///
    /// NOTE: this is called `call` in ShinDataUtil.
    ///
    /// The return must be done with [retsub](Instruction::retsub).
    #[brw(magic(0x48u8))]
    gosub { target: CodeAddress },
    /// Return from a Subroutine called with [gosub](Instruction::gosub)
    ///
    /// NOTE: this is called `return` in ShinDataUtil.
    #[brw(magic(0x49u8))]
    retsub {},
    /// Jump via Table
    ///
    /// Jump to a target address based on the value of the index.
    #[brw(magic(0x4au8))]
    jt {
        index: NumberSpec,
        table: U16SmallList<CodeAddress, 32>,
    },
    // 0x4b not implemented
    /// Generate a random number between min and max (inclusive)
    #[brw(magic(0x4cu8))]
    rnd {
        dest: Register,
        min: NumberSpec,
        max: NumberSpec,
    },
    /// Push Values to call stack
    ///
    /// Used to preserve values of memory in the function. Must be restored with [pop](Instruction::pop) before using [return](`Instruction::return`) or [retsub](Instruction::retsub)
    #[brw(magic(0x4du8))]
    push { values: U8SmallNumberList },
    /// Pop Values from call stack
    ///
    /// Used to restore values of memory previously pushed by [push](Instruction::push)
    #[brw(magic(0x4eu8))]
    pop { dest: U8SmallList<Register> },
    /// Call Subroutine with Parameters
    ///
    /// The return must be done with [return](`Instruction::return`).
    #[brw(magic(0x4fu8))]
    call {
        target: CodeAddress,
        args: U8SmallNumberList,
    },
    /// Return from Subroutine called with [call](Instruction::call)
    #[brw(magic(0x50u8))]
    r#return {},

    /// Send command to the game engine
    Command(CompiletimeCommand),
}
