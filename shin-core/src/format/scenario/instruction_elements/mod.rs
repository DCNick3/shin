//! Defines common types used in instructions. You know, almost like elements of instruction.

mod bitmask_number_array;
mod code_address;
mod message_id;
mod number_spec;
mod register;

pub use bitmask_number_array::BitmaskNumberArray;
pub use code_address::CodeAddress;
pub use message_id::MessageId;
pub use number_spec::NumberSpec;
pub use register::{Register, RegisterRepr, RegisterReprParseError};
