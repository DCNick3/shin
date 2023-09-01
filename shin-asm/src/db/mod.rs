// this is for something generated in the salsa output
#![allow(clippy::unused_unit)]

pub mod diagnostics;
pub mod file;
pub mod in_file;
pub mod items;
pub mod symbols;

// TODO: maybe increase jar granularity to per-file?
#[salsa::jar(db = Db)]
pub struct Jar(
    file::File,
    file::Program,
    file::ParsedFile,
    diagnostics::Diagnostics,
    symbols::DefRefId,
    symbols::DefMap,
    symbols::DefMap_get,
    symbols::build_def_map,
    super::hir::collect_file_bodies,
    // items::Item,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::diagnostics::Diagnostics;
    use crate::db::symbols::Name;
    use miette::Diagnostic;
    use salsa::DebugWithDb;

    #[test]
    fn def_maps() {
        let db = Database::default();
        let db = &db;
        let file = file::File::new(
            db,
            "test.sal".to_string(),
            r#"
def ABIBA = 3 + 3

subroutine KEKA
endsub

    add $2, 2, 2
LABEL1:
    sub $2, 2, 2
    j LABEL1
LABEL2:
        "#
            .to_string(),
        );
        let program = file::Program::new(db, vec![file]);
        let def_map = symbols::build_def_map(db, program);

        dbg!(def_map.debug_all(db));

        let abiba_def = def_map.get(db, Name("ABIBA".into())).unwrap();

        dbg!(abiba_def.code_ref(db).file.debug_all(db));

        dbg!(abiba_def.debug_all(db));
        dbg!(def_map.get(db, Name("ABOBA".into())).debug_all(db));
        dbg!(def_map.get(db, Name("KEKA".into())).debug_all(db));
        dbg!(def_map.get(db, Name("LABEL1".into())).debug_all(db));
        dbg!(def_map.get(db, Name("LABEL2".into())).debug_all(db));
    }

    #[test]
    fn bodies() {
        let db = Database::default();
        let db = &db;
        let file = file::File::new(
            db,
            "test.sal".to_string(),
            r#"
LABEL_1:
    add "ass", 2, 2
    MSGINIT -1

LABEL_2:
    add $1, -1, 14
    jt $v0, {
        0 => SNR_0,
        1 => SNR_1,
    } // parser does not split these blocks for some reason..
LABEL_3:
    SELECT 1, 2, $choice, 14, "NCSELECT", [
        "To Be",
        "Not to Be",
    ]
    
LABEL_4:
    // eh, parser seems to get stuck on parenthesis
    exp $result, 1 * ($2 + $3 & 7)
            "#
            .to_string(),
        );

        let bodies = super::super::hir::collect_file_bodies(db, file);

        // crate::db::diagnostics::

        dbg!(bodies);

        let diagnostics =
            super::super::hir::collect_file_bodies::accumulated::<Diagnostics>(db, file);
        for diagnostic in Diagnostics::with_source(db, diagnostics) {
            dbg!(diagnostic.labels().unwrap().collect::<Vec<_>>());
            eprintln!("{:?}", diagnostic);
        }
    }
}
