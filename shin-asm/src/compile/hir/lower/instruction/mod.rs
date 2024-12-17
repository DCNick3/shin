mod commands;
mod from_instr_args;
mod instr_lowerer;
mod instructions;
mod into_instruction;
mod router;

use shin_core::format::scenario::instructions::Instruction;

use self::router::{Router, RouterBuilder};
use crate::compile::{
    hir,
    hir::lower::{
        from_hir::{FromHirBlockCtx, FromHirCollectors},
        LowerError, LowerResult,
    },
};

pub fn instruction_from_hir(
    collectors: &mut FromHirCollectors,
    ctx: &FromHirBlockCtx,
    instr: hir::InstructionId,
) -> LowerResult<Instruction> {
    let Some(name) = ctx.instr(instr).name.as_ref() else {
        return Err(LowerError);
    };

    let builder = RouterBuilder::new();
    let builder = instructions::instructions(builder);
    let builder = commands::commands(builder);
    let router = builder.build();

    router.handle_instr(collectors, ctx, name, instr)
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};
    use shin_core::{
        format::{
            scenario::{
                instruction_elements::{MessageId, NumberSpec, U8Bool},
                instructions::{Instruction, UnaryOperation, UnaryOperationType},
            },
            text::U16FixupString,
        },
        vm::command::{compiletime::MSGSET, CompiletimeCommand},
    };

    use crate::compile::{
        db::Database,
        hir,
        hir::lower::{
            from_hir::{FromHirBlockCtx, FromHirCollectors},
            test_utils, CodeAddressCollector, HirDiagnosticCollector,
        },
        resolve::ResolveContext,
    };

    fn from_hir(source: &str) -> Result<Instruction, String> {
        let db = Database::default();
        let db = &db;
        let (file, block_id, block) = test_utils::lower_hir_block_ok(db, source);

        let mut diagnostics = HirDiagnosticCollector::new();
        let mut code_address_collector = CodeAddressCollector::new();
        let resolve_ctx = ResolveContext::new_empty(db);

        let (instr, _) = block.instructions.iter().next().unwrap();

        let mut file_diagnostics = diagnostics.with_file(file);
        let mut block_diagnostics = file_diagnostics.with_block(block_id.into());
        let mut collectors = FromHirCollectors {
            diagnostics: &mut block_diagnostics,
            code_address_collector: &mut code_address_collector,
        };
        let ctx = FromHirBlockCtx {
            resolve_ctx: &resolve_ctx,
            block: &block,
        };

        let instr = hir::lower::instruction::instruction_from_hir(&mut collectors, &ctx, instr);

        let code_addresses = code_address_collector.into_block_ids();

        if !code_addresses.is_empty() {
            todo!("code addresses are not supported yet")
        }

        if diagnostics.is_empty() {
            Ok(instr.unwrap())
        } else {
            Err(test_utils::diagnostic_collector_to_str(db, diagnostics))
        }
    }

    fn check_from_hir_ok(source: &str, expected: Instruction) {
        let instr = from_hir(source).expect("failed to lower hir to instruction");

        assert_eq!(instr, expected);
    }

    fn check_from_hir_err(source: &str, expected: Expect) {
        let err = from_hir(source).expect_err("expected hir lowering to fail");

        expected.assert_eq(&err);
    }

    #[test]
    fn test_zero() {
        check_from_hir_ok(
            "zero $v0",
            Instruction::uo(UnaryOperation {
                ty: UnaryOperationType::Zero,
                destination: "$v0".parse().unwrap(),
                source: NumberSpec::constant(0),
            }),
        );
        check_from_hir_ok(
            "zero $v0, 42",
            Instruction::uo(UnaryOperation {
                ty: UnaryOperationType::Zero,
                destination: "$v0".parse().unwrap(),
                source: NumberSpec::constant(42),
            }),
        );
        check_from_hir_ok(
            "zero $v0, $v0",
            Instruction::uo(UnaryOperation {
                ty: UnaryOperationType::Zero,
                destination: "$v0".parse().unwrap(),
                source: NumberSpec::register("$v0".parse().unwrap()),
            }),
        );
    }

    #[test]
    fn test_not16() {
        check_from_hir_ok(
            "not16 $v0, $v1",
            Instruction::uo(UnaryOperation {
                ty: UnaryOperationType::Not16,
                destination: "$v0".parse().unwrap(),
                source: NumberSpec::register("$v1".parse().unwrap()),
            }),
        );
        check_from_hir_ok(
            "not16 $v0, 42",
            Instruction::uo(UnaryOperation {
                ty: UnaryOperationType::Not16,
                destination: "$v0".parse().unwrap(),
                source: NumberSpec::constant(42),
            }),
        );
    }

    #[test]
    fn test_msgset() {
        check_from_hir_ok(
            r#"MSGSET "Heeey!!""#,
            Instruction::Command(CompiletimeCommand::MSGSET(MSGSET {
                msg_id: MessageId(0),
                auto_wait: U8Bool(true),
                text: U16FixupString::new("Heeey!!".to_string()),
            })),
        )
    }

    #[test]
    fn test_type_error() {
        check_from_hir_err(
            "zero 42",
            expect![[r#"
                Error: Expected a register, but got an integer literal
                   ╭─[test.sal:1:6]
                   │
                 1 │ zero 42
                   │      ──  
                   │           
                ───╯
            "#]],
        );
        check_from_hir_err(
            "zero namae_or_whatever",
            expect![[r#"
                Error: Expected a register, but got a name reference
                   ╭─[test.sal:1:6]
                   │
                 1 │ zero namae_or_whatever
                   │      ─────────────────  
                   │                          
                ───╯
            "#]],
        );

        // this checks that the spans are correct for unicode characters
        check_from_hir_err(
            "zero зелибоба",
            expect![[r#"
                Error: Expected a register, but got a name reference
                   ╭─[test.sal:1:6]
                   │
                 1 │ zero зелибоба
                   │      ────────  
                   │                 
                ───╯
            "#]],
        );
    }
}
