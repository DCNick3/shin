use crate::compile::from_hir::{HirBlockId, HirId, HirIdWithBlock};
use crate::compile::{Db, File, MakeWithFile, WithFile};

use std::fmt::Debug;

use ariadne::Span as _;
use text_size::TextRange;

/// A text range associated with a file. Fully identifies a span of text in the program. Final form of the diagnostic location
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Span(WithFile<TextRange>);

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

impl DiagnosticLocation for Span {
    type Context<'a> = ();
    fn span(&self, _: Self::Context<'_>) -> Span {
        *self
    }
}

/// A location specified by a HIR node range and a file id. Diagnostic machinery will use the HIR source map to get the actual source range.
#[derive(Debug, Copy, Clone)]
pub struct HirLocation(pub WithFile<(HirIdWithBlock, HirIdWithBlock)>);

impl HirLocation {
    pub fn single_node(hir_id_in_block: HirIdWithBlock, file: File) -> Self {
        Self(WithFile::new((hir_id_in_block, hir_id_in_block), file))
    }
}
impl DiagnosticLocation for HirLocation {
    type Context<'a> = &'a dyn Db;

    fn span(&self, _db: Self::Context<'_>) -> Span {
        let WithFile {
            file: _file,
            value: (_start_node, _end_node),
        } = self.0;

        todo!("Collect HIR source maps and use them to get the location")
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic<L> {
    pub message: String,
    pub location: L,
    pub additional_labels: Vec<(String, L)>,
}

impl<L> Diagnostic<L> {
    pub fn new(message: String, location: L) -> Self {
        Self {
            message,
            location,
            additional_labels: Vec::new(),
        }
    }

    fn map_location<NewL, F: Fn(L) -> NewL>(self, f: F) -> Diagnostic<NewL> {
        Diagnostic {
            message: self.message,
            location: f(self.location),
            additional_labels: self
                .additional_labels
                .into_iter()
                .map(|(m, l)| (m, f(l)))
                .collect(),
        }
    }
}

macro_rules! make_diagnostic {
    ($token:expr => $file:expr, $($fmt:expr),+) => {
        {
            use $crate::syntax::ast::AstSpanned as _;
            $crate::compile::diagnostics::Diagnostic::new(format!($($fmt),+), $token.span($file))
        }
    };
    ($span:expr, $($fmt:expr),+) => {
        $crate::compile::diagnostics::Diagnostic::new(format!($($fmt),+), $span)
    };
}
pub(crate) use make_diagnostic;

impl Diagnostic<TextRange> {
    pub fn in_file(self, file: File) -> Diagnostic<Span> {
        self.map_location(|location| Span::new(file, location))
    }
}

impl Diagnostic<HirId> {
    pub fn in_block(self, block: impl Into<HirBlockId>) -> Diagnostic<HirIdWithBlock> {
        let block = block.into();
        self.map_location(|location| HirIdWithBlock::new(location, block))
    }
}

impl Diagnostic<HirIdWithBlock> {
    pub fn in_file(self, file: File) -> Diagnostic<HirLocation> {
        self.map_location(|location| HirLocation::single_node(location, file))
    }
}

fn lower_diagnostic_into_ariadne<L: DiagnosticLocation>(
    context: L::Context<'_>,
    diagnostic: Diagnostic<L>,
) -> ariadne::Report<'static, Span> {
    let span = diagnostic.location.span(context);

    ariadne::Report::build(ariadne::ReportKind::Error, *span.source(), span.start())
        .with_message(diagnostic.message)
        .with_label(ariadne::Label::new(span))
        .with_labels(
            diagnostic
                .additional_labels
                .into_iter()
                .map(|(message, location)| {
                    let span = location.span(context);
                    ariadne::Label::new(span).with_message(message)
                }),
        )
        .finish()
}

impl Diagnostic<Span> {
    pub fn emit(self, db: &dyn Db) {
        SourceDiagnosticAccumulator::push(db, self)
    }
}

impl Diagnostic<HirLocation> {
    pub fn emit(self, db: &dyn Db) {
        HirDiagnosticAccumulator::push(db, self)
    }
}

#[salsa::accumulator]
pub struct SourceDiagnosticAccumulator(Diagnostic<Span>);

#[salsa::accumulator]
pub struct HirDiagnosticAccumulator(Diagnostic<HirLocation>);

// TODO: write a macro that gets the accumulated errors and lowers the Hir errors into Source errors
