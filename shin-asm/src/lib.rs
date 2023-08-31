// macro hack
extern crate self as shin_asm;

mod db;
// pub mod file_db;
pub mod parser;
// pub mod symbols;
pub mod hir;
pub mod syntax;

pub(crate) use db::Jar;
