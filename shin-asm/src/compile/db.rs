use super::{def_map, diagnostics, file, hir, types};

// TODO: maybe increase jar granularity to per-file?
#[salsa::jar(db = Db)]
pub struct Jar(
    file::File,
    file::File_emit_diagnostics,
    file::Program,
    types::SalsaBlockIdWithFile,
    diagnostics::SourceDiagnosticAccumulator,
    diagnostics::HirDiagnosticAccumulator,
    diagnostics::char_map,
    def_map::build_def_map,
    hir::HirBlockBodies,
    hir::HirBlockBodies_get_block,
    hir::HirBlockBodySourceMaps,
    hir::HirBlockBodySourceMaps_get_block,
    hir::collect_file_bodies_with_source_maps,
    hir::collect_file_bodies,
    hir::lower::lower_block,
);

pub trait Db: salsa::DbWithJar<Jar> {}
impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> {}

#[salsa::db(Jar)]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Database {}
// impl salsa::ParallelDatabase for Database {
//     fn snapshot(&self) -> salsa::Snapshot<Self> {
//         salsa::Snapshot::new(Self {
//             storage: self.storage.snapshot(),
//         })
//     }
// }
