use crate::compile::{
    hir,
    hir::lower::{
        from_hir::{FromHirBlockCtx, FromHirCollectors},
        FromHirExpr,
    },
};

// the non-generic part of the implementation
fn expect_opt_args_inner(
    collectors: &mut FromHirCollectors,
    instr: hir::InstructionId,
    instr_args: &[hir::ExprId],
    n_m: usize,
    n_o: usize,
) {
    if instr_args.len() > n_m + n_o {
        let msg = if n_m + n_o > 0 {
            format!(
                "Extra argument: expected no more than {} arguments",
                n_m + n_o
            )
        } else {
            format!("Extra argument: expected no arguments")
        };

        collectors.emit_diagnostic(instr_args[n_m + n_o].into(), msg);
    }
    if instr_args.len() < n_m {
        collectors.emit_diagnostic(
            instr.into(),
            format!(
                "Missing argument: expected at least {} arguments, but got {}",
                n_m,
                instr_args.len()
            ),
        );
    }
}

#[inline]
fn expect_opt_args<const N_M: usize, const N_O: usize>(
    collectors: &mut FromHirCollectors,
    ctx: &FromHirBlockCtx,
    instr: hir::InstructionId,
) -> ([Option<hir::ExprId>; N_M], [Option<hir::ExprId>; N_O]) {
    let instr_args = &ctx.instr(instr).args;
    // N_M is the number of mandatory arguments
    // N_O is the number of optional arguments

    expect_opt_args_inner(collectors, instr, instr_args, N_M, N_O);

    let mut args_m = [None; N_M];
    for (i, arg) in args_m.iter_mut().enumerate() {
        *arg = instr_args.get(i).copied();
    }
    let mut args_o = [None; N_O];
    for (i, arg) in args_o.iter_mut().enumerate() {
        *arg = instr_args.get(i + N_M).copied();
    }

    (args_m, args_o)
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
                let ([$($mty,)*], [$($oty,)*]) = expect_opt_args(collectors, ctx, instr);
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
