use super::*;

pub(super) fn item(p: &mut Parser<'_>) {
    if p.at(T![function]) {
        todo!()
    } else if p.at(T![subroutine]) {
        todo!()
    } else if p.at(IDENT) {
        instructions_block(p);
    } else if p.at_ts(EOL_SET) {
        p.bump_any();
        // empty items are allowed
    } else {
        p.err_recover("expected an instruction or label", EOL_SET);
    }
}

fn instructions_block(p: &mut Parser<'_>) {
    let m = p.start();

    while p.at(IDENT) {
        instruction_or_label(p);
    }

    m.complete(p, INSTRUCTIONS_BLOCK);
}

fn instruction_or_label(p: &mut Parser<'_>) {
    assert!(p.at(IDENT));

    if p.nth_at(1, T![:]) {
        label(p);
    } else {
        instruction(p);
    }
}

fn label(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump(IDENT);
    p.bump(T![:]);

    // optionally eat a newline
    p.eat(NEWLINE);

    m.complete(p, LABEL);
}

fn instruction(p: &mut Parser<'_>) {
    assert!(!p.nth_at(1, T![:]));

    let m = p.start();
    p.bump(IDENT);

    if p.at_ts(expressions::EXPR_FIRST) {
        instr_arg_list(p);
    } else if p.at_ts(EOL_SET) {
        p.eat(NEWLINE);
    } else {
        // TODO: we definitely need to change the `err_recover` to fast-forward to the next line or smth
        p.err_recover("expected an instruction or label", EOL_SET)
    }

    m.complete(p, INSTRUCTION);
}

fn instr_arg_list(p: &mut Parser<'_>) {
    let m = p.start();

    while !p.at_ts(EOL_SET) {
        delimited(
            p,
            EOL_SET,
            T![,],
            expressions::EXPR_FIRST,
            |p: &mut Parser<'_>| expressions::expr(p).is_some(),
        );
    }

    m.complete(p, INSTR_ARG_LIST);
}
