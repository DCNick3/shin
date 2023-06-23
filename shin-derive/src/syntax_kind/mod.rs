mod input;

pub use input::SyntaxKindInput;
use proc_macro2::TokenStream;
use quote::quote;

pub fn impl_syntax_kind(input: SyntaxKindInput) -> TokenStream {
    println!("{:#?}", input);

    quote!()
}
