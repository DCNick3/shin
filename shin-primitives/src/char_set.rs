/// Mask of the value bits of a continuation byte.
const CONT_MASK: u8 = 0b0011_1111;

/// Returns the initial codepoint accumulator for the first byte.
/// The first byte is special, only want bottom 5 bits for width 2, 4 bits
/// for width 3, and 3 bits for width 4.
#[inline]
const fn utf8_first_byte(byte: u8, width: u32) -> u32 {
    (byte & (0x7F >> width)) as u32
}

/// Returns the value of `ch` updated with continuation byte `byte`.
#[inline]
const fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
    (ch << 6) | (byte & CONT_MASK) as u32
}

struct BytesIter<'a> {
    bytes: &'a [u8],
    index: usize,
}
impl<'a> BytesIter<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, index: 0 }
    }
    const fn next(&mut self) -> Option<u8> {
        if self.index < self.bytes.len() {
            let byte = self.bytes[self.index];
            self.index += 1;
            Some(byte)
        } else {
            None
        }
    }
}

/// Reads the last code point out of a byte iterator (assuming a
/// UTF-8-like encoding).
///
/// # Safety
///
/// `bytes` must produce a valid UTF-8-like (UTF-8 or WTF-8) string
const unsafe fn next_code_point(bytes: &mut BytesIter) -> Option<u32> {
    // Decode UTF-8
    let x = match bytes.next() {
        Some(x) => x,
        None => return None,
    };
    if x < 128 {
        return Some(x as u32);
    }

    // Multibyte case follows
    // Decode from a byte combination out of: [[[x y] z] w]
    // NOTE: Performance is sensitive to the exact formulation here
    let init = utf8_first_byte(x, 2);
    // SAFETY: `bytes` produces an UTF-8-like string,
    // so the iterator must produce a value here.
    let y = unsafe { bytes.next().unwrap_unchecked() };
    let mut ch = utf8_acc_cont_byte(init, y);
    if x >= 0xE0 {
        // [[x y z] w] case
        // 5th bit in 0xE0 .. 0xEF is always clear, so `init` is still valid
        // SAFETY: `bytes` produces an UTF-8-like string,
        // so the iterator must produce a value here.
        let z = unsafe { bytes.next().unwrap_unchecked() };
        let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
        ch = init << 12 | y_z;
        if x >= 0xF0 {
            // [x y z w] case
            // use only the lower 3 bits of `init`
            // SAFETY: `bytes` produces an UTF-8-like string,
            // so the iterator must produce a value here.
            let w = unsafe { bytes.next().unwrap_unchecked() };
            ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
        }
    }

    Some(ch)
}

const fn insert_sorted(array: &mut [char], sorted_up_to: &mut usize, value: char) -> bool {
    let mut i = *sorted_up_to;
    if i >= array.len() {
        return false;
    }

    while i > 0 && array[i - 1] > value {
        array[i] = array[i - 1];
        i -= 1;
    }
    array[i] = value;
    *sorted_up_to += 1;

    true
}

struct InsertionSorter<const N: usize> {
    chars: [char; N],
    sorted_up_to: usize,
}

impl<const N: usize> InsertionSorter<N> {
    const fn new() -> Self {
        Self {
            chars: ['\0'; N],
            sorted_up_to: 0,
        }
    }

    const fn insert(&mut self, value: char) -> bool {
        // call a type-erased function to not blow up the code size unnecessarily
        insert_sorted(&mut self.chars, &mut self.sorted_up_to, value)
    }

    const fn get_sorted(&self) -> Option<[char; N]> {
        if self.sorted_up_to == N {
            Some(self.chars)
        } else {
            None
        }
    }
}

fn char_set_contains(set: &[char], value: char) -> bool {
    set.binary_search(&value).is_ok()
}

pub struct CharSet<const N: usize> {
    chars: [char; N],
}

impl<const N: usize> CharSet<N> {
    //noinspection RsAssertEqual
    pub const fn new(contents: &str) -> Self {
        let mut iter = BytesIter::new(contents.as_bytes());
        let mut sorter = InsertionSorter::<N>::new();
        while let Some(codepoint) = unsafe { next_code_point(&mut iter) } {
            // insertion sort is O(N^2), but it can be implemented with const fn
            if !sorter.insert(unsafe { char::from_u32_unchecked(codepoint) }) {
                panic!("CharSet::new: the template parameter N does not match the number of characters in the string");
            }
        }

        let Some(chars) = sorter.get_sorted() else {
            panic!("CharSet::new: the template parameter N does not match the number of characters in the string");
        };

        Self { chars }
    }

    pub fn contains(&self, value: char) -> bool {
        // call a type-erased function to not blow up the code size unnecessarily
        char_set_contains(&self.chars, value)
    }
}

#[cfg(test)]
mod tests {
    use crate::char_set::CharSet;

    #[test]
    fn test_smoke() {
        const TABLE1: CharSet<3> = CharSet::new("312");
        assert_eq!(&TABLE1.chars, &['1', '2', '3']);
        assert!(!TABLE1.contains('0'));
        assert!(TABLE1.contains('1'));
        assert!(TABLE1.contains('2'));
        assert!(TABLE1.contains('3'));
        assert!(!TABLE1.contains('4'));
    }

    #[test]
    fn test_overflow() {
        let error = std::panic::catch_unwind(|| {
            CharSet::<1>::new("12");
        })
        .unwrap_err();
        assert_eq!(error.downcast_ref::<&str>(), Some(&"CharSet::new: the template parameter N does not match the number of characters in the string"));
    }

    #[test]
    fn test_underflow() {
        let error = std::panic::catch_unwind(|| {
            CharSet::<3>::new("1");
        })
        .unwrap_err();
        assert_eq!(error.downcast_ref::<&str>(), Some(&"CharSet::new: the template parameter N does not match the number of characters in the string"));
    }

    #[test]
    fn test_unicode() {
        const TABLE1: CharSet<2> = CharSet::new("🦀奉");
        assert_eq!(&TABLE1.chars, &['奉', '🦀']);
        assert!(!TABLE1.contains('🦄'));
        assert!(TABLE1.contains('🦀'));
        assert!(TABLE1.contains('奉'));
    }
}
