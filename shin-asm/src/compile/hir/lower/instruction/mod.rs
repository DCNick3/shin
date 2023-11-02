mod from_instr_args;

use shin_core::format::scenario::{
    instruction_elements::NumberSpec,
    instructions::{Instruction, UnaryOperation, UnaryOperationType},
};

use crate::compile::{
    hir,
    hir::lower::{
        from_hir::{FromHirBlockCtx, FromHirCollectors},
        FromHirExpr,
    },
};

// fn zero()

pub fn instruction_from_hir(
    collectors: &mut FromHirCollectors,
    ctx: &FromHirBlockCtx,
    instr: hir::InstructionId,
) -> Option<Instruction> {
    let Some(name) = ctx.instr(instr).name.as_ref() else {
        return None;
    };

    match name.as_str() {
        "zero" => {
            let [destination, source] =
                from_instr_args::expect_no_more_args(collectors, ctx, instr);

            let destination = destination.map(|id| FromHirExpr::from_hir_expr(collectors, ctx, id));
            let source = source.map(|id| FromHirExpr::from_hir_expr(collectors, ctx, id));

            if destination.is_none() {
                collectors.emit_diagnostic(instr.into(), "Missing a `destination` argument".into());
            }

            let destination = destination??;
            let source = source.flatten().unwrap_or(NumberSpec::constant(0));

            Some(Instruction::uo(UnaryOperation {
                ty: UnaryOperationType::Zero,
                destination,
                source,
            }))
        }
        "not16" | "neg" | "abs" => {
            let [destination, source] =
                from_instr_args::expect_exactly_args(collectors, ctx, instr);

            let destination = destination.map(|id| FromHirExpr::from_hir_expr(collectors, ctx, id));
            let source = source.map(|id| FromHirExpr::from_hir_expr(collectors, ctx, id));

            let destination = destination??;
            let source = source??;

            let ty = match name.as_str() {
                "not16" => UnaryOperationType::Not16,
                "neg" => UnaryOperationType::Negate,
                "abs" => UnaryOperationType::Abs,
                _ => unreachable!(),
            };

            Some(Instruction::uo(UnaryOperation {
                ty,
                destination,
                source,
            }))
        }
        "j" => {
            let [target] = from_instr_args::expect_exactly_args(collectors, ctx, instr);
            let target = target.map(|id| FromHirExpr::from_hir_expr(collectors, ctx, id));

            let target = target??;

            Some(Instruction::j { target })
        }
        _ => {
            collectors.emit_diagnostic(instr.into(), format!("Unknown instruction: `{}`", name));
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};
    use shin_core::format::scenario::{
        instruction_elements::NumberSpec,
        instructions::{Instruction, UnaryOperation, UnaryOperationType},
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
