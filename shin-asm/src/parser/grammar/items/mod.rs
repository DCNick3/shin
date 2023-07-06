mod functions;

use super::*;

pub(super) fn item(p: &mut Parser<'_>) {
    if p.at_ts(functions::FUNCTION_OR_SUBROUTINE_START) {
        functions::function_definition(p);
    } else if p.at(IDENT) {
        instructions_block(p);
    } else if p.at_ts(EOL_SET) {
        p.bump_any();
        // empty items are allowed
    } else {
        p.err_and_bump("expected an instruction or label");
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

    let m_name = p.start();
    p.bump(IDENT);
    m_name.complete(p, INSTRUCTION_NAME);

    if p.at_ts(expressions::EXPR_FIRST) {
        instr_arg_list(p);
    }
    if p.at_ts(EOL_SET) {
        p.eat(NEWLINE);
    } else {
        p.err_and_bump_over_many("expected an instruction or label", EOL_SET)
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
