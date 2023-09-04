use super::*;

pub(super) fn instructions_block_set(p: &mut Parser<'_>) {
    assert!(p.at(IDENT));

    let m = p.start();

    while p.at(IDENT) {
        instructions_block(p);
        while p.eat(NEWLINE) {}
    }

    m.complete(p, INSTRUCTIONS_BLOCK_SET);
}

fn instructions_block(p: &mut Parser<'_>) {
    assert!(p.at(IDENT));
    let m = p.start();

    labels(p);
    body(p);

    m.complete(p, INSTRUCTIONS_BLOCK);
}

fn labels(p: &mut Parser<'_>) {
    let m = p.start();

    let mut have_label = false;
    while p.at(IDENT) && p.nth_at(1, T![:]) {
        label(p);
        have_label = true;
        while p.eat(NEWLINE) {}
    }

    if have_label {
        m.complete(p, INSTRUCTIONS_BLOCK_LABELS);
    } else {
        m.abandon(p);
    }
}

fn label(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump(IDENT);
    p.bump(T![:]);

    m.complete(p, LABEL);
}

fn body(p: &mut Parser<'_>) {
    let m = p.start();

    let mut have_instruction = false;
    while p.at(IDENT) && !p.nth_at(1, T![:]) {
        instruction(p);
        have_instruction = true;
        while p.eat(NEWLINE) {}
    }

    if have_instruction {
        m.complete(p, INSTRUCTIONS_BLOCK_BODY);
    } else {
        m.abandon(p);
    }
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
