use crate::compile::{resolve, BlockIdWithFile, HirBlockBody, HirDiagnosticCollector};
use binrw::io::NoSeek;
use binrw::BinWrite;
use shin_core::format::scenario::instructions::Instruction;
use std::io;

struct CountWrite {
    count: u64,
}

impl CountWrite {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn count(&self) -> u64 {
        self.count
    }
}

impl io::Write for CountWrite {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = buf.len();
        self.count += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Stores the lowered instructions block. The instructions are final, except they don't have fixed addresses yet (kinda like a relocatable object file ig).
pub struct LoweredBlock {
    /// Stores the lowered instructions. `None` means that the instruction was not lowered due to an error.
    ///
    /// All the `CodeAddress` elements have zero value, as the addresses in the final file are not yet known at this stages.
    /// They are stored as `BlockIdWithFile` in the `code_addresses` field instead.
    // NOTE: this __can__ be replaced by another type of lowered, but not yet placed instruction, but we opt not to do so
    // it's easier or smth
    pub instructions: Vec<Option<Instruction>>,
    /// Stores the actual values that `CodeAddress` elements in `instructions` refer to.
    pub code_addresses: Vec<BlockIdWithFile>,
}

impl LoweredBlock {
    pub fn from_hir(
        diagnostics: &mut HirDiagnosticCollector,
        resolve_ctx: &resolve::ResolveContext,
        block: &HirBlockBody,
    ) -> Self {
        let mut instructions = Vec::with_capacity(block.instructions.len());
        let mut code_addresses = Vec::new();

        for (instr, _) in block.instructions.iter() {
            instructions.push(super::instruction::instruction_from_hir(
                diagnostics,
                resolve_ctx,
                &mut code_addresses,
                block,
                instr,
            ));
        }

        Self {
            instructions,
            code_addresses,
        }
    }

    /// Checks whether all the instructions in the block are lowered.
    pub fn complete(&self) -> bool {
        self.instructions.iter().all(|instr| instr.is_some())
    }

    /// Computes the size of the serialized block in bytes
    pub fn size(&self) -> Option<u32> {
        let mut size = 0;
        for instr in &self.instructions {
            let instr = instr.as_ref()?;

            let mut count_write = NoSeek::new(CountWrite::new());
            instr
                .write(&mut count_write)
                .expect("BUG: failed to write instruction");

            size += count_write.into_inner().count();
        }

        Some(size.try_into().expect("BUG: block size overflow"))
    }

    pub fn debug_dump(&self) -> String {
        use std::fmt::Write;
        let mut buf = String::new();
        writeln!(buf, "instructions:").unwrap();

        for instr in &self.instructions {
            match instr {
                None => writeln!(buf, "  <error>").unwrap(),
                // TODO: make a reasonable `Display` impl?
                Some(instr) => writeln!(buf, "  {:?}", instr).unwrap(),
            }
        }

        writeln!(buf, "code addresses:").unwrap();
        for code_address in &self.code_addresses {
            writeln!(buf, "  {:?}", code_address).unwrap();
        }

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::LoweredBlock;
    use expect_test::{expect, Expect};

    fn check_from_hir_ok(source: &str, expected: Expect) {
        use crate::compile::{
            db::Database,
            diagnostics::{HirDiagnosticAccumulator, SourceDiagnosticAccumulator},
            file::File,
            from_hir::HirDiagnosticCollector,
            hir,
            resolve::ResolveContext,
        };

        let db = Database::default();
        let db = &db;
        let file = File::new(db, "test.sal".to_string(), source.to_string());

        let bodies = hir::collect_file_bodies(db, file);

        let hir_errors =
            hir::collect_file_bodies::accumulated::<HirDiagnosticAccumulator>(db, file);
        let source_errors =
            hir::collect_file_bodies::accumulated::<SourceDiagnosticAccumulator>(db, file);
        if !source_errors.is_empty() || !hir_errors.is_empty() {
            panic!(
                "hir lowering produced errors:\n\
                source-level: {source_errors:?}\n\
                hir-level: {hir_errors:?}"
            );
        }

        let block = bodies.get_block(db, bodies.get_block_ids(db)[0]).unwrap();

        let mut diagnostics = HirDiagnosticCollector::new();
        let resolve_ctx = ResolveContext::new(db);

        let lowered = LoweredBlock::from_hir(&mut diagnostics, &resolve_ctx, &block);

        if !diagnostics.is_empty() {
            panic!(
                "hir lowering produced errors:\n\
                {:#?}",
                diagnostics
            );
        }

        let lowered = lowered.debug_dump();

        expected.assert_eq(&lowered);
    }

    #[test]
    pub fn check_basic() {
        check_from_hir_ok(
            r#"
            zero $v0
            abs $v1, 42
            not16 $v2, $v1
            "#,
            expect![[r#"
            instructions:
              uo(UnaryOperation { ty: Zero, destination: $v0, source: 0 })
              uo(UnaryOperation { ty: Abs, destination: $v1, source: 42 })
              uo(UnaryOperation { ty: Not16, destination: $v2, source: $v1 })
            code addresses:
            "#]],
        );
    }
}
