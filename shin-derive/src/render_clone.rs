use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use synstructure::Structure;

use crate::{
    sanitization::{RENDER_CLONE, RENDER_CLONE_CTX},
    util::parse_opt_attribute,
};

#[derive(FromMeta, Default)]
struct RenderCloneFieldMeta {
    #[darling(default)]
    needs_render: bool,
}

pub fn impl_render_clone(structure: Structure) -> TokenStream {
    let body = structure.each_variant(|variant| {
        variant.construct(|field, i| {
            let binding = &variant.bindings()[i];
            let binding = &binding.binding;

            let meta =
                parse_opt_attribute::<RenderCloneFieldMeta>(&field, "render_clone", &field.attrs)
                    .unwrap()
                    .unwrap_or_default();

            if meta.needs_render {
                quote! {
                    #RENDER_CLONE::render_clone(#binding, ctx)
                }
            } else {
                quote! {
                    ::core::clone::Clone::clone(#binding)
                }
            }
        })
    });

    structure.gen_impl(quote! {
        gen impl #RENDER_CLONE for @Self {
            fn render_clone(&self, ctx: &mut #RENDER_CLONE_CTX) -> Self {
                match self {
                    #body
                }
            }
        }
    })
}
