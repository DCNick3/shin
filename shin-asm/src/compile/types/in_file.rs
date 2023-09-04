use crate::compile::file::File;
use either::Either;

/// `InFile<T>` stores a value of `T` inside a particular file/syntax tree.
///
/// Typical usages are:
///
/// * `InFile<SyntaxNode>` -- syntax node in a file
/// * `InFile<ast::FnDef>` -- ast node in a file
/// * `InFile<TextSize>` -- offset in a file
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct InFile<T> {
    pub file: File,
    pub value: T,
}

impl<T> InFile<T> {
    pub fn new(file: File, value: T) -> InFile<T> {
        InFile { file, value }
    }

    pub fn with_value<U>(&self, value: U) -> InFile<U> {
        InFile::new(self.file, value)
    }

    pub fn map<F: FnOnce(T) -> U, U>(self, f: F) -> InFile<U> {
        InFile::new(self.file, f(self.value))
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
        InFile::new(file, self)
    }
}

impl<N: crate::syntax::AstNode> MakeInFile for crate::syntax::ptr::AstPtr<N> {}

impl<T: Clone> InFile<&T> {
    pub fn cloned(&self) -> InFile<T> {
        self.with_value(self.value.clone())
    }
}

impl<T> InFile<Option<T>> {
    pub fn transpose(self) -> Option<InFile<T>> {
        let value = self.value?;
        Some(InFile::new(self.file, value))
    }
}

impl<L, R> InFile<Either<L, R>> {
    pub fn transpose(self) -> Either<InFile<L>, InFile<R>> {
        match self.value {
            Either::Left(l) => Either::Left(InFile::new(self.file, l)),
            Either::Right(r) => Either::Right(InFile::new(self.file, r)),
        }
    }
}
