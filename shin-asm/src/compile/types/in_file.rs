use either::Either;

use crate::compile::file::File;

/// `WithFile<T>` stores a value of `T` associated with a particular [`File`].
///
/// Typical usages are:
///
/// * `WithFile<SyntaxNode>` -- syntax node in a file
/// * `WithFile<ast::FnDef>` -- ast node in a file
/// * `WithFile<TextSize>` -- offset in a file
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct WithFile<T> {
    pub value: T,
    pub file: File,
}

impl<T> WithFile<T> {
    pub fn new(value: T, file: File) -> WithFile<T> {
        WithFile { value, file }
    }

    pub fn with_value<U>(&self, value: U) -> WithFile<U> {
        WithFile::new(value, self.file)
    }

    pub fn map<F: FnOnce(T) -> U, U>(self, f: F) -> WithFile<U> {
        WithFile::new(f(self.value), self.file)
    }

    pub fn as_ref(&self) -> WithFile<&T> {
        self.with_value(&self.value)
    }

    // pub fn file_syntax(&self, db: &dyn Db) -> &syntax::SourceFile {
    //     self.file.parse(db).syntax(db)
    // }
}

pub trait MakeWithFile: Sized {
    fn in_file(self, file: File) -> WithFile<Self> {
        WithFile::new(self, file)
    }
}

impl<N: crate::syntax::AstNode> MakeWithFile for crate::syntax::ptr::AstPtr<N> {}
impl MakeWithFile for text_size::TextRange {}

impl<T: Clone> WithFile<&T> {
    pub fn cloned(&self) -> WithFile<T> {
        self.with_value(self.value.clone())
    }
}

impl<T> WithFile<Option<T>> {
    pub fn transpose(self) -> Option<WithFile<T>> {
        Some(WithFile::new(self.value?, self.file))
    }
}

impl<L, R> WithFile<Either<L, R>> {
    pub fn transpose(self) -> Either<WithFile<L>, WithFile<R>> {
        match self.value {
            Either::Left(l) => Either::Left(WithFile::new(l, self.file)),
            Either::Right(r) => Either::Right(WithFile::new(r, self.file)),
        }
    }
}
