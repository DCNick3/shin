mod event;
mod grammar;
mod input;
mod lang;
mod lex;
mod output;
mod parser;
mod shortcuts;
mod syntax_kind;
mod token_set;

#[cfg(test)]
mod tests;

pub use input::Input;
pub use lex::LexedStr;
pub use output::Output;
use output::Step;
pub use shortcuts::StrStep;
pub use syntax_kind::SyntaxKind;
pub(crate) use syntax_kind::T;
pub(crate) use token_set::TokenSet;

pub fn parse(input: &Input) -> Output {
    let mut p = parser::Parser::new(input);
    grammar::source_file(&mut p);
    let events = p.finish();
    let res = event::process(events);

    if cfg!(debug_assertions) {
        let mut depth = 0;
        let mut first = true;
        for step in res.iter() {
            assert!(depth > 0 || first);
            first = false;
            match step {
                Step::Enter { .. } => depth += 1,
                Step::Exit => depth -= 1,
                Step::Token { .. } | Step::Error { .. } => (),
            }
        }
        assert!(!first, "no tree at all");
        assert_eq!(depth, 0, "unbalanced tree");
    }

    res
}
