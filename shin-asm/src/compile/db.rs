use super::{def_map, diagnostics, file, hir};

// TODO: maybe increase jar granularity to per-file?
#[salsa::jar(db = Db)]
pub struct Jar(
    file::File,
    file::Program,
    diagnostics::Diagnostics,
    def_map::DefMap,
    def_map::DefMap_get,
    def_map::build_def_map,
    hir::HirBlockBodies,
    hir::HirBlockBodies_get,
    hir::collect_file_bodies,
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
