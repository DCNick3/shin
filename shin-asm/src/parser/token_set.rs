//! A bit-set of `SyntaxKind`s.

use crate::parser::SyntaxKind;

/// A bit-set of `SyntaxKind`s
#[derive(Clone, Copy)]
// NOTE: this requires us to have less than 128 token types, and the tokens to have low values in the SyntaxKind enum
// this is enforced by the shin-derive macro
pub(crate) struct TokenSet(u128);

impl TokenSet {
    pub(crate) const EMPTY: TokenSet = TokenSet(0);

    pub(crate) const fn new(kinds: &[SyntaxKind]) -> TokenSet {
        let mut res = 0u128;
        let mut i = 0;
        while i < kinds.len() {
            res |= mask(kinds[i]);
            i += 1;
        }
        TokenSet(res)
    }

    pub(crate) const fn union(self, other: TokenSet) -> TokenSet {
        TokenSet(self.0 | other.0)
    }

    pub(crate) const fn add(self, kind: SyntaxKind) -> TokenSet {
        TokenSet(self.0 | mask(kind))
    }

    pub(crate) const fn contains(&self, kind: SyntaxKind) -> bool {
        self.0 & mask(kind) != 0
    }
}

const fn mask(kind: SyntaxKind) -> u128 {
    1u128 << (kind as usize)
}

#[test]
fn token_set_works_for_tokens() {
    use crate::parser::SyntaxKind::*;
    let ts = TokenSet::new(&[EOF, R_ANGLE]);
    assert!(ts.contains(EOF));
    assert!(ts.contains(R_ANGLE));
    assert!(!ts.contains(PLUS));
}
