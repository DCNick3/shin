use crate::compile::{
    hir, resolve, BlockIdWithFile, FromHirExpr, HirBlockBody, HirDiagnosticCollectorWithBlock,
};
use shin_core::format::scenario::instruction_elements::{NumberSpec, UntypedNumberSpec};
use shin_core::format::scenario::instructions::{Instruction, UnaryOperation, UnaryOperationType};

fn expect_no_more_args<const N: usize>(
    diagnostics: &mut HirDiagnosticCollectorWithBlock,
    block: &HirBlockBody,
    instr: hir::InstructionId,
) -> [Option<hir::ExprId>; N] {
    let instr = &block.instructions[instr];
    if instr.args.len() > N {
        diagnostics.emit(
            instr.args[N].into(),
            format!("Expected no more than {} arguments", N),
        );
    }

    let mut args = [None; N];
    for (i, arg) in args.iter_mut().enumerate() {
        *arg = instr.args.get(i).copied();
    }

    args
}

pub fn instruction_from_hir(
    diagnostics: &mut HirDiagnosticCollectorWithBlock,
    resolve_ctx: &resolve::ResolveContext,
    _code_addresses: &mut Vec<BlockIdWithFile>,
    block: &HirBlockBody,
    instr: hir::InstructionId,
) -> Option<Instruction> {
    let Some(name) = block.instructions[instr].name.as_ref() else {
        return None;
    };

    match name.as_str() {
        "zero" => {
            let [destination, source] = expect_no_more_args(diagnostics, block, instr);

            let destination = destination
                .map(|id| FromHirExpr::from_hir_expr(diagnostics, resolve_ctx, block, id));
            let source =
                source.map(|id| FromHirExpr::from_hir_expr(diagnostics, resolve_ctx, block, id));

            if destination.is_none() {
                diagnostics.emit(instr.into(), "Missing a `destination` argument".into());
            }

            let destination = destination??;
            let source = source
                .flatten()
                .unwrap_or(NumberSpec::new(UntypedNumberSpec::Constant(0)));

            Some(Instruction::uo(UnaryOperation {
                ty: UnaryOperationType::Zero,
                destination,
                source,
            }))
        }
        "not16" | "neg" | "abs" => {
            let [destination, source] = expect_no_more_args(diagnostics, block, instr);

            let destination = destination
                .map(|id| FromHirExpr::from_hir_expr(diagnostics, resolve_ctx, block, id));
            let source =
                source.map(|id| FromHirExpr::from_hir_expr(diagnostics, resolve_ctx, block, id));

            if destination.is_none() {
                diagnostics.emit(instr.into(), "Missing a `destination` argument".into());
            }
            if source.is_none() {
                diagnostics.emit(instr.into(), "Missing a `source` argument".into());
            }

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
        _ => {
            diagnostics.emit(instr.into(), format!("Unknown instruction: `{}`", name));
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::compile::hir::lower::test_utils;
    use expect_test::{expect, Expect};
    use shin_core::format::scenario::instruction_elements::{NumberSpec, UntypedNumberSpec};
    use shin_core::format::scenario::instructions::{
        Instruction, UnaryOperation, UnaryOperationType,
    };

    fn from_hir(source: &str) -> Result<Instruction, String> {
        use crate::compile::{
            db::Database, from_hir::HirDiagnosticCollector, hir, resolve::ResolveContext,
        };

        let db = Database::default();
        let db = &db;
        let (file, block_id, block) = test_utils::lower_hir_block_ok(db, source);

        let mut diagnostics = HirDiagnosticCollector::new();
        let resolve_ctx = ResolveContext::new(db);

        let mut code_addresses = Vec::new();
        let (instr, _) = block.instructions.iter().next().unwrap();
        let instr = hir::lower::instruction::instruction_from_hir(
            &mut diagnostics.with_file(file).with_block(block_id.into()),
            &resolve_ctx,
            &mut code_addresses,
            &block,
            instr,
        );

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
                source: NumberSpec::new(UntypedNumberSpec::Constant(0)),
            }),
        );
        check_from_hir_ok(
            "zero $v0, 42",
            Instruction::uo(UnaryOperation {
                ty: UnaryOperationType::Zero,
                destination: "$v0".parse().unwrap(),
                source: NumberSpec::new(UntypedNumberSpec::Constant(42)),
            }),
        );
        check_from_hir_ok(
            "zero $v0, $v0",
            Instruction::uo(UnaryOperation {
                ty: UnaryOperationType::Zero,
                destination: "$v0".parse().unwrap(),
                source: NumberSpec::new(UntypedNumberSpec::Register("$v0".parse().unwrap())),
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
                source: NumberSpec::new(UntypedNumberSpec::Register("$v1".parse().unwrap())),
            }),
        );
        check_from_hir_ok(
            "not16 $v0, 42",
            Instruction::uo(UnaryOperation {
                ty: UnaryOperationType::Not16,
                destination: "$v0".parse().unwrap(),
                source: NumberSpec::new(UntypedNumberSpec::Constant(42)),
            }),
        );
    }

    #[test]
    fn test_type_error() {
        check_from_hir_err(
            "zero 42",
            expect![[r#"
            Error: Expected a register reference, but got an integer literal
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
                Error: Expected a register reference, but got a name reference
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
                Error: Expected a register reference, but got a name reference
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
