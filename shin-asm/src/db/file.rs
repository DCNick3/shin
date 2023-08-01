use crate::db::diagnostics::Diagnostics;
use crate::db::Db;
use crate::syntax;
use crate::syntax::ast;

#[salsa::input]
pub struct File {
    #[id]
    pub path: String,
    #[return_ref]
    pub contents: String,
}

impl File {
    pub fn parse(self, db: &dyn Db) -> ParsedFile {
        let parse = syntax::SourceFile::parse(self.contents(db));

        for error in parse.errors() {
            Diagnostics::emit(db, self, error.clone());
        }

        ParsedFile::new(db, parse.tree())
    }
}

#[salsa::input]
pub struct Program {
    #[return_ref]
    pub files: Vec<File>,
    // TODO: program config (probably a toml)
}

impl Program {
    pub fn parse_files(self, db: &dyn Db) -> impl Iterator<Item = (File, ParsedFile)> + '_ {
        self.files(db)
            .iter()
            .copied()
            .map(|file| (file, file.parse(db)))
    }

    pub fn file_trees(self, db: &dyn Db) -> impl Iterator<Item = (File, &ast::SourceFile)> + '_ {
        self.parse_files(db)
            .map(|(file, parsed)| (file, parsed.syntax(db)))
    }
}

#[salsa::tracked]
pub struct ParsedFile {
    #[return_ref]
    pub syntax: syntax::SourceFile,
}
