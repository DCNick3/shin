use crate::format::scenario::instructions::{BitmaskNumberArray, MessageId, NumberSpec};
use crate::format::scenario::types::U8SmallNumberList;
use crate::format::text::{StringArray, U16FixupString, U16String, U8FixupString, U8String};
use crate::vm::command::layer::LayerPropertySmallList;
use crate::vm::VmCtx;
use smallvec::SmallVec;

/// Defines how to convert I to Self
///
/// I is a compile time representation (e.g. a NumberSpec), while Self is a runtime representation (e.g. an i32)
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

impl FromVmCtx<NumberSpec> for i32 {
    fn from_vm_ctx(ctx: &VmCtx, input: NumberSpec) -> Self {
        ctx.get_number(input)
    }
}
impl FromVmCtxDefault for NumberSpec {
    type Output = i32;
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
        input.0.map(|n| ctx.get_number(n))
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

impl FromVmCtx<U8SmallNumberList> for LayerPropertySmallList {
    fn from_vm_ctx(ctx: &VmCtx, input: U8SmallNumberList) -> Self {
        input
            .0
            .into_iter()
            .map(|n| {
                let n = ctx.get_number(n);
                num_traits::FromPrimitive::from_i32(n).unwrap_or_else(|| {
                    panic!(
                        "LayerPropertySmallList::from_vm_ctx: invalid layer type: {}",
                        n
                    )
                })
            })
            .collect()
    }
}
