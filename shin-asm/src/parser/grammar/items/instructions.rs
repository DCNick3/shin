use super::*;

pub(super) fn instructions_block(p: &mut Parser<'_>) {
    let m = p.start();

    if p.at(IDENT) {
        instruction_or_label(p);
    }

    while p.at(IDENT) && !p.nth_at(1, T![:]) {
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

    let m_name = p.start();
    p.bump(IDENT);
    m_name.complete(p, INSTRUCTION_NAME);

    if p.at_ts(expressions::EXPR_FIRST) {
        instr_arg_list(p);
    }
    newline(p);

    m.complete(p, INSTRUCTION);
}

fn instr_arg_list(p: &mut Parser<'_>) {
    let m = p.start();

    delimited(
        p,
        EOL_SET,
        T![,],
        expressions::EXPR_FIRST,
        |p: &mut Parser<'_>| expressions::expr(p).is_some(),
    );

    m.complete(p, INSTR_ARG_LIST);
}
