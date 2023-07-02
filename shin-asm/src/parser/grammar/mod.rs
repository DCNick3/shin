mod expressions;
mod items;

use crate::parser::{
    parser::{CompletedMarker, Marker, Parser},
    SyntaxKind::{self, *},
    TokenSet, T,
};

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
