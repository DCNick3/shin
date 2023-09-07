use crate::compile::from_hir::HirIdInBlock;
use crate::compile::{Db, File, InFile, MakeInFile};

use std::fmt::Debug;

use ariadne::Span as _;
use text_size::TextRange;

#[derive(Debug, Copy, Clone)]
pub struct Span(InFile<TextRange>);

impl Span {
    pub fn new(file: File, range: TextRange) -> Self {
        Self(range.in_file(file))
    }
}

impl ariadne::Span for Span {
    type SourceId = File;

    fn source(&self) -> &Self::SourceId {
        &self.0.file
    }

    fn start(&self) -> usize {
        self.0.value.start().into()
    }

    fn end(&self) -> usize {
        self.0.value.end().into()
    }
}

trait DiagnosticLocation: Debug + Copy + 'static {
    type Context<'a>: Copy;

    fn span(&self, context: Self::Context<'_>) -> Span;
}

/// A location specified by a range in the file. Does not include the file id. Needs to be enriched with the file id before being emitted.
#[derive(Debug, Copy, Clone)]
pub struct FileLocation(pub TextRange);

impl FileLocation {
    fn in_file(self, file: File) -> SourceLocation {
        SourceLocation(Span::new(file, self.0))
    }
}

/// A location specified by a text range and a file id.
#[derive(Debug, Copy, Clone)]
pub struct SourceLocation(pub Span);

/// A location specified by a HIR node range and a file id. Diagnostic machinery will use the HIR source map to get the actual source range.
#[derive(Debug, Copy, Clone)]
pub struct HirLocation(pub InFile<(HirIdInBlock, HirIdInBlock)>);

impl HirLocation {
    pub fn single_node(hir_id_in_block: HirIdInBlock, file: File) -> Self {
        Self(InFile::new((hir_id_in_block, hir_id_in_block), file))
    }
}

impl DiagnosticLocation for SourceLocation {
    type Context<'a> = ();
    fn span(&self, _: Self::Context<'_>) -> Span {
        self.0
    }
}
impl DiagnosticLocation for HirLocation {
    type Context<'a> = &'a dyn Db;

    fn span(&self, _db: Self::Context<'_>) -> Span {
        let InFile {
            file: _file,
            value: (_start_node, _end_node),
        } = self.0;

        todo!("Collect HIR source maps and use them to get the location")
    }
}

pub trait DiagnosticClone<L> {
    fn clone_box(&self) -> Box<dyn Diagnostic<L>>;
}

pub trait Diagnostic<L>: Debug + DiagnosticClone<L> + 'static {
    fn message(&self) -> String;
    fn location(&self) -> L;
    fn additional_labels(&self) -> Vec<(String, L)>;
}

#[derive(Debug, Clone)]
pub struct SimpleDiagnostic<L> {
    message: String,
    location: L,
}

impl<L> SimpleDiagnostic<L> {
    pub fn new(message: String, location: L) -> Self {
        Self { message, location }
    }
}

macro_rules! make_diagnostic {
    ($token:expr => $file:expr, $($fmt:expr),+) => {
        {
            use $crate::syntax::ast::AstSpanned as _;
            $crate::compile::diagnostics::SimpleDiagnostic::new(
                format!($($fmt),+),
                $crate::compile::diagnostics::SourceLocation(
                    $token.span($file)
                )
            )
        }
    };
    ($span:expr, $($fmt:expr),+) => {
        $crate::compile::diagnostics::SimpleDiagnostic::new(format!($($fmt),+), $span)
    };
}
pub(crate) use make_diagnostic;
macro_rules! emit_diagnostic {
    ($db:expr, $($args:tt)*) => {
        {
            #[allow(unused_imports)]
            use $crate::compile::diagnostics::{FileDiagnosticExt as _, SourceDiagnosticExt as _};
            $crate::compile::diagnostics::make_diagnostic!($($args)*).emit($db)
        }
    };
}
pub(crate) use emit_diagnostic;

impl<L: Debug + Copy + 'static> DiagnosticClone<L> for SimpleDiagnostic<L> {
    fn clone_box(&self) -> Box<dyn Diagnostic<L>> {
        Box::new(self.clone())
    }
}

impl<L: Debug + Copy + 'static> Diagnostic<L> for SimpleDiagnostic<L> {
    fn message(&self) -> String {
        self.message.clone()
    }

    fn location(&self) -> L {
        self.location
    }

    fn additional_labels(&self) -> Vec<(String, L)> {
        vec![]
    }
}

pub trait FileDiagnosticExt: Sized + Diagnostic<FileLocation> {
    fn in_file(self, file: File) -> DiagnosticInFile {
        DiagnosticInFile(InFile::new(Box::new(self), file))
    }
}

impl<T: Diagnostic<FileLocation>> FileDiagnosticExt for T {}

#[derive(Debug)]
pub struct DiagnosticInFile(InFile<Box<dyn Diagnostic<FileLocation>>>);

impl DiagnosticClone<SourceLocation> for DiagnosticInFile {
    fn clone_box(&self) -> Box<dyn Diagnostic<SourceLocation>> {
        let &DiagnosticInFile(InFile { ref value, file }) = self;
        Box::new(Self(InFile::new(value.clone_box(), file)))
    }
}

impl Diagnostic<SourceLocation> for DiagnosticInFile {
    fn message(&self) -> String {
        self.0.value.message()
    }

    fn location(&self) -> SourceLocation {
        let &DiagnosticInFile(InFile { ref value, file }) = self;
        value.location().in_file(file)
    }

    fn additional_labels(&self) -> Vec<(String, SourceLocation)> {
        let &DiagnosticInFile(InFile { ref value, file }) = self;
        value
            .additional_labels()
            .into_iter()
            .map(|(message, location)| (message, location.in_file(file)))
            .collect()
    }
}

fn lower_diagnostic_into_ariadne<L: DiagnosticLocation>(
    context: L::Context<'_>,
    diagnostic: &dyn Diagnostic<L>,
) -> ariadne::Report<'static, Span> {
    let location = diagnostic.location();
    let span = location.span(context);

    ariadne::Report::build(ariadne::ReportKind::Error, *span.source(), span.start())
        .with_message(diagnostic.message())
        .with_label(ariadne::Label::new(span))
        .with_labels(
            diagnostic
                .additional_labels()
                .into_iter()
                .map(|(message, location)| {
                    let span = location.span(context);
                    ariadne::Label::new(span).with_message(message)
                }),
        )
        .finish()
}

pub trait SourceDiagnosticExt: Sized + Diagnostic<SourceLocation> {
    fn emit(self, db: &dyn Db) {
        SourceDiagnosticAccumulator::push(db, Box::new(self))
    }
}
impl<T: Diagnostic<SourceLocation>> SourceDiagnosticExt for T {}

pub trait HirDiagnosticExt: Sized + Diagnostic<HirLocation> {
    fn emit(self, db: &dyn Db) {
        HirDiagnosticAccumulator::push(db, Box::new(self))
    }
}
impl<T: Diagnostic<HirLocation>> HirDiagnosticExt for T {}

impl Clone for Box<dyn Diagnostic<SourceLocation>> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
impl Clone for Box<dyn Diagnostic<HirLocation>> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[salsa::accumulator]
pub struct SourceDiagnosticAccumulator(Box<dyn Diagnostic<SourceLocation>>);

#[salsa::accumulator]
pub struct HirDiagnosticAccumulator(Box<dyn Diagnostic<HirLocation>>);

// TODO: write a macro that gets the accumulated errors and lowers the Hir errors into Source errors
