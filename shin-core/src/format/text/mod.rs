//! Encoding and decoding of Shift-JIS variant used by the shin engine.

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::io;

mod string;

pub use string::{SJisString, StringArray};

/// A zero-terminated Shift-JIS string.
pub type ZeroString = SJisString<()>;
/// A Shift-JIS string with a u8 length descriptor.
pub type U8String = SJisString<u8>;
/// A Shift-JIS string with a u16 length descriptor.
pub type U16String = SJisString<u16>;
/// A Shift-JIS string with a u8 length descriptor and fixup applied.
pub type U8FixupString = SJisString<u8, string::WithFixup>;
/// A Shift-JIS string with a u16 length descriptor and fixup applied.
pub type U16FixupString = SJisString<u16, string::WithFixup>;

include!("conv_tables.rs");

#[inline]
fn convert_single_sjis_char(c: u8) -> char {
    if c < 0x20 {
        // SAFETY: c < 0x20, so it is safe to construct such a char
        unsafe { char::from_u32_unchecked(c as u32) }
    } else if (0x20..0x80).contains(&c) {
        let index = (c - 0x20) as usize;
        // SAFETY: index < 0x60, so it is safe to access the table
        unsafe { *ASCII_TABLE.get_unchecked(index) }
    } else if (0xa0..0xe0).contains(&c) {
        let index = (c - 0xa0) as usize;
        // SAFETY: index < 0x40, so it is safe to access the table
        unsafe { *KATAKANA_TABLE.get_unchecked(index) }
    } else {
        // unmapped, no such first byte
        '\0'
    }
}

#[inline]
fn convert_double_sjis_char(first: u8, second: u8) -> char {
    // column actually spans two JIS rows
    // so, it's in range 0-193
    let column = if matches!(second, 0x40..=0x7e | 0x80..=0xfc) {
        if (0x40..=0x7e).contains(&second) {
            second - 0x40
        } else {
            second - 0x41
        }
    } else {
        return '\0';
    } as usize;

    let row = match first {
        0x81..=0xa0 => (first - 0x81) * 2, // 64 JIS rows (each HI byte value spans 2 rows)
        0xe0..=0xfc => (first - 0xe0) * 2 + 62, // 58 JIS rows (each HI byte value spans 2 rows)
        _ => return '\0',
    } as usize;

    // row \in [0; 121]
    // column \in [0; 193]
    // addr \in [0; 121*94 + 193] = [0; 11567]
    let addr = row * 94 + column;

    // SAFETY: addr < 11567, so it is safe to access the table
    unsafe { *JIS_TABLE.get_unchecked(addr) }
}

fn is_extended(c: u8) -> bool {
    matches!(c, 0x81..=0x9f | 0xe0..=0xfc)
}

/// The game engine files are encoded in (a variant of) Shift-JIS
/// But the game engine itself uses UTF-8
/// This function converts (a variant of) Shift-JIS to UTF-8
/// This function stops reading either at the first null byte or when byte_size bytes have been read
pub fn read_sjis_string<T: io::Read>(s: &mut T, byte_size: Option<usize>) -> io::Result<String> {
    use io::Read;

    let mut res = String::new();
    // TODO: maybe there is a better estimation
    if let Some(size) = byte_size {
        res.reserve(size);
    }
    let mut b = s
        .bytes()
        .take_while(|c| c.as_ref().map_or(true, |&c| c != 0))
        .take(byte_size.unwrap_or(usize::MAX));

    while let Some(c1) = b.next() {
        let c1 = c1?;
        let utf8_c = if is_extended(c1) {
            let c2 = b.next().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "unexpected end of string when reading double-byte char",
                )
            })??;
            let utf8_c = convert_double_sjis_char(c1, c2);

            if utf8_c == '\0' {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid double-byte char",
                ));
                // bail!("unmappable sjis char: 0x{:02x}, 0x{:02x}", c1, c2);
            }
            utf8_c
        } else {
            let utf8_c = convert_single_sjis_char(c1);
            if utf8_c == '\0' {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid single-byte char",
                ));
                // bail!("unmappable sjis char: 0x{:02x}", c1);
            }
            utf8_c
        };

        res.push(utf8_c);
    }

    Ok(res)
}

const FIXUP_ENCODED: &str = "｢｣ｧｨｩｪｫｬｭｮｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜｦﾝｰｯ､ﾟﾞ･?｡";
const FIXUP_DECODED: &str = "「」ぁぃぅぇぉゃゅょあいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんーっ、？！…　。";

static FIXUP_DECODE_TABLE: Lazy<HashMap<char, char>> =
    Lazy::new(|| FIXUP_ENCODED.chars().zip(FIXUP_DECODED.chars()).collect());

static FIXUP_ENCODE_TABLE: Lazy<HashMap<char, char>> =
    Lazy::new(|| FIXUP_DECODED.chars().zip(FIXUP_ENCODED.chars()).collect());

/// Apply transformations that the game does to some strings
/// This basically involves replacing some common characters with those that have shorted Shift-JIS encoding
pub fn encode_string_fixup(s: &str) -> String {
    s.chars()
        .map(|c| FIXUP_ENCODE_TABLE.get(&c).copied().unwrap_or(c))
        .collect()
}

/// Apply transformations that the game does to some strings
/// This basically involves replacing  
pub fn decode_string_fixup(s: &str) -> String {
    s.chars()
        .map(|c| FIXUP_DECODE_TABLE.get(&c).copied().unwrap_or(c))
        .collect()
}

mod tests {
    #[allow(unused)]
    use super::*;

    #[test]
    fn test_sjis() {
        let s = b"\x82\xa0\x82\xa2\x82\xa4\x82\xa6\x82\xa8";
        let s = read_sjis_string(&mut io::Cursor::new(s), Some(s.len())).unwrap();
        assert_eq!(s, "あいうえお");
    }

    // these files were auto-generated by a script
    // they check that the Shift_JIS decoder works the same way the original engine does it
    // to be more precise, it was tested against the Higirashi version
    include!("sjis_to_utf8_tests.rs");
    include!("sjis_unmapped_tests.rs");
}
