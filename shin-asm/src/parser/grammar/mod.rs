use crate::parser::{
    parser::Parser,
    SyntaxKind::{self, *},
};

pub(crate) fn source_file(p: &mut Parser<'_>) {
    let m = p.start();
    // items::mod_contents(p, false);
    m.complete(p, SOURCE_FILE);
}
