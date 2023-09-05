// macro hack
extern crate self as shin_asm;

pub mod compile;
pub mod elements;
pub mod parser;
pub mod syntax;

pub(crate) use compile::db::Jar;
