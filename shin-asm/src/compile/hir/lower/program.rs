use std::rc::Rc;

use itertools::Itertools;
use rustc_hash::FxHashMap;

use crate::compile::{
    def_map,
    hir::lower::{lower_file, LoweredBlock, LoweredFile},
    BlockIdWithFile, Db, File, Program,
};

#[salsa::tracked]
pub struct LoweredProgram {
    #[return_ref]
    pub files: FxHashMap<File, LoweredFile>,
}

impl LoweredProgram {
    pub fn block(&self, db: &dyn Db, block_id: BlockIdWithFile) -> Rc<LoweredBlock> {
        self.files(db)
            .get(&block_id.file)
            .unwrap()
            .bodies(db)
            .get(&block_id.value)
            .unwrap()
            .clone()
    }

    pub fn debug_dump(&self, db: &dyn Db) -> String {
        use std::fmt::Write;
        let mut result = String::new();

        let files = self.files(db);
        for file_id in files.keys().sorted_by_key(|f| f.path(db)) {
            let path = file_id.path(db);
            let file = files.get(file_id).unwrap();
            writeln!(result, "File: {}", path).unwrap();
            writeln!(result, "{}\n\n", file.debug_dump(db)).unwrap();
        }

        result
    }
}

#[salsa::tracked]
pub fn lower_program(db: &dyn Db, program: Program) -> LoweredProgram {
    let def_map = def_map::build_def_map(db, program);

    let mut files = FxHashMap::default();
    for &file in program.files(db) {
        files.insert(file, lower_file(db, def_map, file));
    }

    LoweredProgram::new(db, files)
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};
    use indoc::indoc;

    use crate::compile::{
        diagnostics::{HirDiagnosticAccumulator, SourceDiagnosticAccumulator},
        hir::lower::test_utils,
        File, Program,
    };

    fn check_from_hir(source: &str, expected: Expect) {
        use std::fmt::Write;

        use crate::compile::db::Database;

        let db = Database::default();
        let db = &db;

        let file = File::new(db, "test.sal".to_string(), source.to_string());
        let program = Program::new(db, vec![file]);

        let lowered_program = super::lower_program(db, program);

        let hir_errors = super::lower_program::accumulated::<HirDiagnosticAccumulator>(db, program);
        let source_errors =
            super::lower_program::accumulated::<SourceDiagnosticAccumulator>(db, program);
        let diags = test_utils::diagnostics_to_str(db, hir_errors, source_errors);

        let mut result = String::new();
        if !diags.is_empty() {
            writeln!(result, "Diagnostics:\n{}", diags).unwrap();
        }

        write!(result, "{}", lowered_program.debug_dump(db)).unwrap();

        expected.assert_eq(&result);
    }

    #[test]
    fn check_basic() {
        check_from_hir(
            indoc! {"
                def $BIBA = $a0
                def FORTY_TWO = 42

                BLOCK1:
                    abs $v0, 42
                    zero $BIBA
                
                BLOCK2:
                    not16 $v1, FORTY_TWO
                    abs $a1, $BIBA
            "},
            expect![[r#"
                File: test.sal
                block Block { item_index: ItemIndex(2), block_index: BlockIndex(0) }:
                instructions:
                  uo(UnaryOperation { ty: Abs, destination: $v0, source: 42 })
                  uo(UnaryOperation { ty: Zero, destination: $a0, source: 0 })
                code addresses:

                block Block { item_index: ItemIndex(2), block_index: BlockIndex(1) }:
                instructions:
                  uo(UnaryOperation { ty: Not16, destination: $v1, source: 42 })
                  uo(UnaryOperation { ty: Abs, destination: $a1, source: $a0 })
                code addresses:




            "#]],
        );
    }
}
