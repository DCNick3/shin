use std::io::{Cursor, Seek as _, SeekFrom, Write as _};

use binrw::BinWrite;
use itertools::Itertools;
use rustc_hash::FxHashMap;
use shin_core::format::scenario::{instruction_elements::CodeAddress, ScenarioHeader};

use crate::compile::{
    hir::lower::{LowerResult, LoweredProgram},
    BlockIdWithFile, Db, MakeWithFile,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockLayout {
    block_offsets: FxHashMap<BlockIdWithFile, CodeAddress>,
    block_order: Vec<BlockIdWithFile>,
    snr_file_size: u32,
}

impl BlockLayout {
    pub fn debug_dump(&self, db: &dyn Db) -> String {
        use std::fmt::Write;
        let mut result = String::new();

        writeln!(result, "file size: {}", self.snr_file_size).unwrap();

        for &block_id in &self.block_order {
            let offset = self.block_offsets[&block_id];
            writeln!(
                result,
                "{:08x?} {:?} @ {}",
                offset,
                block_id.value,
                block_id.file.path(db)
            )
            .unwrap();
        }

        result
    }
}

#[salsa::input]
pub struct DonorHeaders {
    #[return_ref]
    pub head_data: Vec<u8>,
    pub snr_header: ScenarioHeader,
}

#[salsa::tracked]
pub fn layout_blocks(
    db: &dyn Db,
    headers: DonorHeaders,
    program: LoweredProgram,
) -> LowerResult<BlockLayout> {
    let mut block_offsets = FxHashMap::default();
    let mut block_order = Vec::new();

    let mut position = headers.snr_header(db).code_offset;
    for (&file_id, file) in program
        .files(db)
        .iter()
        .sorted_by_key(|(file, _)| file.path(db))
    {
        for (&block_id, block) in file.bodies(db) {
            block_order.push(block_id.in_file(file_id));
            block_offsets.insert(block_id.in_file(file_id), CodeAddress(position));
            position += block.code_size()?;
        }
    }

    Ok(BlockLayout {
        block_offsets,
        block_order,
        snr_file_size: position,
    })
}

#[salsa::tracked]
pub fn generate_snr(db: &dyn Db, headers: DonorHeaders, program: LoweredProgram) -> Vec<u8> {
    let block_layout = layout_blocks(db, headers, program).unwrap();

    let header = headers.snr_header(db);
    let header = ScenarioHeader {
        size: block_layout.snr_file_size,
        // TODO: recalculate `dialogue_line_count`
        ..header
    };

    let mut output = Cursor::new(Vec::new());
    output.write_all(headers.head_data(db)).unwrap();
    output.seek(SeekFrom::Start(0)).unwrap();
    header.write(&mut output).unwrap();
    output
        .seek(SeekFrom::Start(header.code_offset as u64))
        .unwrap();

    for &block_id in &block_layout.block_order {
        let offset = block_layout.block_offsets[&block_id];
        assert_eq!(output.position(), offset.0 as u64);

        let block = program.block(db, block_id);
        let resolved_block = block.resolve_code_addresses(&block_layout.block_offsets);
        for instr in resolved_block {
            instr.write(&mut output).unwrap();
        }
    }

    output.into_inner()
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};
    use indoc::indoc;
    use shin_asm::compile::hir::lower::LoweredProgram;
    use shin_core::format::scenario::ScenarioHeader;

    use crate::compile::{
        db::Database,
        diagnostics::{HirDiagnosticAccumulator, SourceDiagnosticAccumulator},
        generate_snr::DonorHeaders,
        hir, Db, File, Program,
    };

    fn lower_program(db: &dyn Db, source: &str) -> (Program, LoweredProgram) {
        let file = File::new(db, "test.sal".to_string(), source.to_string());
        let program = Program::new(db, vec![file]);
        let lowered_program = hir::lower::lower_program(db, program);
        let hir_errors =
            hir::lower::lower_program::accumulated::<HirDiagnosticAccumulator>(db, program);
        let source_errors =
            hir::lower::lower_program::accumulated::<SourceDiagnosticAccumulator>(db, program);
        assert!(hir_errors.is_empty());
        assert!(source_errors.is_empty());

        (program, lowered_program)
    }

    fn check_block_layout(source: &str, expected: Expect) {
        let db = Database::default();
        let db = &db;

        let (_, lowered_program) = lower_program(db, source);

        let donor_headers = DonorHeaders::new(
            db,
            vec![0u8; 0x1000],
            ScenarioHeader {
                size: 0x1000,
                dialogue_line_count: 27,
                unk2: 6,
                unk3: 19,
                unk4_zero: 0,
                unk5_zero: 0,
                unk6_zero: 0,
                code_offset: 0x1000,
            },
        );

        let layout = super::layout_blocks(db, donor_headers, lowered_program).unwrap();

        let actual = layout.debug_dump(db);

        expected.assert_eq(&actual);
    }

    fn check_snr(source: &str, expected: Expect) {
        let db = Database::default();
        let db = &db;

        let (_, lowered_program) = lower_program(db, source);

        let donor_headers = DonorHeaders::new(
            db,
            vec![0u8; 0x80],
            ScenarioHeader {
                size: 0x0,
                dialogue_line_count: 27,
                unk2: 6,
                unk3: 19,
                unk4_zero: 0,
                unk5_zero: 0,
                unk6_zero: 0,
                code_offset: 0x80,
            },
        );

        let snr = super::generate_snr(db, donor_headers, lowered_program);

        let actual = pretty_hex::pretty_hex(&snr);

        expected.assert_eq(&actual);
    }

    #[test]
    fn test_layout() {
        check_block_layout(
            indoc! {"
                ABOBA:
                    neg $v0, 42
                    abs $v1, 42

                BIBA:
                    not16 $v0, 42
                    zero $v1

                KEKA:
                    neg $v0, 42
                    abs $v1, 42
                    j ABOBA
                    j ABOBA
                    j BIBA
            "},
            expect![[r#"
                file size: 4141
                00001000 BlockId { item_index: 0, block_index: Some(0) } @ test.sal
                0000100a BlockId { item_index: 0, block_index: Some(1) } @ test.sal
                00001014 BlockId { item_index: 0, block_index: Some(2) } @ test.sal
            "#]],
        );
    }

    #[test]
    fn test_snr() {
        check_snr(
            indoc! {"
                ABOBA:
                    neg $v0, 42
                    abs $v1, 42
    
                BIBA:
                    not16 $v0, 42
                    zero $v1
    
                KEKA:
                    neg $v0, 42
                    abs $v1, 42
                    j ABOBA
                    j ABOBA
                    j BIBA
            "},
            expect![[r#"
                Length: 173 (0xad) bytes
                0000:   53 4e 52 20  ad 00 00 00  1b 00 00 00  06 00 00 00   SNR ............
                0010:   13 00 00 00  00 00 00 00  00 00 00 00  00 00 00 00   ................
                0020:   80 00 00 00  00 00 00 00  00 00 00 00  00 00 00 00   ................
                0030:   00 00 00 00  00 00 00 00  00 00 00 00  00 00 00 00   ................
                0040:   00 00 00 00  00 00 00 00  00 00 00 00  00 00 00 00   ................
                0050:   00 00 00 00  00 00 00 00  00 00 00 00  00 00 00 00   ................
                0060:   00 00 00 00  00 00 00 00  00 00 00 00  00 00 00 00   ................
                0070:   00 00 00 00  00 00 00 00  00 00 00 00  00 00 00 00   ................
                0080:   40 82 00 00  2a 40 83 01  00 2a 40 81  00 00 2a 40   @...*@...*@...*@
                0090:   80 01 00 00  40 82 00 00  2a 40 83 01  00 2a 47 80   ....@...*@...*G.
                00a0:   00 00 00 47  80 00 00 00  47 8a 00 00  00            ...G....G...."#]],
        )
    }
}
