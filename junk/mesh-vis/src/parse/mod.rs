pub mod matrix;
pub mod modes;
pub mod vertex;

use iced_core::color;
use iced_widget::{text, Column};

use crate::Message;

pub enum HexParseResult {
    Valid,
    PartiallyValid,
}

pub struct HexParseState {
    pub bytes: Vec<u8>,
    pub result: HexParseResult,
}

impl HexParseState {
    pub fn parse(text: &str) -> Self {
        // find first non-hex character
        let mut valid_hex_prefix_len = text
            .find(|c: char| !c.is_ascii_hexdigit())
            .unwrap_or(text.len());
        if valid_hex_prefix_len % 2 != 0 {
            valid_hex_prefix_len -= 1;
        }
        let valid_hex_prefix = &text[..valid_hex_prefix_len];
        let parsed_bytes = hex::decode(valid_hex_prefix).unwrap();
        let parse_state = if valid_hex_prefix_len == text.len() {
            HexParseResult::Valid
        } else {
            HexParseResult::PartiallyValid
        };

        Self {
            bytes: parsed_bytes,
            result: parse_state,
        }
    }

    pub fn view(&self) -> Column<'static, Message> {
        Column::with_children([
            match self.result {
                HexParseResult::Valid => text("Valid hex").color(color!(0x00c000)),
                HexParseResult::PartiallyValid => text("Invalid hex!").color(color!(0xc00000)),
            }
            .into(),
            text(format!("{0} (0x{0:x}) bytes", self.bytes.len())).into(),
        ])
    }
}

pub enum FloatParseResult {
    Valid,
    PartiallyValid,
}

pub struct FloatParseState {
    pub raw_floats: Vec<[u8; 4]>,
    pub floats: Vec<f32>,
    pub result: FloatParseResult,
}

impl FloatParseState {
    pub fn parse(bytes: &[u8]) -> Self {
        const FLOAT_LEN: usize = 4;

        let mut valid_prefix_len = bytes.len();
        valid_prefix_len -= valid_prefix_len % FLOAT_LEN;

        let valid_prefix = &bytes[..valid_prefix_len];

        let raw_floats: Vec<[u8; 4]> = valid_prefix
            .chunks_exact(4)
            .map(|c| c.try_into().unwrap())
            .collect();
        let floats = raw_floats.iter().map(|&c| f32::from_le_bytes(c)).collect();

        let result = if valid_prefix_len == bytes.len() {
            FloatParseResult::Valid
        } else {
            FloatParseResult::PartiallyValid
        };

        Self {
            raw_floats,
            floats,
            result,
        }
    }

    pub fn view(&self) -> Column<'static, Message> {
        Column::with_children([
            match self.result {
                FloatParseResult::Valid => text("Valid floats").color(color!(0x00c000)),
                FloatParseResult::PartiallyValid => text("Invalid floats!").color(color!(0xc00000)),
            }
            .into(),
            text(format!("{0} (0x{0:x}) floats", self.floats.len())).into(),
            text(format!("{:?}", self.floats)).into(),
        ])
    }
}
