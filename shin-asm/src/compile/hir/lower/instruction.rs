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
    use shin_core::format::scenario::instruction_elements::{NumberSpec, UntypedNumberSpec};
    use shin_core::format::scenario::instructions::{
        Instruction, UnaryOperation, UnaryOperationType,
    };

    fn check_from_hir_ok(source: &str, expected: Instruction) {
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

        let block_id = bodies.get_block_ids(db)[0];
        let block = bodies.get_block(db, block_id).unwrap();

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

        let Some(instr) = instr else {
            panic!(
                "instruction was not lowered. diagnostics: {:#?}",
                diagnostics
            );
        };

        assert_eq!(instr, expected);
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
        check_from_hir_ok(
            "zero 42",
            Instruction::uo(UnaryOperation {
                ty: UnaryOperationType::Zero,
                destination: "$v0".parse().unwrap(),
                source: NumberSpec::new(UntypedNumberSpec::Register("$v1".parse().unwrap())),
            }),
        );
    }
}
