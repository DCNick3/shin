use super::db::Db;
use crate::syntax;

#[salsa::input]
pub struct File {
    #[id]
    pub path: String,
    #[return_ref]
    pub contents: String,
}

#[salsa::tracked]
impl File {
    #[salsa::tracked]
    pub fn emit_diagnostics(self, db: &dyn Db) {
        let parse = syntax::SourceFile::parse(self.contents(db));
        for error in parse.errors() {
            error.clone().into_diagnostic().in_file(self).emit(db)
        }
    }

    pub fn parse(self, db: &dyn Db) -> syntax::SourceFile {
        // NOTE: currently, we will not emit diagnostics if the file was never parsed
        // This is unlikely, but probably not very good design?
        self.emit_diagnostics(db);

        syntax::SourceFile::parse(self.contents(db)).tree()
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
