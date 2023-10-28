use super::{def_map, diagnostics, file, hir, types};
use crate::compile::generate_snr;

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
    def_map::DefMap,
    def_map::DefMap_global_register,
    def_map::DefMap_local_register,
    def_map::DefMap_resolve_item,
    def_map::build_def_map,
    hir::HirBlockBodies,
    hir::HirBlockBodies_get_block,
    hir::HirBlockBodies_get_block_ids,
    hir::HirBlockBodySourceMaps,
    hir::HirBlockBodySourceMaps_get_block,
    hir::collect_file_bodies_with_source_maps,
    hir::collect_file_bodies,
    hir::lower::lower_block,
    hir::lower::lower_file,
    hir::lower::LoweredFile,
    hir::lower::lower_program,
    hir::lower::LoweredProgram,
    generate_snr::DonorHeaders,
    generate_snr::layout_blocks,
    generate_snr::generate_snr,
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
