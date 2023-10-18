mod from_vm_ctx;

pub use from_vm_ctx::*;

use crate::format::scenario::instruction_elements::Register;
use crate::format::scenario::instructions::{
    BinaryOperationType, CodeAddress, Expression, ExpressionTerm, JumpCond, JumpCondType,
    NumberSpec,
};
use smallvec::SmallVec;
use tracing::warn;

/// Contains the full VM state
///
/// It consists of a memory, two stacks (call and data)
pub struct VmCtx {
    /// Memory (aka registers I guess)
    memory: [i32; 0x1000],
    /// Call stack
    ///
    /// Stores the return address for each call instruction
    ///
    /// Also [push](super::Instruction::push) uses this stack for some reason
    call_stack: Vec<CodeAddress>,
    /// Data stack
    ///
    /// Stores the arguments for each call instruction
    ///
    /// Can be addressed via [Register] with addresses > 0x1000
    ///
    /// Also called mem3 in ShinDataUtil
    arguments_stack: Vec<i32>,
    /// PRNG state, updated on each instruction executed
    prng_state: u32,
}

#[inline]
fn bool(v: i32) -> bool {
    v != 0
}

#[inline]
fn unbool(v: bool) -> i32 {
    if v {
        -1
    } else {
        0
    }
}

#[inline]
fn real(v: i32) -> f32 {
    v as f32 / 1000.0
}

#[inline]
fn unreal(v: f32) -> i32 {
    (v * 1000.0) as i32
}

#[inline]
fn angle(v: i32) -> f32 {
    real(v) * std::f32::consts::PI * 2.0
}

#[inline]
fn unangle(v: f32) -> i32 {
    unreal(v / std::f32::consts::PI / 2.0)
}

impl VmCtx {
    pub fn new(init_val: i32, random_seed: u32) -> Self {
        let mut memory = [0; 0x1000];
        memory[0] = init_val;

        Self {
            memory,
            call_stack: Vec::new(),
            arguments_stack: vec![0; 0x16], // Umineko scenario writes out of bounds of the stack so we add some extra space
            prng_state: random_seed,
        }
    }

    pub(super) fn get_prng_state(&self) -> u32 {
        self.prng_state
    }

    /// Get the value from memory
    ///
    /// The address can be a stack offset (mem3) or main memory address (mem1)
    #[inline]
    pub fn get_memory(&self, addr: Register) -> i32 {
        if let Some(offset) = addr.as_stack_offset() {
            self.arguments_stack[self.arguments_stack.len() - 1 - (offset) as usize]
        } else {
            self.memory[addr.raw() as usize]
        }
    }

    /// Set a memory address to a value
    ///
    /// The address can be a stack offset (mem3) or main memory address (mem1)
    #[inline]
    pub fn set_memory(&mut self, addr: Register, val: i32) {
        if let Some(offset) = addr.as_stack_offset() {
            let len = self.arguments_stack.len();
            // the top of the data stack is always the frame size
            // so we need to subtract 1 to get the actual top of the stack
            self.arguments_stack[len - 1 - (offset) as usize] = val;
        } else {
            self.memory[addr.raw() as usize] = val;
        }
    }

    /// Read NumberSpec from memory (or return the constant value)
    #[inline]
    pub fn get_number(&self, number: NumberSpec) -> i32 {
        match number {
            NumberSpec::Constant(c) => c,
            NumberSpec::Register(addr) => self.get_memory(addr),
        }
    }

    /// Evaluate jump condition in this context
    pub fn compute_jump_condition(&self, cond: JumpCond, left: i32, right: i32) -> bool {
        let result = match cond.condition {
            JumpCondType::Equal => left == right,
            JumpCondType::NotEqual => left != right,
            JumpCondType::GreaterOrEqual => left >= right,
            JumpCondType::Greater => left > right,
            JumpCondType::LessOrEqual => left <= right,
            JumpCondType::Less => left < right,
            JumpCondType::BitwiseAndNotZero => (left & right) != 0,
            JumpCondType::BitSet => (left & (1 << (right % 32))) != 0,
        };

        if cond.is_negated {
            !result
        } else {
            result
        }
    }

    pub fn push_code_stack(&mut self, addr: CodeAddress) {
        self.call_stack.push(addr);
    }

    pub fn pop_code_stack(&mut self) -> CodeAddress {
        self.call_stack.pop().unwrap()
    }

    pub fn push_data_stack_frame(&mut self, val: &[i32]) {
        for &v in val.iter().rev() {
            self.arguments_stack.push(v);
        }
        self.arguments_stack.push(val.len() as i32);
    }

    pub fn pop_data_stack_frame(&mut self) {
        let count = self.arguments_stack.pop().unwrap() as usize;
        for _ in 0..count {
            self.arguments_stack.pop().unwrap();
        }
    }

    /// Evaluate a RPN expression in this context
    pub fn evaluate_expression(&self, expr: &Expression) -> i32 {
        let mut stack = SmallVec::<[i32; 16]>::new();
        for term in expr.0.iter() {
            match term {
                &ExpressionTerm::Push(v) => stack.push(self.get_number(v)),
                ExpressionTerm::Add => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(left + right);
                }
                ExpressionTerm::Subtract => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(left - right);
                }
                ExpressionTerm::Multiply => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(left * right);
                }
                ExpressionTerm::Divide => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(if right != 0 { left / right } else { 0 });
                }
                ExpressionTerm::Modulo => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    let div = if right != 0 { left / right } else { 0 };
                    stack.push(left - div * right);
                }
                ExpressionTerm::ShiftLeft => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(left << right);
                }
                ExpressionTerm::ShiftRight => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(left >> right);
                }
                ExpressionTerm::BitwiseAnd => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(left & right);
                }
                ExpressionTerm::BitwiseOr => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(left | right);
                }
                ExpressionTerm::BitwiseXor => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(left ^ right);
                }
                ExpressionTerm::Negate => {
                    let val = stack.pop().unwrap();
                    stack.push(-val);
                }
                ExpressionTerm::BitwiseNot => {
                    let val = stack.pop().unwrap();
                    stack.push(!val);
                }
                ExpressionTerm::Abs => {
                    let val = stack.pop().unwrap();
                    stack.push(val.abs());
                }
                ExpressionTerm::CmpEqual => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(unbool(left == right));
                }
                ExpressionTerm::CmpNotEqual => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(unbool(left != right));
                }
                ExpressionTerm::CmpGreater => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(unbool(left > right));
                }
                ExpressionTerm::CmpGreaterOrEqual => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(unbool(left >= right));
                }
                ExpressionTerm::CmpLessOrEqual => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(unbool(left <= right));
                }
                ExpressionTerm::CmpLess => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(unbool(left < right));
                }
                ExpressionTerm::CmpZero => {
                    let val = stack.pop().unwrap();
                    stack.push(unbool(val == 0));
                }
                ExpressionTerm::CmpNotZero => {
                    let val = stack.pop().unwrap();
                    stack.push(unbool(val != 0));
                }
                ExpressionTerm::LogicalAnd => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(unbool(bool(left) && bool(right)));
                }
                ExpressionTerm::LogicalOr => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(unbool(bool(left) || bool(right)));
                }
                ExpressionTerm::Select => {
                    let cond = stack.pop().unwrap();
                    let true_val = stack.pop().unwrap();
                    let false_val = stack.pop().unwrap();
                    stack.push(if bool(cond) { true_val } else { false_val });
                }
                ExpressionTerm::MultiplyReal => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    // TODO: figure out how negative values are handled
                    assert!(left >= 0 && right >= 0); // not sure if this will behave correctly otherwise
                    stack.push(left * right / 1000);
                }
                ExpressionTerm::DivideReal => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    assert!(left >= 0 && right >= 0); // not sure if this will behave correctly otherwise
                    stack.push(left * 1000 / right);
                }
                ExpressionTerm::Sin => {
                    let val = stack.pop().unwrap();
                    stack.push(unreal(angle(val).sin()));
                }
                ExpressionTerm::Cos => {
                    let val = stack.pop().unwrap();
                    stack.push(unreal(angle(val).cos()));
                }
                ExpressionTerm::Tan => {
                    let val = stack.pop().unwrap();
                    stack.push(unreal(angle(val).tan()));
                }
                ExpressionTerm::Min => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(left.min(right));
                }
                ExpressionTerm::Max => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(left.max(right));
                }
            }
        }
        if stack.len() != 1 {
            warn!("Expression did not evaluate to a single value");
        }

        stack.pop().unwrap()
    }

    pub fn evaluate_binary_operation(&self, ty: BinaryOperationType, left: i32, right: i32) -> i32 {
        match ty {
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
            BinaryOperationType::Modulo => {
                let div = if right != 0 { left / right } else { 0 };
                left - div * right
            }
            BinaryOperationType::BitwiseAnd => left & right,
            BinaryOperationType::BitwiseOr => left | right,
            BinaryOperationType::BitwiseXor => left ^ right,
            BinaryOperationType::LeftShift => left << (right % 32),
            BinaryOperationType::RightShift => left >> (right % 32),
            BinaryOperationType::MultiplyReal => unreal(real(left) * real(right)),
            BinaryOperationType::DivideReal => unreal(real(left) / real(right)),
            BinaryOperationType::ATan2 => unangle(f32::atan2(real(left), real(right))),
            BinaryOperationType::SetBit => left | (1 << (right % 32)),
            BinaryOperationType::ClearBit => left & !(1 << (right % 32)),
            BinaryOperationType::ACursedOperation => {
                // Defined as `ctz((0xffffffff << R) & L)`
                let r = right % 32;
                let l = left & (-1 << r);
                let l = if l == 0 { 32 } else { l };
                let l = l.trailing_zeros();
                l as i32
            }
        }
    }

    /// Update the PRNG state.
    /// This is called after each instruction is executed.
    pub fn update_prng(&mut self) {
        self.prng_state = self.prng_state.wrapping_mul(0x343fd).wrapping_add(0x269ec3);
    }

    /// Generate a random number in the range [a, b]
    ///
    /// This is called by the `rand` instruction
    ///
    /// The PRNG state is **NOT** updated after this call
    ///
    /// (it is updated by AdvVm via [Self::update_prng] after each the instruction is executed)
    pub fn run_prng(&self, a: i32, b: i32) -> i32 {
        let state = self.prng_state;

        if a == b {
            a
        } else {
            let useful_state = (state >> 8 & 0xffff) as i32;
            let interval_size = (b - a).abs() + 1;
            let lower_bound = a.min(b);

            let amplitude = (useful_state * interval_size) >> 0x10;

            lower_bound + amplitude
        }
    }
}
