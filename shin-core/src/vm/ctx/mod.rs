mod from_vm_ctx;

pub use from_vm_ctx::*;

use crate::format::scenario::instructions::{
    CodeAddress, Expression, ExpressionTerm, JumpCond, JumpCondType, MemoryAddress, NumberSpec,
};
use smallvec::SmallVec;
use tracing::warn;

pub struct VmCtx {
    /// Memory (aka registers I guess)
    memory: [i32; 0x1000],
    /// Call stack
    /// Stores the return address for each call instruction
    /// Also push instruction pushes here for some reason
    call_stack: Vec<CodeAddress>,
    /// Data stack
    /// Stores the arguments for each call instruction
    /// Can be addresses via MemoryAddress with addresses > 0x1000
    /// Also called mem3 in ShinDataUtil
    data_stack: Vec<i32>,
    /// PRNG state, updated on each instruction executed
    prng_state: u32,
}

impl VmCtx {
    pub fn new(init_val: i32, random_seed: u32) -> Self {
        let mut memory = [0; 0x1000];
        memory[0] = init_val;

        Self {
            memory,
            call_stack: Vec::new(),
            data_stack: vec![0; 0x16], // Umineko scenario writes out of bounds of the stack so we add some extra space
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
    pub fn get_memory(&self, addr: MemoryAddress) -> i32 {
        if let Some(offset) = addr.as_stack_offset() {
            self.data_stack[self.data_stack.len() - 1 - (offset) as usize]
        } else {
            self.memory[addr.0 as usize]
        }
    }

    /// Set a memory address to a value
    ///
    /// The address can be a stack offset (mem3) or main memory address (mem1)
    #[inline]
    pub fn set_memory(&mut self, addr: MemoryAddress, val: i32) {
        if let Some(offset) = addr.as_stack_offset() {
            let len = self.data_stack.len();
            // the top of the data stack is always the frame size
            // so we need to subtract 1 to get the actual top of the stack
            self.data_stack[len - 1 - (offset) as usize] = val;
        } else {
            self.memory[addr.0 as usize] = val;
        }
    }

    /// Read NumberSpec from memory (or return the constant value)
    #[inline]
    pub fn get_number(&self, number: NumberSpec) -> i32 {
        match number {
            NumberSpec::Constant(c) => c,
            NumberSpec::Memory(addr) => self.get_memory(addr),
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
            JumpCondType::BitSet => todo!(),
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
            self.data_stack.push(v);
        }
        self.data_stack.push(val.len() as i32);
    }

    pub fn pop_data_stack_frame(&mut self) {
        let count = self.data_stack.pop().unwrap() as usize;
        for _ in 0..count {
            self.data_stack.pop().unwrap();
        }
    }

    /// Evaluate a RPN expression in this context
    pub fn evaluate_expression(&self, expr: &Expression) -> i32 {
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

    /// Update the PRNG state
    /// This is called after each instruction is executed
    pub fn update_prng(&mut self) {
        self.prng_state = self.prng_state.wrapping_mul(0x343fd).wrapping_add(0x269ec3);
    }

    /// Generate a random number in the range [a, b]
    /// This is called by the `rand` instruction
    /// The PRNG state is **NOT** updated after this call
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
