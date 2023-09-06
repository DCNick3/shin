use crate::compile::{Db, File, InFile};
use std::fmt;
use std::fmt::{Debug, Display};

use miette::NamedSource;
use std::sync::Arc;

// TODO: we need to support diagnostics with source maps represented in hir node ids
#[salsa::accumulator]
pub struct Diagnostics(InFile<Arc<miette::Report>>);

struct DiagnosticWithSourceCode {
    error: Arc<miette::Report>,
    source_code: NamedSource,
}

impl Debug for DiagnosticWithSourceCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.error, f)
    }
}

impl Display for DiagnosticWithSourceCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.error, f)
    }
}

impl std::error::Error for DiagnosticWithSourceCode {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.error.source()
    }
}

impl miette::Diagnostic for DiagnosticWithSourceCode {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.error.code()
    }

    fn severity(&self) -> Option<miette::Severity> {
        self.error.severity()
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.error.help()
    }

    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.error.url()
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.source_code)
    }

    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        self.error.labels()
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
        self.error.related()
    }

    fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
        self.error.diagnostic_source()
    }
}

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

    pub fn emit_for(
        db: &dyn Db,
        file: File,
        diagnostic: impl miette::Diagnostic + Send + Sync + 'static,
    ) {
        Self::push(
            db,
            InFile::new(file, Arc::new(miette::Report::new(diagnostic))),
        )
    }

    pub fn with_source(
        db: &dyn Db,
        diagnostics: Vec<InFile<Arc<miette::Report>>>,
    ) -> Vec<miette::Report> {
        diagnostics
            .into_iter()
            .map(|diag| {
                let path = diag.file.path(db);
                let source_code = diag.file.contents(db).clone();

                miette::Report::new(DiagnosticWithSourceCode {
                    error: diag.value.clone(),
                    source_code: NamedSource::new(path, source_code),
                })
            })
            .collect()
    }

    pub fn debug_dump(db: &dyn Db, diagnostics: Vec<InFile<Arc<miette::Report>>>) -> String {
        use std::fmt::Write as _;

        let mut errors = String::new();
        for diagnostic in Diagnostics::with_source(db, diagnostics) {
            writeln!(errors, "{:?}", diagnostic).unwrap();
        }
        errors
    }
}
