use std::rc::Rc;

use itertools::Itertools;
use rustc_hash::FxHashMap;

use crate::compile::{
    hir, hir::lower::LoweredBlock, types::SalsaBlockIdWithFile, BlockId, Db, DefMap, File,
    MakeWithFile,
};

#[salsa::tracked]
pub struct LoweredFile {
    #[return_ref]
    pub bodies: FxHashMap<BlockId, Rc<LoweredBlock>>,
}

impl LoweredFile {
    pub fn debug_dump(&self, db: &dyn Db) -> String {
        use std::fmt::Write;
        let mut result = String::new();

        let bodies = self.bodies(db);
        for block_id in bodies.keys().sorted() {
            let block = bodies.get(block_id).unwrap();
            writeln!(result, "block {:?}:", block_id.repr()).unwrap();
            writeln!(result, "{}", block.debug_dump()).unwrap();
        }

        result
    }
}

#[salsa::tracked]
pub fn lower_file(db: &dyn Db, def_map: DefMap, file: File) -> LoweredFile {
    let block_bodies = hir::collect_file_bodies(db, file);

    let mut bodies = FxHashMap::default();

    for &block_id in block_bodies.bodies(db).keys() {
        let salsa_block_id = SalsaBlockIdWithFile::new(db, block_id.in_file(file));
        bodies.insert(
            block_id,
            Rc::new(hir::lower::lower_block(db, def_map, salsa_block_id)),
        );
    }

    LoweredFile::new(db, bodies)
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};
    use indoc::indoc;

    use crate::compile::{
        def_map::build_def_map,
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
        let def_map = build_def_map(db, program);

        let lowered_file = super::lower_file(db, def_map, file);

        let hir_errors =
            super::lower_file::accumulated::<HirDiagnosticAccumulator>(db, def_map, file);
        let source_errors =
            super::lower_file::accumulated::<SourceDiagnosticAccumulator>(db, def_map, file);
        let diags = test_utils::diagnostics_to_str(db, hir_errors, source_errors);

        let mut result = String::new();
        if !diags.is_empty() {
            writeln!(result, "Diagnostics:\n{}", diags).unwrap();
        }

        write!(result, "{}", lowered_file.debug_dump(db)).unwrap();

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

    #[test]
    fn check_code_addresses() {
        check_from_hir(
            indoc! {"
                BLOCK1:
                    j BLOCK2
                
                BLOCK2:
                    j BLOCK1
            "},
            expect![[r#"
                block Block { item_index: ItemIndex(0), block_index: BlockIndex(0) }:
                instructions:
                  j { target: 0x0j }
                code addresses:
                  WithFile { value: BlockId { item_index: 0, block_index: Some(1) }, file: File(Id { value: 1 }) }

                block Block { item_index: ItemIndex(0), block_index: BlockIndex(1) }:
                instructions:
                  j { target: 0x0j }
                code addresses:
                  WithFile { value: BlockId { item_index: 0, block_index: Some(0) }, file: File(Id { value: 1 }) }

            "#]],
        );
    }

    #[test]
    fn check_err() {
        check_from_hir(
            indoc! {"
            BLOCK1:
                abs $v0, 42
                ABOBA 42 42 42
            
            BLOCK2:
                zero 42, 43
                not16 $KEKAS, z
            "},
            expect![[r#"
                Diagnostics:
                Error: expected COMMA
                   ╭─[test.sal:3:13]
                   │
                 3 │     ABOBA 42 42 42
                   │             ─  
                   │                 
                ───╯


                Error: expected COMMA
                   ╭─[test.sal:3:16]
                   │
                 3 │     ABOBA 42 42 42
                   │                ─  
                   │                    
                ───╯


                Error: Expected a register, but got an integer literal
                   ╭─[test.sal:6:10]
                   │
                 6 │     zero 42, 43
                   │          ──  
                   │               
                ───╯


                Error: Unresolved register alias: `$KEKAS`
                   ╭─[test.sal:7:11]
                   │
                 7 │     not16 $KEKAS, z
                   │           ──────  
                   │                    
                ───╯


                Error: Could not find the definition of `z`
                   ╭─[test.sal:7:19]
                   │
                 7 │     not16 $KEKAS, z
                   │                   ─  
                   │                       
                ───╯


                Error: Unknown instruction: `ABOBA`
                   ╭─[test.sal:3:5]
                   │
                 3 │     ABOBA 42 42 42
                   │     ───────────────  
                   │                       
                ───╯

                block Block { item_index: ItemIndex(0), block_index: BlockIndex(0) }:
                instructions:
                  uo(UnaryOperation { ty: Abs, destination: $v0, source: 42 })
                  <error>
                code addresses:

                block Block { item_index: ItemIndex(0), block_index: BlockIndex(1) }:
                instructions:
                  <error>
                  <error>
                code addresses:

            "#]],
        )
    }
}
