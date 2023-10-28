//! Lower hir into instruction types

mod block;
mod elements;
mod file;
mod instruction;
mod program;

mod from_hir;

#[cfg(test)]
mod test_utils;

pub use block::{lower_block, LoweredBlock};
pub use file::{lower_file, LoweredFile};
pub use from_hir::{
    CodeAddressCollector, FromHirExpr, HirDiagnosticCollector, HirDiagnosticCollectorWithBlock,
    HirDiagnosticCollectorWithFile,
};
pub use program::{lower_program, LoweredProgram};
