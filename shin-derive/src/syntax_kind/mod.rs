//! Auto-generates the SyntaxKind and some other utilities for the asm lexer and parser.

mod input;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use std::str::FromStr;
use syn::LitChar;

use crate::syntax_kind::input::{MappingItem, SyntaxMapping};
pub use input::{SyntaxKindInput, SyntaxList};

fn generate_syntax_kind_enum(input: &SyntaxKindInput) -> TokenStream {
    fn generate_variant(ident: &Ident, doc: &str) -> TokenStream {
        quote_spanned! { ident.span() =>
            #[doc = #doc]
            #ident,
        }
    }

    fn generate_list(list: &SyntaxList, doc: &str) -> TokenStream {
        let mut variants = TokenStream::new();
        for ident in &list.ident_list {
            variants.extend(generate_variant(ident, doc));
        }
        variants
    }

    fn generate_mapping(mapping: &SyntaxMapping, doc_prefix: &str) -> TokenStream {
        let mut variants = TokenStream::new();
        for item in &mapping.mapping_list {
            variants.extend(generate_variant(
                &item.ident,
                &format!("{}{}", doc_prefix, item.content.value()),
            ));
        }
        variants
    }

    let technical = generate_list(&input.technical, "Technical token, only used for parsing");
    let punct = generate_mapping(&input.punct, "Punctuation: ");
    let literals = generate_list(&input.literals, "A literal");
    let tokens = generate_list(&input.tokens, "A token");
    let nodes = generate_list(&input.nodes, "A syntax node");

    quote! {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
        #[repr(u16)]
        pub enum SyntaxKind {
            #technical
            #punct
            #literals
            #tokens
            #nodes
        }
    }
}

fn generate_t_macro(input: &SyntaxKindInput) -> TokenStream {
    let mut rules = TokenStream::new();

    for MappingItem { ident, content, .. } in &input.punct.mapping_list {
        let matcher = match content.value().as_str() {
            "(" | ")" | "[" | "]" | "{" | "}" => {
                let c = LitChar::new(content.value().chars().next().unwrap(), content.span());
                quote!(#c)
            }
            other => TokenStream::from_str(other).expect("Invalid punct"),
        };

        rules.extend(quote! {
             [#matcher] => {
                $crate::parser::SyntaxKind::#ident
            };
        })
    }

    quote! {
        macro_rules! T {
            #rules
        }
        pub(crate) use T;
    }
}

pub fn impl_syntax_kind(input: SyntaxKindInput) -> TokenStream {
    let syntax_kind_enum = generate_syntax_kind_enum(&input);
    let t_macro = generate_t_macro(&input);

    quote! {
        #syntax_kind_enum
        #t_macro
    }
}
