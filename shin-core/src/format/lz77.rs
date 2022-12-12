//! This module contains implementation of the variant of lz77 used in the game
//!
//! To distinguish between literals and references by using bitmaps.
//!
//! These bitmaps are 8-bit integers, where each bit represents either one literal byte or one reference.
//!
//! The references are encoded as 16-bit big-endian (sic!) integers. It encodes offset and length of the reference,
//!     but the amount of bits spent on each part is dependent on the format (I use const generics for this).
//!
//! The minimum offset is 1, so the actual offset is offset + 1.
//! The minimum length is 3, so the actual length is length + 3.
//!
//! Encoding is (to be) implemented using a sliding window and a greedy algorithm.
//! Theoretically the efficiency can be improved by using a bit of backtracking,
//!     but it seems this improves compression ratio only by several percent (not worth the time).

use bytes::Buf;
use std::io;

pub fn decompress<const OFFSET_BITS: u32>(input: &[u8], output: &mut Vec<u8>) {
    let mut input = io::Cursor::new(input);

    while input.has_remaining() {
        let map = input.get_u8();
        for i in 0..8 {
            if !input.has_remaining() {
                break;
            }

            if ((map >> i) & 1) == 0 {
                /* literal value */
                output.push(input.get_u8());
            } else {
                /* back seek */
                let backseek_spec = input.get_u16(); // big endian Oo

                /*  MSB  XXXXXXXX          YYYYYYYY    LSB
                    val  len               backOffset
                    size (16-OFFSET_BITS)  OFFSET_BITS
                */

                let back_offset_mask = (1 << OFFSET_BITS) - 1; // magic to get the last OFFSET_BITS bits

                let len = (backseek_spec >> OFFSET_BITS) + 3;
                let back_offset = (backseek_spec & back_offset_mask) + 1;

                for _ in 0..len {
                    let last = output.len() - back_offset as usize;
                    // TODO: make this fallible?
                    // TODO: this might be optimized by stopping the bounds checking after we have enough data to guarantee that it's in bounds
                    output.push(output[last]);
                }
            }
        }
    }
}
