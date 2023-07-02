//! This module provides a way to construct a `File`.
//! It is intended to be completely decoupled from the
//! parser, so as to allow to evolve the tree representation
//! and the parser algorithm independently.
//!
//! The `TreeSink` trait is the bridge between the parser and the
//! tree builder: the parser produces a stream of events like
//! `start node`, `finish node`, and `FileBuilder` converts
//! this stream to a real tree.
use std::mem;

use crate::parser::{
    output::Output,
    SyntaxKind::{self, *},
};

/// `Parser` produces a flat list of `Event`s.
/// They are converted to a tree-structure in
/// a separate pass, via `TreeBuilder`.
#[derive(Debug)]
pub(crate) enum Event {
    /// This event signifies the start of the node.
    /// It should be either abandoned (in which case the
    /// `kind` is `TOMBSTONE`, and the event is ignored),
    /// or completed via a `Finish` event.
    ///
    /// All tokens between a `Start` and a `Finish` would
    /// become the children of the respective node.
    ///
    Start(SyntaxKind),

    /// Complete the previous `Start` event
    Finish,

    /// Produce a single leaf-element.
    /// `n_raw_tokens` is used to glue complex contextual tokens.
    /// For example, lexer tokenizes `>>` as `>`, `>`, and
    /// `n_raw_tokens = 2` is used to produced a single `>>`.
    Token(SyntaxKind),
    Error {
        msg: String,
    },
}

impl Event {
    pub(crate) fn tombstone() -> Self {
        Event::Start(TOMBSTONE)
    }
}

/// Generate the syntax tree with the control of events.
pub(super) fn process(mut events: Vec<Event>) -> Output {
    let mut res = Output::default();

    for i in 0..events.len() {
        match mem::replace(&mut events[i], Event::tombstone()) {
            Event::Start(kind) => {
                // For events[A, B, C], B is A's forward_parent, C is B's forward_parent,
                // in the normal control flow, the parent-child relation: `A -> B -> C`,
                // while with the magic forward_parent, it writes: `C <- B <- A`.

                if kind != TOMBSTONE {
                    res.enter_node(kind);
                }
            }
            Event::Finish => res.leave_node(),
            Event::Token(kind) => {
                res.token(kind);
            }
            Event::Error { msg } => res.error(msg),
        }
    }

    res
}
