use super::*;

pub(crate) const LITERAL_FIRST: TokenSet = TokenSet::new(&[INT_NUMBER, RATIONAL_NUMBER, STRING]);

pub(crate) fn literal(p: &mut Parser<'_>) -> Option<CompletedMarker> {
    if !p.at_ts(LITERAL_FIRST) {
        return None;
    }
    let m = p.start();
    p.bump_any();
    Some(m.complete(p, LITERAL))
}

pub(super) const ATOM_EXPR_FIRST: TokenSet = LITERAL_FIRST.union(TokenSet::new(&[
    IDENT,
    REGISTER_IDENT,
    T!['('],
    T!['{'],
    T!['['],
]));
pub(super) const EXPR_RECOVERY_SET: TokenSet = TokenSet::new(&[T![')'], T!['}'], T![']']]);

pub(super) fn atom_expr(p: &mut Parser<'_>) -> Option<CompletedMarker> {
    if let Some(m) = literal(p) {
        return Some(m);
    }

    let done = match p.current() {
        IDENT => {
            let m = p.start();
            p.bump(IDENT);
            m.complete(p, NAME_REF_EXPR)
        }
        REGISTER_IDENT => {
            let m = p.start();
            p.bump(REGISTER_IDENT);
            m.complete(p, REGISTER_REF_EXPR)
        }
        T!['('] => paren_expr(p),
        T!['['] => array_expr(p),
        T!['{'] => mapping_expr(p),
        _ => {
            p.err_and_bump("expected expression");
            return None;
        }
    };
    Some(done)
}

fn paren_expr(p: &mut Parser<'_>) -> CompletedMarker {
    assert!(p.at(T!['(']));
    let m = p.start();

    p.bump(T!['(']);
    expr(p);
    p.expect(T![')']);

    m.complete(p, PAREN_EXPR)
}

fn array_expr(p: &mut Parser<'_>) -> CompletedMarker {
    assert!(p.at(T!['[']));
    let m = p.start();

    p.bump(T!['[']);
    delimited(p, EOL_SET.add(T![']']), T![,], EXPR_FIRST, |p| {
        expr(p).is_some()
    });

    // while !p.at_ts(EOL_SET) && !p.at(T![']']) {
    //     if expr(p).is_none() {
    //         break;
    //     }
    //
    //     if !p.at(T![']']) && !p.expect(T![,]) {
    //         break;
    //     }
    // }
    p.expect(T![']']);

    m.complete(p, ARRAY_EXPR)
}

fn mapping_expr(p: &mut Parser<'_>) -> CompletedMarker {
    assert!(p.at(T!['{']));
    let m = p.start();

    p.bump(T!['{']);
    delimited(p, EOL_SET.add(T![']']), T![,], MAPPING_ENTRY_FIRST, |p| {
        mapping_entry(p).is_some()
    });

    p.expect(T!['}']);

    m.complete(p, MAPPING_EXPR)
}

const MAPPING_ENTRY_FIRST: TokenSet = TokenSet::new(&[INT_NUMBER]);

fn mapping_entry(p: &mut Parser<'_>) -> Option<CompletedMarker> {
    let m = p.start();
    if !p.at(INT_NUMBER) {
        m.abandon(p);
        return None;
    }

    p.bump(INT_NUMBER);
    p.expect(T![=>]);
    expr(p);

    Some(m.complete(p, MAPPING_ENTRY))
}
