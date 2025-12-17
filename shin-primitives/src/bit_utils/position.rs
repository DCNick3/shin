use std::{marker::PhantomData, ops::Shl};

pub struct BitPosition<W> {
    pub byte_offset: usize,
    pub bit_offset: u8,
    pub phantom: PhantomData<W>,
}

impl<W: num_traits::One + Shl<u8, Output = W>> BitPosition<W> {
    const WORD_BITS: u8 = (size_of::<W>() * 8) as u8;

    pub fn from_bit_index(value: usize) -> Self {
        Self {
            byte_offset: value / Self::WORD_BITS as usize,
            bit_offset: (value % Self::WORD_BITS as usize) as u8,
            phantom: PhantomData,
        }
    }
    pub fn as_bit_index(&self) -> usize {
        self.byte_offset * Self::WORD_BITS as usize + self.bit_offset as usize
    }
    pub fn bit_offset_le(&self) -> u8 {
        self.bit_offset
    }
    pub fn bit_offset_be(&self) -> u8 {
        Self::WORD_BITS - self.bit_offset - 1
    }

    pub fn advance_one(&mut self) -> bool {
        self.advance(1)
    }

    pub fn bits_until_word_boundary(&self) -> u8 {
        if self.bit_offset == 0 {
            0
        } else {
            self.bits_until_next_word_boundary()
        }
    }

    pub fn bits_until_next_word_boundary(&self) -> u8 {
        Self::WORD_BITS - self.bit_offset
    }

    pub fn advance(&mut self, value: u8) -> bool {
        self.bit_offset += value;
        if self.bit_offset >= Self::WORD_BITS {
            // while possible to support, we do not want this
            assert_eq!(self.bit_offset, Self::WORD_BITS);

            self.bit_offset = 0;
            self.byte_offset += 1;
            true
        } else {
            false
        }
    }
}
