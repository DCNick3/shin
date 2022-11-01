use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};

/// A string wrapper that converts the str to a $path `TokenStream`, allowing
/// for constant-time idents that can be shared across threads
#[derive(Clone, Copy)]
pub struct IdentStr(&'static str);

impl IdentStr {
    #[cfg_attr(coverage_nightly, no_coverage)] // const-only function
    pub(crate) const fn new(str: &'static str) -> Self {
        IdentStr(str)
    }
}

impl ToTokens for IdentStr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let idents = self
            .0
            .split("::")
            .map(|ident| Ident::new(ident, Span::call_site()));
        tokens.append_separated(idents, quote!(::));
    }
}

macro_rules! ident_str {
    () => {};

    ($vis:vis $ident:ident = $path:expr; $($tail:tt)*) => {
        ident_str!($vis $ident = $path);
        ident_str!($($tail)*);
    };

    ($vis:vis $ident:ident = $path:expr) => {
        $vis const $ident: $crate::sanitization::IdentStr =
            $crate::sanitization::IdentStr::new($path);
    };
}

macro_rules! from_shin_core {
    ($path:path) => {
        concat!("shin_core::", stringify!($path))
    };
}

macro_rules! from_binrw {
    ($path:path) => {
        concat!("binrw::", stringify!($path))
    };
}

ident_str! {
    pub VM_CTX = from_shin_core!(vm::VmCtx);
    pub FROM_VM_CTX = from_shin_core!(vm::FromVmCtx);
    pub FROM_VM_CTX_DEFAULT = from_shin_core!(vm::FromVmCtxDefault);
    pub MEMORY_ADDRESS = from_shin_core!(format::scenario::instructions::MemoryAddress);
    pub COMMAND_RESULT = from_shin_core!(vm::command::CommandResult);

    pub BIN_READ = from_binrw!(BinRead);
    pub BIN_WRITE = from_binrw!(BinWrite);
}
