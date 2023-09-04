use super::*;

pub(super) fn instructions_block(p: &mut Parser<'_>) {
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

    // optionally eat a newline
    p.eat(NEWLINE);

    m.complete(p, LABEL);
}

fn body(p: &mut Parser<'_>) {
    let m = p.start();

    let mut have_instruction = false;
    while p.at(IDENT) && !p.nth_at(1, T![:]) {
        instruction(p);
        have_instruction = true;
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
