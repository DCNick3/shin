use super::*;

fn register_or_normal_name_def(p: &mut Parser) {
    if p.at(REGISTER_IDENT) {
        register_name_def(p);
    } else {
        name_def_r(p, TokenSet::EMPTY);
    }
}

pub(super) fn alias_definition(p: &mut Parser) {
    assert!(p.at(T![def]));

    let m = p.start();

    p.bump(T![def]);

    register_or_normal_name_def(p);

    if p.expect(T![=]) {
        expressions::expr(p);
    }

    newline(p);

    m.complete(p, ALIAS_DEFINITION);
}
