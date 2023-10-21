//! An assembler for shin scenarios.
//!
//! The parser's design is heavily inspired by rust-analyzer's parser.
//!
//! The later stages of compilation diverge more, but are also somewhat inspired by rust-analyzer.
//!
//! Its design (hopefully) will allow for easier integration with an IDE later down the line.

// macro hack
extern crate self as shin_asm;

pub mod compile;
pub mod parser;
pub mod syntax;

pub(crate) use compile::db::Jar;
