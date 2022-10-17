use anyhow::{bail, Result};
use bytes::Buf;
use std::io::Cursor;

include!("conv_tables.rs");

#[inline]
fn convert_sjis_char(c: u16) -> char {
    if c < 0x20 {
        // SAFETY: c < 0x20, so it is safe to construct such a char
        unsafe {
            char::from_u32_unchecked(c as u32)
        }
    } else if (0x20..0x80).contains(&c) {
        let index = (c - 0x20) as usize;
        // SAFETY: index < 0x60, so it is safe to access the table
        unsafe { *ASCII_TABLE.get_unchecked(index) }
    } else if (0xa0..0xe0).contains(&c) {
        let index = (c - 0xa0) as usize;
        // SAFETY: index < 0x40, so it is safe to access the table
        unsafe { *KATAKANA_TABLE.get_unchecked(index) }
    } else {
        let lo = (c & 0xff) as u8;
        let hi = (c >> 8) as u8;

        // column actually spans two JIS rows
        // so, it's in range 0-193
        let column = if matches!(lo, 0x40..=0x7e | 0x80..=0xfc) {
            if (0x40..=0x7e).contains(&lo) {
                lo - 0x40
            } else {
                lo - 0x41
            }
        } else {
            return '\0'
        } as usize;

        let row = match hi {
            0x81..=0xa0 => (hi - 0x81) * 2,      // 64 JIS rows (each HI byte value spans 2 rows)
            0xe0..=0xfc => (hi - 0xe0) * 2 + 62, // 58 JIS rows (each HI byte value spans 2 rows)
            _ => return '\0'
        } as usize;

        // row \in [0; 121]
        // column \in [0; 193]
        // addr \in [0; 121*94 + 193] = [0; 11567]
        let addr = row * 94 + column;

        // SAFETY: addr < 11567, so it is safe to access the table
        unsafe { *JIS_TABLE.get_unchecked(addr) }
    }
}

fn is_extended(c: u8) -> bool {
    matches!(c, 0x81..=0x9f | 0xe0..=0xfc)
}

/// The game engine files are encoded in (a variant of) Shift-JIS
/// But the game engine itself uses UTF-8
/// This function converts (a variant of) Shift-JIS to UTF-8
pub fn read_sjis_string(s: &[u8]) -> Result<String> {
    let mut cur = Cursor::new(s);
    let mut res = String::new();
    // TODO: maybe there is a better estimation
    res.reserve(s.len());

    while cur.has_remaining() {
        let c = cur.get_u8();
        let c = if is_extended(c) {
            if !cur.has_remaining() {
                bail!("unexpected end of string when reading double-byte char");
            }
            let c2 = cur.get_u8();
            (c2 as u16) | ((c as u16) << 8)
        } else {
            c as u16
        };

        let utf8_c = convert_sjis_char(c);
        if utf8_c == '\0' {
            bail!("unmappable sjis char: 0x{:x}", c);
        }

        res.push(utf8_c);
    }

    Ok(res)
}

mod tests {
    use super::*;

    #[test]
    fn test_sjis() {
        let s = b"\x82\xa0\x82\xa2\x82\xa4\x82\xa6\x82\xa8";
        let s = read_sjis_string(s).unwrap();
        assert_eq!(s, "あいうえお");
    }

    include!("sjis_to_utf8_tests.rs");
    include!("sjis_unmapped_tests.rs");
}
