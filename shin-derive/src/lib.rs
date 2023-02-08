// this is noisy & not well-supported by IDEs
#![allow(clippy::uninlined_format_args)]

mod command;
pub(crate) mod sanitization;
mod texture_archive;

use crate::command::impl_command;
use crate::texture_archive::impl_texture_archive;
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
