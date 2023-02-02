//! This crate implements the core functionality of shin engine
//!
//! This mostly includes file format parsing, virtual machine, and text layouting.

extern crate self as shin_core;

pub mod format;
pub mod layout;
pub mod time;
pub mod vm;
