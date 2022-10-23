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
                    output.push(output[last]);
                }
            }
        }
    }
}
