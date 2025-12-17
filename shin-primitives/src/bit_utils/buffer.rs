use std::{fmt, fmt::Display};

use crate::bit_utils::BitPosition;

pub trait BitInteger {
    const BITS: u8;
    fn from_u64(value: u64) -> Self;
    fn into_u64(self) -> u64;
}

macro_rules! impl_bit_int {
    ($($ty:ty),*) => {
        $(
            impl BitInteger for $ty {
                const BITS: u8 = <$ty>::BITS as u8;
                fn from_u64(value: u64) -> Self {
                    value as $ty
                }
                fn into_u64(self) -> u64 {
                    self as u64
                }
            }
        )*
    };
}

impl_bit_int!(u8, u16, u32, u64, i8, i16, i32, i64);

type WordType = u8;

mod bits {
    /// Extract `count` bits starting from `start` out of `source`; return them in LSB part of the byte.
    ///
    /// Unlike common in other places, for this function bits are counted from MSB to LSB:
    ///   0123 4567
    /// 0bXXXX_XXXX
    #[inline]
    pub fn extract_bits(source: u8, start: u8, count: u8) -> u8 {
        debug_assert!(start + count <= 8);
        if count == 0 {
            return 0;
        }

        source << start >> (8 - count)
    }

    #[test]
    fn test_extract_bits() {
        assert_eq!(extract_bits(0b1111_0000, 0, 8), 0b1111_0000);
        assert_eq!(extract_bits(0b1111_0000, 0, 7), 0b111_1000);
        assert_eq!(extract_bits(0b1111_0000, 0, 6), 0b11_1100);
        assert_eq!(extract_bits(0b1111_0000, 0, 5), 0b1_1110);
        assert_eq!(extract_bits(0b1111_0000, 0, 4), 0b1111);
        assert_eq!(extract_bits(0b1111_0000, 0, 3), 0b111);
        assert_eq!(extract_bits(0b1111_0000, 0, 2), 0b11);
        assert_eq!(extract_bits(0b1111_0000, 0, 1), 0b1);
        assert_eq!(extract_bits(0b1111_0000, 0, 0), 0b0);

        assert_eq!(extract_bits(0b1100_0101, 0, 2), 0b11);
        assert_eq!(extract_bits(0b1100_0101, 1, 2), 0b10);
        assert_eq!(extract_bits(0b1100_0101, 2, 2), 0b00);
        assert_eq!(extract_bits(0b1100_0101, 3, 2), 0b00);
        assert_eq!(extract_bits(0b1100_0101, 4, 2), 0b01);
        assert_eq!(extract_bits(0b1100_0101, 5, 2), 0b10);
        assert_eq!(extract_bits(0b1100_0101, 6, 2), 0b01);
    }

    #[inline]
    pub fn extract_bit(source: u8, position: u8) -> bool {
        debug_assert!(position < 8);
        ((source >> (7 - position)) & 1) != 0
    }

    #[test]
    fn test_extract_bit() {
        assert_eq!(extract_bit(0b1111_0000, 0), true);
        assert_eq!(extract_bit(0b1111_0000, 1), true);
        assert_eq!(extract_bit(0b1111_0000, 2), true);
        assert_eq!(extract_bit(0b1111_0000, 3), true);
        assert_eq!(extract_bit(0b1111_0000, 4), false);
        assert_eq!(extract_bit(0b1111_0000, 5), false);
        assert_eq!(extract_bit(0b1111_0000, 6), false);
        assert_eq!(extract_bit(0b1111_0000, 7), false);

        assert_eq!(extract_bit(0b1100_0101, 0), true);
        assert_eq!(extract_bit(0b1100_0101, 1), true);
        assert_eq!(extract_bit(0b1100_0101, 2), false);
        assert_eq!(extract_bit(0b1100_0101, 3), false);
        assert_eq!(extract_bit(0b1100_0101, 4), false);
        assert_eq!(extract_bit(0b1100_0101, 5), true);
        assert_eq!(extract_bit(0b1100_0101, 6), false);
        assert_eq!(extract_bit(0b1100_0101, 7), true);
    }

    #[inline]
    pub fn insert_bits(current: u8, start: u8, count: u8, value: u8) -> u8 {
        debug_assert!(start + count <= 8);
        if count == 0 {
            return current;
        }

        let left_bits = 8 - count;
        let positioned_value = value << left_bits >> start;
        let positioned_mask = 0xff << left_bits >> start;

        (current & !positioned_mask) | (positioned_value & positioned_mask)
    }

    #[inline]
    pub fn insert_bits_ref(current: &mut u8, start: u8, count: u8, value: u8) {
        *current = insert_bits(*current, start, count, value)
    }

    #[test]
    fn test_insert_bits() {
        assert_eq!(insert_bits(0b1111_0000, 0, 0, 0b0), 0b1111_0000);

        // this checks that extra bits are truncated
        assert_eq!(insert_bits(0b1111_0000, 0, 1, 0b10), 0b0111_0000);
        assert_eq!(insert_bits(0b1111_0000, 1, 1, 0b10), 0b1011_0000);
        assert_eq!(insert_bits(0b1111_0000, 2, 1, 0b10), 0b1101_0000);
        assert_eq!(insert_bits(0b1111_0000, 3, 1, 0b10), 0b1110_0000);
        assert_eq!(insert_bits(0b1111_0000, 4, 1, 0b10), 0b1111_0000);
        assert_eq!(insert_bits(0b1111_0000, 5, 1, 0b10), 0b1111_0000);
        assert_eq!(insert_bits(0b1111_0000, 6, 1, 0b10), 0b1111_0000);
        assert_eq!(insert_bits(0b1111_0000, 7, 1, 0b10), 0b1111_0000);

        assert_eq!(insert_bits(0b1111_0000, 0, 2, 0b10), 0b1011_0000);
        assert_eq!(insert_bits(0b1111_0000, 1, 2, 0b10), 0b1101_0000);
        assert_eq!(insert_bits(0b1111_0000, 2, 2, 0b10), 0b1110_0000);
        assert_eq!(insert_bits(0b1111_0000, 3, 2, 0b10), 0b1111_0000);
        assert_eq!(insert_bits(0b1111_0000, 4, 2, 0b10), 0b1111_1000);
        assert_eq!(insert_bits(0b1111_0000, 5, 2, 0b10), 0b1111_0100);
        assert_eq!(insert_bits(0b1111_0000, 6, 2, 0b10), 0b1111_0010);

        assert_eq!(insert_bits(0b1111_0000, 0, 4, 0b1111), 0b1111_0000);
        assert_eq!(insert_bits(0b1111_0000, 1, 4, 0b1111), 0b1111_1000);
        assert_eq!(insert_bits(0b1111_0000, 2, 4, 0b1111), 0b1111_1100);
        assert_eq!(insert_bits(0b1111_0000, 3, 4, 0b1111), 0b1111_1110);
        assert_eq!(insert_bits(0b1111_0000, 4, 4, 0b1111), 0b1111_1111);

        assert_eq!(insert_bits(0b1111_0000, 0, 4, 0b0000), 0b0000_0000);
        assert_eq!(insert_bits(0b1111_0000, 1, 4, 0b0000), 0b1000_0000);
        assert_eq!(insert_bits(0b1111_0000, 2, 4, 0b0000), 0b1100_0000);
        assert_eq!(insert_bits(0b1111_0000, 3, 4, 0b0000), 0b1110_0000);
        assert_eq!(insert_bits(0b1111_0000, 4, 4, 0b0000), 0b1111_0000);

        assert_eq!(insert_bits(0b1111_0000, 0, 8, 0b0000_1111), 0b0000_1111);
    }

    #[inline]
    pub fn insert_bit(current: u8, position: u8, value: bool) -> u8 {
        debug_assert!(position < 8);

        if value {
            current | (0x80 >> position)
        } else {
            current & !(0x80 >> position)
        }
    }

    #[inline]
    pub fn insert_bit_ref(current: &mut u8, position: u8, value: bool) {
        *current = insert_bit(*current, position, value)
    }

    #[test]
    fn test_insert_bit() {
        assert_eq!(insert_bit(0b1111_0000, 0, true), 0b1111_0000);
        assert_eq!(insert_bit(0b1111_0000, 1, true), 0b1111_0000);
        assert_eq!(insert_bit(0b1111_0000, 2, true), 0b1111_0000);
        assert_eq!(insert_bit(0b1111_0000, 3, true), 0b1111_0000);
        assert_eq!(insert_bit(0b1111_0000, 4, true), 0b1111_1000);
        assert_eq!(insert_bit(0b1111_0000, 5, true), 0b1111_0100);
        assert_eq!(insert_bit(0b1111_0000, 6, true), 0b1111_0010);
        assert_eq!(insert_bit(0b1111_0000, 7, true), 0b1111_0001);

        assert_eq!(insert_bit(0b1111_0000, 0, false), 0b0111_0000);
        assert_eq!(insert_bit(0b1111_0000, 1, false), 0b1011_0000);
        assert_eq!(insert_bit(0b1111_0000, 2, false), 0b1101_0000);
        assert_eq!(insert_bit(0b1111_0000, 3, false), 0b1110_0000);
        assert_eq!(insert_bit(0b1111_0000, 4, false), 0b1111_0000);
        assert_eq!(insert_bit(0b1111_0000, 5, false), 0b1111_0000);
        assert_eq!(insert_bit(0b1111_0000, 6, false), 0b1111_0000);
        assert_eq!(insert_bit(0b1111_0000, 7, false), 0b1111_0000);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum BitReadError {
    /// Not enough data in the buffer to read all requested bits
    NotEnoughData {
        /// The number of bits requested to read
        requested: usize,
        /// the number of bits left in the buffer
        bits_left: usize,
    },
}

impl Display for BitReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            BitReadError::NotEnoughData {
                requested,
                bits_left,
            } => write!(
                f,
                "Not enough data in the buffer to read all requested bits, requested to read {} bits while only {} bits are left",
                requested, bits_left
            ),
        }
    }
}

impl std::error::Error for BitReadError {}

pub struct BitBuffer {
    data: Vec<WordType>,
    position: BitPosition<WordType>,
}

impl BitBuffer {
    const ALLOC_AHEAD: usize = 0x400;

    pub fn with_data(data: Vec<u8>, position_bits: usize) -> Self {
        Self {
            data,
            position: BitPosition::from_bit_index(position_bits),
        }
    }

    pub fn empty() -> Self {
        Self {
            data: Vec::new(),
            position: BitPosition::from_bit_index(0),
        }
    }

    pub fn into_data(self) -> (Vec<WordType>, usize) {
        (self.data, self.position.as_bit_index())
    }

    pub fn align(&mut self) {
        let advance = self.position.bits_until_word_boundary();
        self.position.advance(advance);
    }

    pub fn bit_position(&self) -> usize {
        self.position.as_bit_index()
    }

    pub fn bits_left(&self) -> usize {
        if self.data.len() == self.position.byte_offset {
            // prevent subtraction with underflow
            assert_eq!(self.position.bit_offset, 0);
            return 0;
        }

        let bytes_left = self.data.len() - self.position.byte_offset - 1;
        let bits_left = 8 - self.position.bit_offset;

        bytes_left * 8 + bits_left as usize
    }

    fn check_enough_bits(&self, count: u8) -> Result<(), BitReadError> {
        let bits_left = self.bits_left();
        if bits_left < count as usize {
            Err(BitReadError::NotEnoughData {
                requested: count as usize,
                bits_left,
            })
        } else {
            Ok(())
        }
    }

    fn expand_for_bits(&mut self, count: u8) {
        let bits_left = self.bits_left();
        if bits_left >= count as usize {
            return;
        }
        let required_bytes = (self.position.as_bit_index() + count as usize).div_ceil(8);
        let new_size = required_bytes.next_multiple_of(Self::ALLOC_AHEAD);

        self.data.resize(new_size, 0);
    }

    pub fn read_bool(&mut self) -> Result<bool, BitReadError> {
        self.check_enough_bits(1)?;

        let current_byte = self.data[self.position.byte_offset];
        let result = bits::extract_bit(current_byte, self.position.bit_offset);

        self.position.advance_one();

        Ok(result)
    }

    fn read_int_impl(&mut self, count: u8) -> u64 {
        let mut result = 0;

        let mut remaining_count = count;

        while remaining_count > 0 {
            let can_take = self.position.bits_until_next_word_boundary();
            let take = std::cmp::min(remaining_count, can_take);

            let current_byte = self.data[self.position.byte_offset];
            let extracted = bits::extract_bits(current_byte, self.position.bit_offset, take);

            result = (result << take) | extracted as u64;

            self.position.advance(take);
            remaining_count -= take;
        }

        result
    }

    #[inline]
    pub fn read_int<T: BitInteger>(&mut self, count: u8) -> Result<T, BitReadError> {
        assert!(count <= T::BITS);
        self.check_enough_bits(count)?;
        let value = self.read_int_impl(count);
        Ok(T::from_u64(value))
    }

    pub fn write_bool(&mut self, value: bool) {
        self.expand_for_bits(1);

        let current_byte = &mut self.data[self.position.byte_offset];
        bits::insert_bit_ref(current_byte, self.position.bit_offset, value);

        self.position.advance_one();
    }

    fn write_int_impl(&mut self, count: u8, value: u64) {
        assert!(
            (u64::BITS - value.leading_zeros()) as u8 <= count,
            "Value is too big to fit into the specified amount of bits"
        );

        let mut remaining_count = count;

        while remaining_count > 0 {
            let can_take = self.position.bits_until_next_word_boundary();
            let take = std::cmp::min(remaining_count, can_take);

            // insert_bits will truncate the extra bits
            let insert = (value >> (remaining_count - take)) as u8;

            let current_byte = &mut self.data[self.position.byte_offset];
            bits::insert_bits_ref(current_byte, self.position.bit_offset, take, insert);

            self.position.advance(take);
            remaining_count -= take;
        }
    }

    #[inline]
    pub fn write_int<T: BitInteger>(&mut self, count: u8, value: T) {
        self.expand_for_bits(count);

        self.write_int_impl(count, value.into_u64());
    }
}

#[cfg(test)]
mod tests {
    use super::BitBuffer;

    #[test]
    fn read_one_byte() {
        let mut buffer = BitBuffer::with_data(vec![0b0000_0000], 0);
        assert_eq!(buffer.read_int::<u8>(8).unwrap(), 0b0000_0000);

        let mut buffer = BitBuffer::with_data(vec![0b1111_0000], 0);
        assert_eq!(buffer.read_int::<u8>(8).unwrap(), 0b1111_0000);

        let mut buffer = BitBuffer::with_data(vec![0b1111_0000], 0);
        assert_eq!(buffer.read_int::<u8>(4).unwrap(), 0b1111);
        assert_eq!(buffer.read_int::<u8>(4).unwrap(), 0b0000);
    }

    #[test]
    fn read_two_bytes() {
        let mut buffer = BitBuffer::with_data(vec![0b0000_0000, 0b1111_1111], 0);
        assert_eq!(buffer.read_int::<u16>(16).unwrap(), 0b0000_0000_1111_1111);

        let mut buffer = BitBuffer::with_data(vec![0b0000_0000, 0b1111_1111], 0);
        assert_eq!(buffer.read_int::<u16>(9).unwrap(), 0b0_0000_0001);
        assert_eq!(buffer.read_int::<u16>(7).unwrap(), 0b111_1111);

        let mut buffer = BitBuffer::with_data(vec![0b1111_0000, 0b0000_1111], 0);
        assert_eq!(buffer.read_int::<u16>(16).unwrap(), 0b1111_0000_0000_1111);

        let mut buffer = BitBuffer::with_data(vec![0b1111_0000, 0b1111_0000], 0);
        assert_eq!(buffer.read_int::<u16>(4).unwrap(), 0b1111);
        assert_eq!(buffer.read_int::<u16>(3).unwrap(), 0b000);
        assert_eq!(buffer.read_int::<u16>(2).unwrap(), 0b01);
        assert_eq!(buffer.read_int::<u16>(1).unwrap(), 0b1);
        assert_eq!(buffer.read_bool().unwrap(), true);
        assert_eq!(buffer.read_bool().unwrap(), true);
        assert_eq!(buffer.read_bool().unwrap(), false);
        assert_eq!(buffer.read_bool().unwrap(), false);
        assert_eq!(buffer.read_bool().unwrap(), false);
        assert_eq!(buffer.read_bool().unwrap(), false);
    }

    #[test]
    fn read_four_bytes() {
        let mut buffer =
            BitBuffer::with_data(vec![0b0000_0000, 0b1111_1111, 0b0000_0000, 0b1111_1111], 0);
        assert_eq!(
            buffer.read_int::<u32>(32).unwrap(),
            0b0000_0000_1111_1111_0000_0000_1111_1111
        );

        let mut buffer =
            BitBuffer::with_data(vec![0b0000_0000, 0b1111_1111, 0b0000_0000, 0b1111_1111], 0);
        assert_eq!(buffer.read_bool().unwrap(), false);
        assert_eq!(
            buffer.read_int::<u32>(30).unwrap(),
            0b00_0000_0111_1111_1000_0000_0111_1111
        );
        assert_eq!(buffer.read_bool().unwrap(), true);
    }

    #[test]
    fn align() {
        let mut buffer = BitBuffer::with_data(vec![0b1111_0000, 0b1111_0000], 0);
        assert_eq!(buffer.read_int::<u8>(7).unwrap(), 0b111_1000);
        buffer.align();

        assert_eq!(buffer.read_int::<u8>(6).unwrap(), 0b11_1100);
        buffer.align();
    }

    #[test]
    fn read_bits_left() {
        let mut buffer = BitBuffer::with_data(vec![0b1111_0000, 0b1111_0000], 0);
        assert_eq!(buffer.bits_left(), 16);
        buffer.read_bool().unwrap();

        assert_eq!(buffer.bits_left(), 15);
        buffer.read_int::<u64>(7).unwrap();

        assert_eq!(buffer.bits_left(), 8);
        buffer.read_bool().unwrap();

        assert_eq!(buffer.bits_left(), 7);
        buffer.read_int::<u64>(7).unwrap();

        assert_eq!(buffer.bits_left(), 0);
    }

    fn finish_write(buffer: BitBuffer) -> (Vec<u8>, usize) {
        // the bitbuffer in the game allocates bytes in chunks, presumably to amortize the costs
        // we do the same to match the written files. but for tests we only want actually written bytes
        let (data, bits) = buffer.into_data();
        let num_bytes = bits.div_ceil(8);

        (data[..num_bytes].to_vec(), bits)
    }

    #[test]
    fn write_one_byte() {
        let mut buffer = BitBuffer::empty();
        buffer.write_bool(true);
        assert_eq!(finish_write(buffer), (vec![0b1000_0000], 1));

        let mut buffer = BitBuffer::empty();
        buffer.write_int(2, 0b01);
        buffer.write_int(6, 0b11_0110);
        assert_eq!(finish_write(buffer), (vec![0b0111_0110], 8));
    }

    #[test]
    fn write_two_bytes() {
        let mut buffer = BitBuffer::empty();
        buffer.write_int(16, 0b0111_0110_1110_1100);
        assert_eq!(finish_write(buffer), (vec![0b0111_0110, 0b1110_1100], 16));

        let mut buffer = BitBuffer::empty();
        buffer.write_int(7, 0b011_0110);
        buffer.write_int(5, 0b1_1011);
        assert_eq!(finish_write(buffer), (vec![0b0110_1101, 0b1011_0000], 12));
    }

    #[test]
    fn write_four_bytes() {
        let mut buffer = BitBuffer::empty();
        buffer.write_int(32, 0b0000_0000_1111_1111_0000_0000_1111_1111);
        assert_eq!(
            finish_write(buffer),
            (vec![0b0000_0000, 0b1111_1111, 0b0000_0000, 0b1111_1111], 32)
        );

        let mut buffer = BitBuffer::empty();
        buffer.write_bool(false);
        buffer.write_int(30, 0b00_0000_0111_1111_1000_0000_0111_1111);
        buffer.write_bool(true);
        assert_eq!(
            finish_write(buffer),
            (vec![0b0000_0000, 0b1111_1111, 0b0000_0000, 0b1111_1111], 32)
        );
    }
}
