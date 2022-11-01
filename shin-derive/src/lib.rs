mod command;
pub(crate) mod sanitization;

use crate::command::impl_command;
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
