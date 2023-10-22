//! Lower hir into instruction types

mod block;
mod elements;
mod instruction;

pub use block::lower_block;

#[cfg(test)]
mod test_utils;
