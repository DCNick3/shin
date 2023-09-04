use super::db::Db;
use super::diagnostics::Diagnostics;
use crate::syntax;

#[salsa::input]
pub struct File {
    #[id]
    pub path: String,
    #[return_ref]
    pub contents: String,
}

impl File {
    pub fn parse(self, db: &dyn Db) -> syntax::SourceFile {
        let parse = syntax::SourceFile::parse(self.contents(db));

        for error in parse.errors() {
            Diagnostics::emit(db, self, error.clone());
        }

        parse.tree()
    }

    pub fn parse_debug_dump(self, db: &dyn Db) -> String {
        let parse = syntax::SourceFile::parse(self.contents(db));

        parse.debug_dump()
    }
}

#[salsa::input]
pub struct Program {
    #[return_ref]
    pub files: Vec<File>,
    // TODO: program config (probably a toml)
}

impl Program {
    pub fn parse_files(self, db: &dyn Db) -> impl Iterator<Item = (File, syntax::SourceFile)> + '_ {
        self.files(db)
            .iter()
            .copied()
            .map(|file| (file, file.parse(db)))
    }
}
