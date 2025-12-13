use std::{marker::PhantomData, ops::Shl};

pub struct BitPosition<W> {
    pub byte_offset: usize,
    pub bit_offset: u8,
    pub phantom: PhantomData<W>,
}

impl<W: num_traits::One + Shl<u8, Output = W>> BitPosition<W> {
    const WORD_BITS: usize = size_of::<W>() * 8;

    pub fn from_bit_index(value: usize) -> Self {
        Self {
            byte_offset: value / Self::WORD_BITS,
            bit_offset: (value % Self::WORD_BITS) as u8,
            phantom: PhantomData,
        }
    }
    pub fn as_bit_index(&self) -> usize {
        self.byte_offset * Self::WORD_BITS + self.bit_offset as usize
    }
    pub fn mask(&self) -> W {
        W::one() << self.bit_offset
    }
    pub fn increment(&mut self) -> bool {
        self.add_bits(1)
    }

    pub fn add_bits(&mut self, value: u8) -> bool {
        self.bit_offset += value;
        if self.bit_offset >= Self::WORD_BITS as u8 {
            // while possible to support, we do not want this
            assert_eq!(self.bit_offset, Self::WORD_BITS as u8);

            self.bit_offset = 0;
            self.byte_offset += 1;
            true
        } else {
            false
        }
    }
}
