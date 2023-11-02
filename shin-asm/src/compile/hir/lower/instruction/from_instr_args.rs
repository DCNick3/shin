use crate::compile::{
    hir,
    hir::lower::{
        from_hir::{FromHirBlockCtx, FromHirCollectors},
        FromHirExpr,
    },
};

pub fn expect_no_more_args<const N: usize>(
    collectors: &mut FromHirCollectors,
    ctx: &FromHirBlockCtx,
    instr: hir::InstructionId,
) -> [Option<hir::ExprId>; N] {
    let instr = ctx.instr(instr);
    if instr.args.len() > N {
        collectors.emit_diagnostic(
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

pub fn expect_exactly_args<const N: usize>(
    collectors: &mut FromHirCollectors,
    ctx: &FromHirBlockCtx,
    instr: hir::InstructionId,
) -> [Option<hir::ExprId>; N] {
    let instr_args = &ctx.instr(instr).args;
    if instr_args.len() > N {
        let msg = if N > 0 {
            format!(
                "Extra argument: expected exactly {} arguments, but got {}",
                N,
                instr_args.len()
            )
        } else {
            format!(
                "Extra argument: expected no arguments, but got {}",
                instr_args.len()
            )
        };

        collectors.emit_diagnostic(instr_args[N].into(), msg);
    } else if instr_args.len() < N {
        collectors.emit_diagnostic(
            instr.into(),
            format!(
                "Missing argument: expected exactly {} arguments, but got {}",
                N,
                instr_args.len()
            ),
        );
    }

    let mut args = [None; N];
    for (i, arg) in args.iter_mut().enumerate() {
        *arg = instr_args.get(i).copied();
    }

    args
}

pub trait FromInstrArgs: Sized {
    fn from_instr_args(
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        instr: hir::InstructionId,
    ) -> Option<Self>;
}

impl FromInstrArgs for () {
    fn from_instr_args(
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        instr: hir::InstructionId,
    ) -> Option<Self> {
        if !ctx.instr(instr).args.is_empty() {
            collectors.emit_diagnostic(
                instr.into(),
                format!("This instruction does not take any arguments"),
            );

            None
        } else {
            Some(())
        }
    }
}

fn transpose<T>(v: Option<Option<T>>) -> Option<Option<T>> {
    match v {
        None => Some(None),
        Some(None) => None,
        Some(Some(v)) => Some(Some(v)),
    }
}

// handle optional arguments
macro_rules! impl_from_instr_args_opt_tuple {
    ($($mty:ident),* @ $($oty:ident),*) => {
        impl<$($mty,)* $($oty,)*> FromInstrArgs for ($($mty,)* $(Option<$oty>,)*)
        where
            $($mty: FromHirExpr,)*
            $($oty: FromHirExpr,)*
        {
            #[allow(non_snake_case)] // this is fiiiine
            fn from_instr_args(
                collectors: &mut FromHirCollectors,
                ctx: &FromHirBlockCtx,
                instr: hir::InstructionId,
            ) -> Option<Self> {
                let [
                    $($mty,)*
                    $($oty,)*
                ] = expect_no_more_args(collectors, ctx, instr);
                $(
                    let $mty = $mty.and_then(|arg| {
                        FromHirExpr::from_hir_expr(collectors, ctx, arg)
                    });
                )*
                $(
                    let $oty = $oty.map(|arg| {
                        FromHirExpr::from_hir_expr(collectors, ctx, arg)
                    });
                )*

                Some(($($mty?,)* $(transpose($oty)?,)*))
            }
        }
    };
}

impl_from_instr_args_opt_tuple!(M1 @);
impl_from_instr_args_opt_tuple!(@ O1);

impl_from_instr_args_opt_tuple!(M1, M2 @);
impl_from_instr_args_opt_tuple!(M1 @ O1);
impl_from_instr_args_opt_tuple!(@ O1, O2);

impl_from_instr_args_opt_tuple!(M1, M2, M3 @);
impl_from_instr_args_opt_tuple!(M1, M2 @ O1);
impl_from_instr_args_opt_tuple!(M1 @ O1, O2);
impl_from_instr_args_opt_tuple!(@ O1, O2, O3);

impl_from_instr_args_opt_tuple!(M1, M2, M3, M4 @);
impl_from_instr_args_opt_tuple!(M1, M2, M3 @ O1);
impl_from_instr_args_opt_tuple!(M1, M2 @ O1, O2);
impl_from_instr_args_opt_tuple!(M1 @ O1, O2, O3);
impl_from_instr_args_opt_tuple!(@ O1, O2, O3, O4);
