use super::*;

pub(crate) const LITERAL_FIRST: TokenSet = TokenSet::new(&[INT_NUMBER, FLOAT_NUMBER, STRING]);

pub(crate) fn literal(p: &mut Parser<'_>) -> Option<CompletedMarker> {
    if !p.at_ts(LITERAL_FIRST) {
        return None;
    }
    let m = p.start();
    p.bump_any();
    Some(m.complete(p, LITERAL))
}

pub(super) const ATOM_EXPR_FIRST: TokenSet =
    LITERAL_FIRST.union(TokenSet::new(&[IDENT, REGISTER_IDENT, T!['(']]));
pub(super) const EXPR_RECOVERY_SET: TokenSet = TokenSet::new(&[T![')'], T![']']]);
