//! Defines `FromVmCtx` and `FromVmCtxDefault` traits, that are used to convert from compile-time (e.g. `NumberSpec`) to runtime (e.g. `i32`) representations of command parameters
//!
//! Also contains implementation for std types & stuff defined in `shin_core::format`, like `U8String` -> `String` stuff

use crate::format::scenario::instruction_elements::{
    BitmaskNumberArray, MessageId, NumberSpec, UntypedNumberSpec,
};
use crate::format::scenario::types::U8SmallNumberList;
use crate::format::text::{StringArray, U16FixupString, U16String, U8FixupString, U8String};
use crate::time::Ticks;
use crate::vm::VmCtx;
use smallvec::SmallVec;

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

identity_from_vm_ctx_default!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, MessageId);

macro_rules! impl_from_vm_ctx_tuple {
    ($($name:ident),+) => {
        impl<$($name: FromVmCtx<UntypedNumberSpec>),+> FromVmCtx<BitmaskNumberArray> for ($($name,)+) {
            fn from_vm_ctx(ctx: &VmCtx, input: BitmaskNumberArray) -> Self {
                let mut iter = input.0.iter().cloned();
                ($(
                    $name::from_vm_ctx(ctx, iter.next().unwrap()),
                )+)
            }
        }
    };
}

impl_from_vm_ctx_tuple!(T1);
impl_from_vm_ctx_tuple!(T1, T2);
impl_from_vm_ctx_tuple!(T1, T2, T3);
impl_from_vm_ctx_tuple!(T1, T2, T3, T4);
impl_from_vm_ctx_tuple!(T1, T2, T3, T4, T5);
impl_from_vm_ctx_tuple!(T1, T2, T3, T4, T5, T6);
impl_from_vm_ctx_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_from_vm_ctx_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);

impl FromVmCtx<UntypedNumberSpec> for i32 {
    fn from_vm_ctx(ctx: &VmCtx, input: UntypedNumberSpec) -> Self {
        ctx.get_number(NumberSpec::new(input))
    }
}
impl FromVmCtxDefault for UntypedNumberSpec {
    type Output = i32;
}
impl FromVmCtx<u8> for bool {
    fn from_vm_ctx(_: &VmCtx, input: u8) -> Self {
        input != 0
    }
}

impl FromVmCtx<U8String> for String {
    fn from_vm_ctx(_: &VmCtx, input: U8String) -> Self {
        input.0
    }
}
impl FromVmCtxDefault for U8String {
    type Output = String;
}

impl FromVmCtx<U16String> for String {
    fn from_vm_ctx(_: &VmCtx, input: U16String) -> Self {
        input.0
    }
}
impl FromVmCtxDefault for U16String {
    type Output = String;
}

impl FromVmCtx<U8FixupString> for String {
    fn from_vm_ctx(_: &VmCtx, input: U8FixupString) -> Self {
        input.0
    }
}
impl FromVmCtxDefault for U8FixupString {
    type Output = String;
}

impl FromVmCtx<U16FixupString> for String {
    fn from_vm_ctx(_: &VmCtx, input: U16FixupString) -> Self {
        input.0
    }
}
impl FromVmCtxDefault for U16FixupString {
    type Output = String;
}

impl FromVmCtx<StringArray> for SmallVec<[String; 4]> {
    fn from_vm_ctx(_: &VmCtx, input: StringArray) -> Self {
        input.0
    }
}
impl FromVmCtxDefault for StringArray {
    type Output = SmallVec<[String; 4]>;
}

impl FromVmCtx<BitmaskNumberArray> for [i32; 8] {
    fn from_vm_ctx(ctx: &VmCtx, input: BitmaskNumberArray) -> Self {
        input.0.map(|n| ctx.get_number(NumberSpec::new(n)))
    }
}
impl FromVmCtxDefault for BitmaskNumberArray {
    type Output = [i32; 8];
}

impl FromVmCtx<U8SmallNumberList> for SmallVec<[i32; 6]> {
    fn from_vm_ctx(ctx: &VmCtx, input: U8SmallNumberList) -> Self {
        input.0.into_iter().map(|n| ctx.get_number(n)).collect()
    }
}
impl FromVmCtxDefault for U8SmallNumberList {
    type Output = SmallVec<[i32; 6]>;
}

// TODO: remove when BitmaskNumberArray is made like the NumberSpec
impl FromVmCtx<UntypedNumberSpec> for Ticks {
    fn from_vm_ctx(ctx: &VmCtx, input: UntypedNumberSpec) -> Self {
        Ticks::from_i32(ctx.get_number(NumberSpec::new(input)))
    }
}
