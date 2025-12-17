use std::{fmt, fmt::Display};

use crate::bit_utils::BitPosition;

pub trait BitInteger {
    const BITS: u8;
    fn from_u64(value: u64) -> Self;
}

macro_rules! impl_bit_int {
    ($($ty:ty),*) => {
        $(
            impl BitInteger for $ty {
                const BITS: u8 = <$ty>::BITS as u8;
                fn from_u64(value: u64) -> Self {
                    value as $ty
                }
            }
        )*
    };
}

impl_bit_int!(u8, u16, u32, u64, i8, i16, i32, i64);

type WordType = u8;

mod extract {
    /// Extract `count` bits starting from `start` out of `source`; return them in LSB part of the byte.
    ///
    /// Unlike common in other places, for this function bits are counted from MSB to LSB:
    ///   0123 4567
    /// 0bXXXX_XXXX
    #[inline]
    pub fn extract_bits(source: u8, start: u8, count: u8) -> u8 {
        debug_assert!(count <= 8);
        debug_assert!(start < 8);
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

    pub fn align(&mut self) {
        let advance = self.position.bits_until_word_boundary();
        self.position.advance(advance);
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

    fn read_bool_impl(&mut self) -> bool {
        let current_byte = self.data[self.position.byte_offset];
        let result = extract::extract_bit(current_byte, self.position.bit_offset);

        self.position.advance_one();

        result
    }

    pub fn read_bool(&mut self) -> Result<bool, BitReadError> {
        self.check_enough_bits(1)?;
        Ok(self.read_bool_impl())
    }

    fn read_int_impl(&mut self, count: u8) -> u64 {
        let mut result = 0;

        let mut remaining_count = count;

        while remaining_count > 0 {
            let can_take = self.position.bits_until_next_word_boundary();
            let take = std::cmp::min(remaining_count, can_take);

            let current_byte = self.data[self.position.byte_offset];
            let extracted = extract::extract_bits(current_byte, self.position.bit_offset, take);

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
}

#[cfg(test)]
mod tests {
    use super::BitBuffer;

    #[test]
    fn one_byte() {
        let mut buffer = BitBuffer::with_data(vec![0b0000_0000], 0);
        assert_eq!(buffer.read_int::<u8>(8).unwrap(), 0b0000_0000);

        let mut buffer = BitBuffer::with_data(vec![0b1111_0000], 0);
        assert_eq!(buffer.read_int::<u8>(8).unwrap(), 0b1111_0000);

        let mut buffer = BitBuffer::with_data(vec![0b1111_0000], 0);
        assert_eq!(buffer.read_int::<u8>(4).unwrap(), 0b1111);
        assert_eq!(buffer.read_int::<u8>(4).unwrap(), 0b0000);
    }

    #[test]
    fn two_bytes() {
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
    fn four_bytes() {
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
    fn bits_left() {
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
}
