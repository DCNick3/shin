mod atom;

use super::*;

pub(super) const EXPR_FIRST: TokenSet = LHS_FIRST;

pub(super) fn expr(p: &mut Parser<'_>) -> Option<CompletedMarker> {
    todo!()
}

const LHS_FIRST: TokenSet = atom::ATOM_EXPR_FIRST.union(TokenSet::new(&[T![!], T![~], T![-]]));

fn lhs(p: &mut Parser<'_>) -> Option<CompletedMarker> {
    todo!()
}
