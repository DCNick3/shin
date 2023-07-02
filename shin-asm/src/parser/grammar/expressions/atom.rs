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

pub(super) fn atom_expr(p: &mut Parser<'_>) -> Option<CompletedMarker> {
    if let Some(m) = literal(p) {
        return Some(m);
    }

    let done = match p.current() {
        IDENT => {
            let m = p.start();
            p.bump_any();
            m.complete(p, NAME_REF_EXPR)
        }
        REGISTER_IDENT => {
            let m = p.start();
            p.bump_any();
            m.complete(p, REGISTER_REF_EXPR)
        }
        T!['['] => array_expr(p),
        T!['{'] => mapping_expr(p),
        _ => {
            p.err_and_bump("expected expression");
            return None;
        }
    };
    Some(done)
}

fn array_expr(p: &mut Parser<'_>) -> CompletedMarker {
    assert!(p.at(T!['[']));
    let m = p.start();

    p.bump(T!['[']);
    while !p.at_ts(EOL_SET) && !p.at(T![']']) {
        if expr(p).is_none() {
            break;
        }

        if !p.at(T![']']) && !p.expect(T![,]) {
            break;
        }
    }
    p.expect(T![']']);

    m.complete(p, ARRAY_EXPR)
}

fn mapping_expr(_p: &mut Parser<'_>) -> CompletedMarker {
    todo!()
}
