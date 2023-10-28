use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data};
use synstructure::Structure;

// TODO: implement vertex macro
// it would be a replacement for sometimes clunky wrld
pub fn impl_vertex(input: Structure) -> TokenStream {
    match input.ast().data {
        Data::Struct(_) => {}
        _ => {
            let e = syn::Error::new(input.ast().span(), "Only one variant is allowed")
                .to_compile_error();
            return quote! {
                #e
            };
        }
    }

    // input.gen_impl()

    todo!()
}
