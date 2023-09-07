mod nodes;
mod tokens;
pub mod visit;

use std::marker::PhantomData;

use super::{
    syntax_node::{SyntaxNode, SyntaxNodeChildren, SyntaxToken},
    SyntaxKind,
};
use either::Either;

pub use shin_derive::{AstNode, AstToken};

use crate::syntax::ptr::AstPtr;
pub use nodes::*;
pub use tokens::*;

pub trait AstSpanned {
    fn text_range(&self) -> crate::syntax::TextRange;

    fn span(&self, file: crate::compile::File) -> crate::compile::diagnostics::Span {
        crate::compile::diagnostics::Span::new(file, self.text_range())
    }
}

/// The main trait to go from untyped `SyntaxNode`  to a typed ast. The
/// conversion itself has zero runtime cost: ast and syntax nodes have exactly
/// the same representation: a pointer to the tree root and a pointer to the
/// node itself.
pub trait AstNode: AstSpanned {
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxNode;
    fn clone_for_update(&self) -> Self
    where
        Self: Sized,
    {
        Self::cast(self.syntax().clone_for_update()).unwrap()
    }
    fn clone_subtree(&self) -> Self
    where
        Self: Sized,
    {
        Self::cast(self.syntax().clone_subtree()).unwrap()
    }
}

pub trait AstNodeExt: AstNode + Sized {
    fn ptr(&self) -> AstPtr<Self> {
        AstPtr::new(self)
    }
}

impl<T: AstNode> AstNodeExt for T {}

/// Like `AstNode`, but wraps tokens rather than interior nodes.
pub trait AstToken: AstSpanned {
    fn can_cast(token: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxToken) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxToken;

    fn text(&self) -> &str {
        self.syntax().text()
    }
}

/// An iterator over `SyntaxNode` children of a particular AST type.
#[derive(Debug, Clone)]
pub struct AstChildren<N> {
    inner: SyntaxNodeChildren,
    ph: PhantomData<N>,
}

impl<N> AstChildren<N> {
    fn new(parent: &SyntaxNode) -> Self {
        AstChildren {
            inner: parent.children(),
            ph: PhantomData,
        }
    }
}

impl<N: AstNode> Iterator for AstChildren<N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        self.inner.find_map(N::cast)
    }
}

impl<L, R> AstSpanned for Either<L, R>
where
    L: AstSpanned,
    R: AstSpanned,
{
    fn text_range(&self) -> crate::syntax::TextRange {
        self.as_ref().either(L::text_range, R::text_range)
    }
}

impl<L, R> AstNode for Either<L, R>
where
    L: AstNode,
    R: AstNode,
{
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        L::can_cast(kind) || R::can_cast(kind)
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if L::can_cast(syntax.kind()) {
            L::cast(syntax).map(Either::Left)
        } else {
            R::cast(syntax).map(Either::Right)
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        self.as_ref().either(L::syntax, R::syntax)
    }
}

mod support {
    use super::{AstChildren, AstNode, AstToken, SyntaxNode};

    pub(super) fn child<N: AstNode>(parent: &SyntaxNode) -> Option<N> {
        parent.children().find_map(N::cast)
    }

    pub(super) fn children<N: AstNode>(parent: &SyntaxNode) -> AstChildren<N> {
        AstChildren::new(parent)
    }

    pub(super) fn token<T: AstToken>(parent: &SyntaxNode) -> Option<T> {
        parent
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find_map(|it| T::cast(it))
    }
}

#[test]
fn assert_ast_is_object_safe() {
    fn _f(_: &dyn AstNode, _: &dyn AstToken) {}
}

#[test]
#[cfg(test)]
fn test_exit_parses() {
    let file = SourceFile::parse("// pls ignore\nEXIT 0, 1");

    eprintln!("{}", file.debug_dump());

    let item = file.tree().items().next().unwrap();
    let Item::InstructionsBlockSet(blocks) = item else {
        panic!("Expected InstructionsBlock, got {:?}", item);
    };
    let block = blocks.blocks().next().unwrap();
    let instruction = block.body().unwrap().instructions().next().unwrap();
    let name = instruction.name().unwrap();
    let args = instruction.args().unwrap();

    assert_eq!(name.to_string(), "EXIT");
    assert_eq!(args.to_string(), "0, 1");

    let args = args.args().collect::<Vec<_>>();
    let [arg1, arg2] = args.as_slice() else {
        panic!("Expected 2 args, got {:?}", args);
    };

    assert_eq!(arg1.to_string(), "0");
    assert_eq!(arg2.to_string(), "1");
}
