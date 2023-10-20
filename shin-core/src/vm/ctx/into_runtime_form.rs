//! Defines `FromVmCtx` and `FromVmCtxDefault` traits, that are used to convert from compile-time (e.g. `NumberSpec`) to runtime (e.g. `i32`) representations of command parameters
//!
//! Also contains implementation for std types & stuff defined in `shin_core::format`, like `U8String` -> `String` stuff

use crate::vm::VmCtx;

/// Defines how to convert a compile-time representation `Self` to a runtime representation
///
/// For example this is used to convey that a NumberSpec can be converted to i32 (by inspecting the VmCtx)
///
/// This also defines __which__ runtime representation is used, making a one-to-one mapping from compile-time to runtime representations
pub trait IntoRuntimeForm {
    type Output;
    fn into_runtime_form(self, ctx: &VmCtx) -> Self::Output;
}

macro_rules! identity_runtime_repr {
    ($($t:ty),*) => {
        $(
            impl IntoRuntimeForm for $t {
                type Output = $t;
                fn into_runtime_form(self, _: &VmCtx) -> Self::Output {
                    self
                }
            }
        )*
    };
}

identity_runtime_repr!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);
