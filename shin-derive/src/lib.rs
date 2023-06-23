// this is noisy & not well-supported by IDEs
#![allow(clippy::uninlined_format_args)]

mod command;
pub(crate) mod sanitization;
mod syntax_kind;
mod texture_archive;
mod util;
mod vertex;

use crate::command::impl_command;
use crate::syntax_kind::impl_syntax_kind;
use crate::syntax_kind::SyntaxKindInput;
use crate::texture_archive::impl_texture_archive;
use crate::vertex::impl_vertex;
use proc_macro::TokenStream;
use synstructure::macros::DeriveInput;

#[proc_macro_derive(Command, attributes(cmd))]
pub fn derive_command(input: TokenStream) -> TokenStream {
    match synstructure::macros::parse::<DeriveInput>(input) {
        Ok(p) => match synstructure::Structure::try_new(&p) {
            Ok(s) => synstructure::MacroResult::into_stream(impl_command(s)),
            Err(e) => e.to_compile_error().into(),
        },
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(TextureArchive, attributes(txa))]
pub fn derive_texture_archive(input: TokenStream) -> TokenStream {
    match synstructure::macros::parse::<DeriveInput>(input) {
        Ok(p) => match synstructure::Structure::try_new(&p) {
            Ok(s) => synstructure::MacroResult::into_stream(impl_texture_archive(s)),
            Err(e) => e.to_compile_error().into(),
        },
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(
    Vertex,
    attributes(
        u8x2, u8x4, s8x2, s8x4, un8x2, un8x4, sn8x2, sn8x4, u16x2, u16x4, s16x2, s16x4, un16x2,
        un16x4, sn16x2, sn16x4, f16x2, f16x4, f32, f32x2, f32x3, f32x4, u32, u32x2, u32x3, u32x4,
        s32, s32x2, s32x3, s32x4, f64, f64x2, f64x3, f64x4, mat2x2, mat2x3, mat2x4, mat3x2, mat3x3,
        mat3x4, mat4x2, mat4x3, mat4x4
    )
)]
pub fn derive_vertex(input: TokenStream) -> TokenStream {
    match synstructure::macros::parse::<DeriveInput>(input) {
        Ok(p) => match synstructure::Structure::try_new(&p) {
            Ok(s) => synstructure::MacroResult::into_stream(impl_vertex(s)),
            Err(e) => e.to_compile_error().into(),
        },
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn syntax_kind(input: TokenStream) -> TokenStream {
    match syn::parse::<SyntaxKindInput>(input) {
        Ok(p) => synstructure::MacroResult::into_stream(impl_syntax_kind(p)),
        Err(e) => e.to_compile_error().into(),
    }
}
