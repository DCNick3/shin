//! Defines `FromVmCtx` and `FromVmCtxDefault` traits, that are used to convert from compile-time (e.g. `NumberSpec`) to runtime (e.g. `i32`) representations of command parameters
//!
//! Also contains implementation for std types & stuff defined in `shin_core::format`, like `U8String` -> `String` stuff

use crate::vm::VmCtx;

/// Defines how to convert a compile-time representation `I` to a runtime representation `Self`
///
/// For example this is used to convey that a NumberSpec can be converted to i32 (by inspecting the VmCtx)
pub trait FromVmCtx<I>
where
    Self: Sized,
{
    fn from_vm_ctx(ctx: &VmCtx, input: I) -> Self;
}

/// Defines the default conversion from VmCtx
///
/// For example this is used to convey that a NumberSpec is usually converted to i32
pub trait FromVmCtxDefault
where
    Self: Sized,
{
    type Output: FromVmCtx<Self>;
    fn from_vm_ctx(ctx: &VmCtx, input: Self) -> Self::Output {
        FromVmCtx::<Self>::from_vm_ctx(ctx, input)
    }
}

macro_rules! identity_from_vm_ctx {
    ($($t:ty),*) => {
        $(
            impl FromVmCtx<$t> for $t {
                fn from_vm_ctx(_: &VmCtx, input: $t) -> Self {
                    input
                }
            }
        )*
    };
}

macro_rules! identity_from_vm_ctx_default {
    ($($t:ty),*) => {
        $(
            identity_from_vm_ctx!($t);
            impl FromVmCtxDefault for $t {
                type Output = $t;
            }
        )*
    };
}

identity_from_vm_ctx_default!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);
