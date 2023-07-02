use crate::parser::output::Step;
use crate::parser::{
    Input, LexedStr, Output,
    SyntaxKind::{self, *},
};
use std::mem;

#[derive(Debug)]
pub enum StrStep<'a> {
    Token { kind: SyntaxKind, text: &'a str },
    Enter { kind: SyntaxKind },
    Exit,
    Error { msg: &'a str, pos: usize },
}

impl<'a> LexedStr<'a> {
    pub fn to_input(&self) -> Input {
        let mut res = Input::default();
        for i in 0..self.len() {
            let kind = self.kind(i);

            // don't include trivia tokens
            if kind.is_trivia() {
                continue;
            }

            res.push(kind);
        }
        res
    }

    pub fn intersperse_trivia(&self, output: &Output, sink: &mut dyn FnMut(StrStep<'_>)) -> bool {
        let mut builder = Builder {
            lexed: self,
            pos: 0,
            state: State::PendingEnter,
            sink,
        };

        for event in output.iter() {
            match event {
                Step::Token { kind } => builder.token(kind),
                Step::Enter { kind } => builder.enter(kind),
                Step::Exit => builder.exit(),
                Step::Error { msg } => {
                    let text_pos = builder.lexed.text_start(builder.pos);
                    (builder.sink)(StrStep::Error { msg, pos: text_pos });
                }
            }
        }

        match mem::replace(&mut builder.state, State::Normal) {
            State::PendingExit => {
                builder.eat_trivias();
                (builder.sink)(StrStep::Exit);
            }
            State::PendingEnter | State::Normal => unreachable!(),
        }

        // is_eof?
        builder.pos == builder.lexed.len()
    }
}

struct Builder<'a, 'b> {
    lexed: &'a LexedStr<'a>,
    pos: usize,
    state: State,
    sink: &'b mut dyn FnMut(StrStep<'_>),
}

enum State {
    PendingEnter,
    Normal,
    PendingExit,
}

impl Builder<'_, '_> {
    fn token(&mut self, kind: SyntaxKind) {
        match mem::replace(&mut self.state, State::Normal) {
            State::PendingEnter => unreachable!(),
            State::PendingExit => (self.sink)(StrStep::Exit),
            State::Normal => (),
        }
        self.eat_trivias();
        self.do_token(kind);
    }

    fn enter(&mut self, kind: SyntaxKind) {
        match mem::replace(&mut self.state, State::Normal) {
            State::PendingEnter => {
                (self.sink)(StrStep::Enter { kind });
                // No need to attach trivias to previous node: there is no
                // previous node.
                return;
            }
            State::PendingExit => (self.sink)(StrStep::Exit),
            State::Normal => (),
        }

        let n_trivias = (self.pos..self.lexed.len())
            .take_while(|&it| self.lexed.kind(it).is_trivia())
            .count();
        let leading_trivias = self.pos..self.pos + n_trivias;
        let n_attached_trivias = n_attached_trivias(
            kind,
            leading_trivias
                .rev()
                .map(|it| (self.lexed.kind(it), self.lexed.text(it))),
        );
        self.eat_n_trivias(n_trivias - n_attached_trivias);
        (self.sink)(StrStep::Enter { kind });
        self.eat_n_trivias(n_attached_trivias);
    }

    fn exit(&mut self) {
        match mem::replace(&mut self.state, State::PendingExit) {
            State::PendingEnter => unreachable!(),
            State::PendingExit => (self.sink)(StrStep::Exit),
            State::Normal => (),
        }
    }

    fn eat_trivias(&mut self) {
        while self.pos < self.lexed.len() {
            let kind = self.lexed.kind(self.pos);
            if !kind.is_trivia() {
                break;
            }
            self.do_token(kind);
        }
    }

    fn eat_n_trivias(&mut self, n: usize) {
        for _ in 0..n {
            let kind = self.lexed.kind(self.pos);
            assert!(kind.is_trivia());
            self.do_token(kind);
        }
    }

    fn do_token(&mut self, kind: SyntaxKind) {
        let text = &self.lexed.range_text(self.pos..self.pos + 1);
        self.pos += 1;
        (self.sink)(StrStep::Token { kind, text });
    }
}

fn n_attached_trivias<'a>(
    _kind: SyntaxKind,
    _trivias: impl Iterator<Item = (SyntaxKind, &'a str)>,
) -> usize {
    // TODO: I _think_ this function handles attachment of trivia tokens to some constructs
    // we don't have those and, at least for now, don't handle whitespace really
    // maybe later it will become clear what to do with this
    0
    // match kind {
    //     CONST | ENUM | FN | IMPL | MACRO_CALL | MACRO_DEF | MACRO_RULES | MODULE | RECORD_FIELD
    //     | STATIC | STRUCT | TRAIT | TUPLE_FIELD | TYPE_ALIAS | UNION | USE | VARIANT => {
    //         let mut res = 0;
    //         let mut trivias = trivias.enumerate().peekable();
    //
    //         while let Some((i, (kind, text))) = trivias.next() {
    //             match kind {
    //                 WHITESPACE if text.contains("\n\n") => {
    //                     // we check whether the next token is a doc-comment
    //                     // and skip the whitespace in this case
    //                     if let Some((COMMENT, peek_text)) = trivias.peek().map(|(_, pair)| pair) {
    //                         if is_outer(peek_text) {
    //                             continue;
    //                         }
    //                     }
    //                     break;
    //                 }
    //                 COMMENT => {
    //                     if is_inner(text) {
    //                         break;
    //                     }
    //                     res = i + 1;
    //                 }
    //                 _ => (),
    //             }
    //         }
    //         res
    //     }
    //     _ => 0,
    // }
}
