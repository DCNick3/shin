//! This crate implements the core functionality of shin engine
//!
//! This mostly includes file format parsing, virtual machine, and text layouting.

#![allow(clippy::uninlined_format_args)]

// macro hack
extern crate self as shin_core;

// re-export for convenience
pub use shin_tasks::create_task_pools;

pub mod format;
pub mod layout;
pub mod time;
pub mod vm;
