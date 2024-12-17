//! In shin-asm, syntax trees are transient objects.
//!
//! That means that we create trees when we need them, and tear them down to
//! save memory. In this architecture, hanging on to a particular syntax node
//! for a long time is ill-advisable, as that keeps the whole tree resident.
//!
//! Instead, we provide a [`SyntaxNodePtr`] type, which stores information about
//! *location* of a particular syntax node in a tree. Its a small type which can
//! be cheaply stored, and which can be resolved to a real [`SyntaxNode`] when
//! necessary.

use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use rowan::TextRange;

use crate::syntax::{syntax_node::SalLanguage, AstNode, AstSpanned, SyntaxNode};

/// A "pointer" to a [`SyntaxNode`], via location in the source code.
pub type SyntaxNodePtr = rowan::ast::SyntaxNodePtr<SalLanguage>;

/// Like `SyntaxNodePtr`, but remembers the type of node.
#[derive(Debug)]
pub struct AstPtr<N: AstNode> {
    raw: SyntaxNodePtr,
    _ty: PhantomData<fn() -> N>,
}

impl<N: AstNode> Clone for AstPtr<N> {
    fn clone(&self) -> AstPtr<N> {
        AstPtr {
            raw: self.raw,
            _ty: PhantomData,
        }
    }
}

impl<N: AstNode> Eq for AstPtr<N> {}

impl<N: AstNode> PartialEq for AstPtr<N> {
    fn eq(&self, other: &AstPtr<N>) -> bool {
        self.raw == other.raw
    }
}

impl<N: AstNode> Hash for AstPtr<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<N: AstNode> AstPtr<N> {
    pub fn new(node: &N) -> AstPtr<N> {
        AstPtr {
            raw: SyntaxNodePtr::new(node.syntax()),
            _ty: PhantomData,
        }
    }

    pub fn to_node(&self, root: &SyntaxNode) -> N {
        let syntax_node = self.raw.to_node(root);
        N::cast(syntax_node).unwrap()
    }

    pub fn syntax_node_ptr(&self) -> SyntaxNodePtr {
        self.raw
    }

    pub fn cast<U: AstNode>(self) -> Option<AstPtr<U>> {
        if !U::can_cast(self.raw.kind()) {
            return None;
        }
        Some(AstPtr {
            raw: self.raw,
            _ty: PhantomData,
        })
    }

    pub fn upcast<M: AstNode>(self) -> AstPtr<M>
    where
        N: Into<M>,
    {
        AstPtr {
            raw: self.raw,
            _ty: PhantomData,
        }
    }

    /// Like `SyntaxNodePtr::cast` but the trait bounds work out.
    pub fn try_from_raw(raw: SyntaxNodePtr) -> Option<AstPtr<N>> {
        N::can_cast(raw.kind()).then_some(AstPtr {
            raw,
            _ty: PhantomData,
        })
    }
}

impl<N: AstNode> AstSpanned for AstPtr<N> {
    fn text_range(&self) -> TextRange {
        self.raw.text_range()
    }
}

impl<N: AstNode> From<AstPtr<N>> for SyntaxNodePtr {
    fn from(ptr: AstPtr<N>) -> SyntaxNodePtr {
        ptr.raw
    }
}

#[test]
fn test_local_syntax_ptr() {
    use crate::syntax::{ast, AstNode, SourceFile};

    let file = SourceFile::parse("function ABOBA()\nEXIT 0,0\nendfun")
        .ok()
        .unwrap();
    let field = file
        .syntax()
        .descendants()
        .find_map(ast::Instruction::cast)
        .unwrap();
    let ptr = SyntaxNodePtr::new(field.syntax());
    let field_syntax = ptr.to_node(file.syntax());
    assert_eq!(field.syntax(), &field_syntax);
}
