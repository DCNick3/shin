use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_quote, Meta, Token};
use synstructure::{BindingInfo, Structure, VariantInfo};

use crate::sanitization::AST_NODE;

struct AstNodeInput<'a> {
    structure: Structure<'a>,
    variant_attrs: Vec<AstAttributeContents>,
}

struct AstAttributeContents {
    kind: syn::Path,
}

impl AstAttributeContents {
    fn from_attributes(attrs: &[syn::Attribute], span: Span) -> Result<Self, TokenStream> {
        let Some(attr) = attrs.iter().find(|a| a.path() == &parse_quote!(ast))
        else {
            return Err(quote_spanned!{ span =>
                ::core::compile_error!("expected #[ast(...)]")
            })
        };

        match &attr.meta {
            Meta::List(meta) => {
                Ok(syn::parse2(meta.tokens.clone()).map_err(|e| e.to_compile_error())?)
            }
            _ => Err(quote_spanned! { attr.span() =>
                ::core::compile_error!("expected #[ast(...)]")
            }),
        }
    }
}

impl Parse for AstAttributeContents {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        struct Item {
            key: syn::Path,
            #[allow(dead_code)]
            eq_token: Token![=],
            value: TokenStream,
        }

        impl Parse for Item {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                Ok(Item {
                    key: input.parse()?,
                    eq_token: input.parse()?,
                    value: input.step(|cursor| {
                        let mut buffer = TokenStream::new();
                        let mut cursor = *cursor;
                        while let Some((tt, next)) = cursor.token_tree() {
                            cursor = next;
                            match tt {
                                TokenTree::Punct(punct) if punct.as_char() == ',' => break,
                                other => buffer.extend(Some(other)),
                            }
                        }

                        Ok((buffer, cursor))
                    })?,
                })
            }
        }

        let items = Punctuated::<Item, Token![,]>::parse_terminated(input)?.into_iter();

        let mut kind = None;

        let mut errors = Vec::new();

        for Item { key, value, .. } in items {
            if key == parse_quote!(kind) {
                if kind.is_some() {
                    errors.push(syn::Error::new(key.span(), "duplicate key"));
                }
                kind = Some(value);
            } else {
                errors.push(syn::Error::new(key.span(), "unknown key"));
            }
        }

        if kind.is_none() {
            errors.push(syn::Error::new(Span::call_site(), "missing key `kind`"));
        }

        if !errors.is_empty() {
            let mut combined_error = syn::Error::new(input.span(), "invalid #[ast(...)]");
            for error in errors {
                combined_error.combine(error);
            }
            return Err(combined_error);
        }

        Ok(Self {
            kind: syn::parse2(kind.unwrap())?,
        })
    }
}

#[test]
fn parse_ast_attr() {
    let s: syn::ItemStruct = parse_quote! {
        #[ast(kind = SOURCE_FILE)]
        struct SourceFile {}
    };
    let s = AstAttributeContents::from_attributes(&s.attrs, s.span()).unwrap();

    assert_eq!(s.kind, parse_quote!(SOURCE_FILE));
}

fn get_syntax_field<'a>(variant: &'a VariantInfo) -> &'a BindingInfo<'a> {
    let [binding] = variant.bindings() else {
        unreachable!()
    };

    binding
}

fn gen_can_cast(input: &AstNodeInput) -> TokenStream {
    let tokens = input.variant_attrs.iter().map(|attr| {
        let kind = &attr.kind;
        quote! {
            if kind == #kind {
                return true;
            }
        }
    });

    quote! {
        #( #tokens )*
        false
    }
}

fn gen_cast(input: &AstNodeInput) -> TokenStream {
    let tokens = input
        .structure
        .variants()
        .iter()
        .zip(input.variant_attrs.iter())
        .map(|(variant, attr)| {
            let kind = &attr.kind;
            let construct = variant.construct(|_, _| quote!(syntax));

            quote! {
                if syntax.kind() == #kind {
                    return Some(#construct);
                }
            }
        });

    quote! {
        #( #tokens )*
        None
    }
}

fn gen_syntax(input: &AstNodeInput) -> TokenStream {
    let body = input.structure.each_variant(|variant| {
        let syntax = get_syntax_field(variant);
        quote! { &#syntax }
    });

    quote! {
        match self {
            #body
        }
    }
}

pub fn impl_ast_node(structure: Structure) -> TokenStream {
    let mut variant_attrs = Vec::new();

    let error = structure.variants().iter().map(|variant| {
        let binding_err = if variant.bindings().len() != 1 {
            let span = variant.ast().ident.span();
            let (span, error) = match variant.prefix {
                Some(ident) => (ident.span(), "this enum variant has zero or more than one field!\nIt should only have a single field if type SyntaxNode.".to_string()),
                None => (span, "this ast structure has zero or more than one field!\nIt should only have a single field if type SyntaxNode.".to_string()),
            };
            quote_spanned! { span => ::core::compile_error!(#error) }
        } else {
            quote! {}
        };

        let attr_err = match AstAttributeContents::from_attributes(variant.ast().attrs, variant.ast().ident.span()) {
            Ok(attr) => {
                variant_attrs.push(attr);
                quote!()
            },
            Err(e) => e,
        };

        quote! {
            #binding_err
            #attr_err
        }
    }).collect::<TokenStream>();

    if !error.is_empty() {
        return error;
    }

    let input = AstNodeInput {
        structure,
        variant_attrs,
    };

    let can_cast_impl = gen_can_cast(&input);
    let cast_impl = gen_cast(&input);
    let syntax_impl = gen_syntax(&input);

    input.structure.gen_impl(quote! {
        gen impl #AST_NODE for @Self {
            fn can_cast(kind: SyntaxKind) -> bool
            where
                Self: Sized,
            { #can_cast_impl }

            fn cast(syntax: SyntaxNode) -> Option<Self>
            where
                Self: Sized,
            { #cast_impl }

            fn syntax(&self) -> &SyntaxNode
            { #syntax_impl }
        }
    })
}

#[cfg(test)]
#[test]
fn test_ast_node() {
    use prettyplease::unparse;

    assert_eq!(
        unparse(
            &syn::parse2(impl_ast_node(
                Structure::try_new(&parse_quote! {
                    #[ast(kind = SOURCE_FILE)]
                    struct SourceFile {
                        syntax: SyntaxNode,
                    }
                })
                .unwrap()
            ))
            .unwrap()
        ),
        unparse(&parse_quote! {
            #[allow(non_upper_case_globals)]
            const _DERIVE_shin_asm_syntax_AstNode_FOR_SourceFile: () = {
                impl shin_asm::syntax::AstNode for SourceFile {
                    fn can_cast(kind: SyntaxKind) -> bool
                    where
                        Self: Sized,
                    {
                        if kind == SOURCE_FILE {
                            return true;
                        }
                        false
                    }
                    fn cast(syntax: SyntaxNode) -> Option<Self>
                    where
                        Self: Sized,
                    {
                        if syntax.kind() == SOURCE_FILE {
                            return Some(SourceFile { syntax: syntax });
                        }
                        None
                    }
                    fn syntax(&self) -> &SyntaxNode {
                        match self {
                            SourceFile {
                                syntax: ref __binding_0,
                            } => &__binding_0,
                        }
                    }
                }
            };
        })
    );
}
