use std::fmt;

use bitbuffer::{BitRead, Endianness};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::format::save::BitReadStream;

type WordType = u32;
const WORD_BITS: usize = WordType::BITS as usize;

#[derive(Clone)]
pub struct IndexSet {
    bits: Vec<WordType>,
}

impl IndexSet {
    const ALLOC_AHEAD: usize = 0x100;

    /// Creates an empty [`IndexSet`]
    pub fn new() -> Self {
        Self { bits: Vec::new() }
    }

    /// Adds an index to the [`IndexSet`]
    ///
    /// Returns `true` if this index wasn't present in the set before, `false` if it was present there already.
    pub fn add(&mut self, index: usize) -> bool {
        let pos = Position::from_bit_index(index);

        if pos.byte_offset < self.bits.len() {
            if self.bits[pos.byte_offset] & pos.mask() != 0 {
                false
            } else {
                self.bits[pos.byte_offset] |= pos.mask();
                true
            }
        } else {
            // allocate more bits
            let new_size = (pos.byte_offset + 1).next_multiple_of(Self::ALLOC_AHEAD);
            self.bits.resize(new_size, 0);

            self.bits[pos.byte_offset] |= pos.mask();

            true
        }
    }

    /// Checks if an index is present in the [`IndexSet`]
    pub fn contains(&self, index: usize) -> bool {
        let pos = Position::from_bit_index(index);
        if pos.byte_offset < self.bits.len() {
            self.bits[pos.byte_offset] & pos.mask() != 0
        } else {
            false
        }
    }

    pub fn size_in_words(&self) -> usize {
        self.bits.len()
    }

    /// Iterate over all indices present in the [`IndexSet`]
    pub fn iter(&self) -> IndexSetIter {
        IndexSetIter {
            iter: self.bits.iter().peekable(),
            position: Position::from_bit_index(0),
        }
    }
}

impl fmt::Debug for IndexSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'a, E: Endianness> BitRead<'a, E> for IndexSet {
    fn read(stream: &mut BitReadStream<'a, E>) -> bitbuffer::Result<Self> {
        let bits = super::read_vec(stream, super::read_u16, |stream| stream.read_int(32))?;

        assert_eq!(bits.len() % Self::ALLOC_AHEAD, 0);

        Ok(Self { bits })
    }
}

impl Serialize for IndexSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.iter())
    }
}

impl<'de> Deserialize<'de> for IndexSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

struct Position {
    pub byte_offset: usize,
    pub bit_offset: u8,
}

impl Position {
    pub fn from_bit_index(value: usize) -> Self {
        Self {
            byte_offset: value / WORD_BITS,
            bit_offset: (value % WORD_BITS) as u8,
        }
    }
    pub fn as_bit_index(&self) -> usize {
        self.byte_offset * WORD_BITS + self.bit_offset as usize
    }
    pub fn mask(&self) -> WordType {
        1 << self.bit_offset
    }
    pub fn increment(&mut self) -> bool {
        self.add_bits(1)
    }

    pub fn add_bits(&mut self, value: u8) -> bool {
        self.bit_offset += value;
        if self.bit_offset >= WORD_BITS as u8 {
            // while possible to support, we do not want this
            assert_eq!(self.bit_offset, WORD_BITS as u8);

            self.bit_offset = 0;
            self.byte_offset += 1;
            true
        } else {
            false
        }
    }
}

pub struct IndexSetIter<'a> {
    iter: std::iter::Peekable<std::slice::Iter<'a, WordType>>,
    position: Position,
}

impl Iterator for IndexSetIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(&&value) = self.iter.peek() {
            let skip_zeroes = std::cmp::min(
                (value >> self.position.bit_offset).trailing_zeros() as u8,
                // avoid jumping over the byte boundary
                WORD_BITS as u8 - self.position.bit_offset,
            );
            let next_byte = self.position.add_bits(skip_zeroes);

            if !next_byte && self.position.mask() & value != 0 {
                // found an index to return!
                let result = self.position.as_bit_index();

                if self.position.increment() {
                    self.iter.next().unwrap();
                }

                return Some(result);
            }

            if next_byte {
                self.iter.next().unwrap();
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::{IndexSet, IndexSetIter, Position};

    #[test]
    #[rustfmt::skip]
    fn iter() {
        fn check_iter(bits: &[u32], expected_indices: &[usize]) {
            let iter = IndexSetIter {
                iter: bits.iter().peekable(),
                position: Position::from_bit_index(0),
            };

            let indices = iter.collect::<Vec<_>>();
            assert_eq!(&indices, expected_indices);
        }

        check_iter(&[], &[]);
        check_iter(&[0b0000_0000_0000_0000_0000_0000_0000_0000], &[]);
        check_iter(&[0b0000_0000_0000_0000_0000_0000_0000_0001], &[0]);
        check_iter(&[0b0000_0000_0000_0000_0000_0000_0000_0010], &[1]);
        check_iter(&[0b0000_0000_0000_0000_0000_0000_0000_0011], &[0, 1]);
        check_iter(&[0b0000_0000_0000_0000_0000_0001_0000_0000], &[8]);
        check_iter(&[0b0000_0000_0000_0000_1000_0000_0000_0000], &[15]);
        check_iter(&[0b1000_0000_0000_0000_0000_0000_0000_0000], &[31]);
        check_iter(&[0b0000_0000_0000_0000_0000_0000_0000_0000, 0b0000_0000_0000_0000_0000_0000_0000_0001], &[32]);
        check_iter(&[0b1000_0000_0000_0000_0000_0000_0000_0000, 0b0000_0000_0000_0000_0000_0000_0000_0001], &[31, 32]);
        check_iter(&[0b0000_0000_0000_0000_0000_0000_0000_0000, 0b1000_0000_0000_0000_0000_0000_0000_0000], &[63]);
    }

    #[test]
    fn set_then_get() {
        let mut vector = IndexSet::new();
        for bit in 0..512 {
            assert_eq!(vector.contains(bit), false);
            assert_eq!(vector.add(bit), true);
            assert_eq!(vector.add(bit), false);
            assert_eq!(vector.contains(bit), true);
        }
    }

    #[test]
    fn alloc_size() {
        let mut vector = IndexSet::new();
        assert_eq!(vector.size_in_words(), 0);
        vector.add(0);
        assert_eq!(vector.size_in_words(), IndexSet::ALLOC_AHEAD);

        let mut vector = IndexSet::new();
        assert_eq!(vector.size_in_words(), 0);
        vector.add(255);
        assert_eq!(vector.size_in_words(), IndexSet::ALLOC_AHEAD);
    }
}
