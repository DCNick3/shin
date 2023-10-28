use crate::compile::hir::{HirBlockId, HirId, HirIdWithBlock};
use crate::compile::{Db, File, MakeWithFile, WithFile};
use std::collections::hash_map::Entry;

use std::fmt::{Debug, Display};

use ariadne::{Source, Span as _};
use rustc_hash::FxHashMap;
use text_size::TextRange;

/// A text range associated with a file. Fully identifies a span of text in the program. Final form of the diagnostic location
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Span(WithFile<TextRange>);

impl Span {
    pub fn new(file: File, range: TextRange) -> Self {
        Self(range.in_file(file))
    }

    pub fn file(&self) -> File {
        self.0.file
    }

    pub fn to_char_span(&self, db: &dyn Db) -> CharSpan {
        let file = self.file();
        let char_map = char_map(db, file);
        let start: usize = self.0.value.start().into();
        let end: usize = self.0.value.end().into();
        let range = (char_map[start], char_map[end]);
        CharSpan(WithFile::new(range, file))
    }
}

#[salsa::tracked]
pub fn char_map(db: &dyn Db, file: File) -> Vec<u32> {
    // stolen from https://github.com/apollographql/apollo-rs/pull/668/files#diff-19fd09cc90a56224f51027101143c2190b9b913993ae35727bfbde19b96f87f7R24
    let contents = file.contents(db);
    let mut map = vec![0; contents.len() + 1];
    let mut char_index = 0;
    for (byte_index, _) in contents.char_indices() {
        map[byte_index] = char_index;
        char_index += 1;
    }

    // Support 1 past the end of the string, for use in exclusive ranges.
    map[contents.len()] = char_index;

    map
}

/// Same as [`Span`], but in terms of characters instead of bytes. This is required until https://github.com/zesterer/ariadne/issues/8 is fixed
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct CharSpan(WithFile<(u32, u32)>);

impl ariadne::Span for CharSpan {
    type SourceId = File;

    fn source(&self) -> &Self::SourceId {
        &self.0.file
    }

    fn start(&self) -> usize {
        self.0.value.0 as usize
    }

    fn end(&self) -> usize {
        self.0.value.1 as usize
    }
}

trait DiagnosticLocation: Debug + Copy + 'static {
    fn span(&self, db: &dyn Db) -> Span;
}

impl DiagnosticLocation for Span {
    fn span(&self, _: &dyn Db) -> Span {
        *self
    }
}

pub type HirLocation = WithFile<HirIdWithBlock>;

impl DiagnosticLocation for HirLocation {
    fn span(&self, db: &dyn Db) -> Span {
        let &WithFile { file, value: node } = self;

        let (_, maps) = collect_file_bodies_with_source_maps(db, file);
        let map = match node.block_id {
            HirBlockId::Block(block) => maps.get_block(db, block).unwrap(),
            HirBlockId::Alias(_) => todo!(),
        };

        Span::new(file, map.get_text_range(node.id).unwrap())
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

    pub fn with_additional_label(mut self, message: String, location: L) -> Self {
        self.additional_labels.push((message, location));
        self
    }

    pub fn map_location<NewL, F: Fn(L) -> NewL>(self, f: F) -> Diagnostic<NewL> {
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
use crate::compile::hir::collect_file_bodies_with_source_maps;
pub(crate) use make_diagnostic;

impl Diagnostic<TextRange> {
    pub fn in_file(self, file: File) -> Diagnostic<Span> {
        self.map_location(|location| Span::new(file, location))
    }
}

impl Diagnostic<Span> {
    pub fn into_ariadne(self, db: &dyn Db) -> ariadne::Report<'static, CharSpan> {
        lower_diagnostic_into_ariadne(db, self)
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
        self.map_location(|location| location.in_file(file))
    }
}

impl Diagnostic<HirLocation> {
    pub fn into_source(self, db: &dyn Db) -> Diagnostic<Span> {
        self.map_location(|location| location.span(db))
    }

    pub fn into_ariadne(self, db: &dyn Db) -> ariadne::Report<'static, CharSpan> {
        lower_diagnostic_into_ariadne(db, self)
    }
}

fn lower_diagnostic_into_ariadne<L: DiagnosticLocation>(
    db: &dyn Db,
    diagnostic: Diagnostic<L>,
) -> ariadne::Report<'static, CharSpan> {
    let span = diagnostic.location.span(db).to_char_span(db);

    ariadne::Report::build(ariadne::ReportKind::Error, *span.source(), span.start())
        .with_message(diagnostic.message)
        .with_label(ariadne::Label::new(span))
        .with_labels(
            diagnostic
                .additional_labels
                .into_iter()
                .map(|(message, location)| {
                    let span = location.span(db).to_char_span(db);
                    ariadne::Label::new(span).with_message(message)
                }),
        )
        .finish()
}

pub struct AriadneDbCache<'db> {
    db: &'db dyn Db,
    sources: FxHashMap<File, Source>,
}

impl<'db> AriadneDbCache<'db> {
    pub fn new(db: &'db dyn Db) -> Self {
        Self {
            db,
            sources: FxHashMap::default(),
        }
    }
}

impl ariadne::Cache<File> for AriadneDbCache<'_> {
    fn fetch(&mut self, &id: &File) -> Result<&Source, Box<dyn Debug + '_>> {
        Ok(match self.sources.entry(id) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Source::from(id.contents(self.db))),
        })
    }

    fn display<'a>(&self, &id: &'a File) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(id.path(self.db)))
    }
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
