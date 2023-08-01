use crate::db::file::File;
use crate::db::in_file::InFile;
use crate::db::Db;

use std::sync::Arc;

#[salsa::accumulator]
pub struct Diagnostics(InFile<Arc<miette::Report>>);

impl Diagnostics {
    pub fn emit(
        db: &dyn Db,
        file: File,
        diagnostic: impl miette::Diagnostic + Send + Sync + 'static,
    ) {
        Self::push(
            db,
            InFile::new(file, Arc::new(miette::Report::new(diagnostic))),
        )
    }
}
