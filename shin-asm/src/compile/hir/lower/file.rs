use crate::compile::hir::lower::LoweredBlock;
use crate::compile::types::SalsaBlockIdWithFile;
use crate::compile::{hir, BlockId, Db, File, MakeWithFile};
use itertools::Itertools;
use rustc_hash::FxHashMap;
use std::rc::Rc;

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
pub fn lower_file(db: &dyn Db, file: File) -> LoweredFile {
    let block_bodies = hir::collect_file_bodies(db, file);

    let mut bodies = FxHashMap::default();

    for &block_id in block_bodies.bodies(db).keys() {
        let salsa_block_id = SalsaBlockIdWithFile::new(db, block_id.in_file(file));
        bodies.insert(
            block_id,
            Rc::new(hir::lower::lower_block(db, salsa_block_id)),
        );
    }

    LoweredFile::new(db, bodies)
}

#[cfg(test)]
mod tests {
    use crate::compile::diagnostics::{HirDiagnosticAccumulator, SourceDiagnosticAccumulator};
    use crate::compile::hir::lower::test_utils;
    use crate::compile::File;
    use expect_test::{expect, Expect};
    use indoc::indoc;

    fn check_from_hir(source: &str, expected: Expect) {
        use crate::compile::db::Database;
        use std::fmt::Write;

        let db = Database::default();
        let db = &db;

        let file = File::new(db, "test.sal".to_string(), source.to_string());
        let lowered_file = super::lower_file(db, file);

        let hir_errors = super::lower_file::accumulated::<HirDiagnosticAccumulator>(db, file);
        let source_errors = super::lower_file::accumulated::<SourceDiagnosticAccumulator>(db, file);
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
                BLOCK1:
                    abs $v0, 42
                    zero $a0
                
                BLOCK2:
                    not16 $v1, $v0
                    abs $a1, $a0
            "},
            expect![[r#"
                block Block { item_index: ItemIndex(0), block_index: BlockIndex(0) }:
                instructions:
                  uo(UnaryOperation { ty: Abs, destination: $v0, source: 42 })
                  uo(UnaryOperation { ty: Zero, destination: $a0, source: 0 })
                code addresses:

                block Block { item_index: ItemIndex(0), block_index: BlockIndex(1) }:
                instructions:
                  uo(UnaryOperation { ty: Not16, destination: $v1, source: $v0 })
                  uo(UnaryOperation { ty: Abs, destination: $a1, source: $a0 })
                code addresses:

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
                not16 $v1, z
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


                Error: Expected either a number or a register, found a name reference
                   ╭─[test.sal:7:16]
                   │
                 7 │     not16 $v1, z
                   │                ─  
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
