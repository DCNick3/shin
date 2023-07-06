mod alias;
mod functions;
mod instructions;

use super::*;

pub(super) fn item(p: &mut Parser<'_>) {
    if p.at_ts(functions::FUNCTION_OR_SUBROUTINE_START) {
        functions::function_definition(p);
    } else if p.at(IDENT) {
        instructions::instructions_block(p);
    } else if p.at(DEF_KW) {
        alias::alias_definition(p);
    } else if p.at_ts(EOL_SET) {
        p.bump_any();
        // empty items are allowed
    } else {
        p.err_and_bump("expected an instruction or label");
    }
}
