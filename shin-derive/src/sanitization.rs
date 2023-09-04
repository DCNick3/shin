//! This module contains helpers to refer to items from crates used by macros.
// why was it called sanitization, again?

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
        // TODO: we need some hack to not require `extern crate self as shin_asm;` and `extern crate self as shin_core;`
        // `proc_macro_crate` could help, but
        // 1. it has problems detecting the crate name for some reason
        // 2. it's hard to report errors from context of ToTokens

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

macro_rules! from_shin_asm {
    ($path:path) => {
        concat!("shin_asm::", stringify!($path))
    };
}

macro_rules! from_shin_render {
    ($path:path) => {
        concat!("shin_render::", stringify!($path))
    };
}

macro_rules! from_shin {
    ($path:path) => {
        concat!("shin::", stringify!($path))
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

    pub TEXTURE_ARCHIVE = from_shin!(asset::texture_archive::TextureArchive);
    pub TEXTURE_ARCHIVE_BUILDER = from_shin!(asset::texture_archive::TextureArchiveBuilder);
    pub LAZY_GPU_TEXTURE = from_shin_render!(LazyGpuTexture);

    pub BIN_READ = from_binrw!(BinRead);
    pub BIN_WRITE = from_binrw!(BinWrite);

    pub SYNTAX_KIND = from_shin_asm!(syntax::SyntaxKind);

    pub AST_NODE = from_shin_asm!(syntax::AstNode);
    pub AST_TOKEN = from_shin_asm!(syntax::AstToken);

    pub SYNTAX_NODE = from_shin_asm!(syntax::SyntaxNode);
    pub SYNTAX_TOKEN = from_shin_asm!(syntax::SyntaxToken);
}
