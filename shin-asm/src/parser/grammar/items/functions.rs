use super::*;

pub(super) const FUNCTION_OR_SUBROUTINE_START: TokenSet =
    TokenSet::new(&[T![function], T![subroutine]]);

pub(super) fn function_definition(p: &mut Parser<'_>) {
    assert_matches!(p.current(), T![function] | T![subroutine]);
    let m = p.start();

    let start_token = p.bump_ts(FUNCTION_OR_SUBROUTINE_START);
    let expected_end_token = match start_token {
        T![function] => T![endfun],
        T![subroutine] => T![endsub],
        _ => unreachable!(),
    };

    function_name(p, TokenSet::EMPTY); // TODO: figure out the recovery story

    if p.at(T!['(']) {
        if start_token == T![subroutine] {
            p.error("subroutines cannot have parameters");
        }
        function_definition_params(p);
    } else if start_token == T![function] {
        p.error("expected a parameter list");
    }

    if p.at(T!['[']) {
        function_definition_preserves(p);
    }

    if p.eat_ts(EOL_SET).is_none() {
        p.err_and_bump_over_many("expected a newline", EOL_SET);
    }

    instructions_block(p);

    if !p.eat(expected_end_token) {
        // TODO: maybe this error message is suboptimal
        p.err_and_bump(&format!("expected '{:?}'", expected_end_token));
    }

    // TODO: maybe have a helper function "expect_eol" or something
    if p.eat_ts(EOL_SET).is_none() {
        p.err_and_bump_over_many("expected end-of-line", EOL_SET)
    }

    m.complete(p, FUNCTION_DEFINITION);
}

fn function_name(p: &mut Parser<'_>, recovery: TokenSet) {
    if p.at(IDENT) {
        let m = p.start();
        p.bump(IDENT);
        m.complete(p, FUNCTION_NAME);
    } else {
        p.err_and_bump_unmatching("expected a name", recovery);
    }
}

fn function_definition_params(p: &mut Parser<'_>) {
    assert!(p.at(T!['(']));

    let m = p.start();

    p.bump(T!['(']);

    // if p.at(REGISTER_IDENT) {
    delimited(
        p,
        EOL_SET.add(T![')']),
        T![,],
        TokenSet::new(&[REGISTER_IDENT]),
        |p: &mut Parser<'_>| p.expect(REGISTER_IDENT),
    );
    // }

    p.expect(T![')']);

    m.complete(p, FUNCTION_DEFINITION_PARAMS);
}

fn function_definition_preserves(p: &mut Parser<'_>) {
    assert!(p.at(T!['[']));

    let m = p.start();

    p.bump(T!['[']);

    delimited(
        p,
        EOL_SET.add(T![']']),
        T![,],
        TokenSet::new(&[REGISTER_IDENT]),
        register_range_opt,
    );

    p.expect(T![']']);

    m.complete(p, FUNCTION_DEFINITION_PRESERVES);
}

fn register_range_opt(p: &mut Parser<'_>) -> bool {
    let m = p.start();

    if !p.at(REGISTER_IDENT) {
        return false;
    }

    p.bump(REGISTER_IDENT);

    if p.eat(T![-]) {
        p.expect(REGISTER_IDENT);
    }

    m.complete(p, REGISTER_RANGE);

    true
}