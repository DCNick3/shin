use crate::compile::file::File;
use either::Either;

/// `InFile<T>` stores a value of `T` inside a particular file/syntax tree.
///
/// Typical usages are:
///
/// * `InFile<SyntaxNode>` -- syntax node in a file
/// * `InFile<ast::FnDef>` -- ast node in a file
/// * `InFile<TextSize>` -- offset in a file
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct InFile<T> {
    pub value: T,
    pub file: File,
}

impl<T> InFile<T> {
    pub fn new(value: T, file: File) -> InFile<T> {
        InFile { value, file }
    }

    pub fn with_value<U>(&self, value: U) -> InFile<U> {
        InFile::new(value, self.file)
    }

    pub fn map<F: FnOnce(T) -> U, U>(self, f: F) -> InFile<U> {
        InFile::new(f(self.value), self.file)
    }

    pub fn as_ref(&self) -> InFile<&T> {
        self.with_value(&self.value)
    }

    // pub fn file_syntax(&self, db: &dyn Db) -> &syntax::SourceFile {
    //     self.file.parse(db).syntax(db)
    // }
}

pub trait MakeInFile: Sized {
    fn in_file(self, file: File) -> InFile<Self> {
        InFile::new(self, file)
    }
}

impl<N: crate::syntax::AstNode> MakeInFile for crate::syntax::ptr::AstPtr<N> {}
impl MakeInFile for text_size::TextRange {}

impl<T: Clone> InFile<&T> {
    pub fn cloned(&self) -> InFile<T> {
        self.with_value(self.value.clone())
    }
}

impl<T> InFile<Option<T>> {
    pub fn transpose(self) -> Option<InFile<T>> {
        Some(InFile::new(self.value?, self.file))
    }
}

impl<L, R> InFile<Either<L, R>> {
    pub fn transpose(self) -> Either<InFile<L>, InFile<R>> {
        match self.value {
            Either::Left(l) => Either::Left(InFile::new(l, self.file)),
            Either::Right(r) => Either::Right(InFile::new(r, self.file)),
        }
    }
}
