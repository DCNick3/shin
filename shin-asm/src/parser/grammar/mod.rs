mod expressions;
mod items;

use crate::parser::{
    parser::{CompletedMarker, Marker, Parser},
    SyntaxKind::{self, *},
    TokenSet, T,
};
use assert_matches::assert_matches;

const EOL_SET: TokenSet = TokenSet::new(&[NEWLINE, EOF]);

pub(crate) fn source_file(p: &mut Parser<'_>) {
    let m = p.start();
    while !p.at(EOF) {
        items::item(p);
    }

    // items::mod_contents(p, false);
    m.complete(p, SOURCE_FILE);
}

/// The `parser` passed this is required to at least consume one token if it returns `true`.
/// If the `parser` returns false, parsing will stop.
fn delimited(
    p: &mut Parser<'_>,
    stop_set: TokenSet,
    delim: SyntaxKind,
    first_set: TokenSet,
    mut parser: impl FnMut(&mut Parser<'_>) -> bool,
) {
    while !p.at_ts(stop_set) && !p.at(EOF) {
        if !parser(p) {
            break;
        }
        if !p.at(delim) {
            if p.at_ts(first_set) {
                p.error(format!("expected {:?}", delim));
            } else {
                break;
            }
        } else {
            p.bump(delim);
        }
    }
}

fn name_def_r(p: &mut Parser<'_>, recovery: TokenSet) {
    if p.at(IDENT) {
        let m = p.start();
        p.bump(IDENT);
        m.complete(p, NAME_DEF);
    } else {
        p.err_and_bump_unmatching("expected a name", recovery);
    }
}

fn register_name_def(p: &mut Parser<'_>) {
    if p.at(REGISTER_IDENT) {
        let m = p.start();
        p.bump(REGISTER_IDENT);
        m.complete(p, REGISTER_NAME_DEF);
    } else {
        p.err_and_bump("expected a register name");
    }
}

fn newline(p: &mut Parser<'_>) {
    if p.eat_ts(EOL_SET).is_none() {
        p.err_and_bump_over_many("expected a newline", EOL_SET);
    }
}
