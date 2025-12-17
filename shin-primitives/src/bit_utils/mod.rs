//! Utilities for working with bit-level data.

pub mod bit_read;
mod buffer;
mod position;

pub use self::{
    buffer::{BitBuffer, BitInteger, BitReadError},
    position::BitPosition,
};
