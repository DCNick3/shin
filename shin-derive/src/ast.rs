use crate::sanitization::{
    AST_NODE, AST_SPANNED, AST_TOKEN, SYNTAX_KIND, SYNTAX_NODE, SYNTAX_TOKEN, TEXT_RANGE,
};
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_quote, Meta, Token};
use synstructure::{BindingInfo, Structure, VariantInfo};

#[derive(Debug, Copy, Clone)]
pub enum AstKind {
    Node,
    Token,
}

impl AstKind {
    fn trait_ident(&self) -> crate::sanitization::IdentStr {
        match self {
            AstKind::Node => AST_NODE,
            AstKind::Token => AST_TOKEN,
        }
    }

    fn syntax_kind_ty(&self) -> crate::sanitization::IdentStr {
        match self {
            AstKind::Node => SYNTAX_NODE,
            AstKind::Token => SYNTAX_TOKEN,
        }
    }
}

struct AstNodeInput<'a> {
    structure: Structure<'a>,
    variant_attrs: Vec<AstAttributeContents>,
}

#[derive(Debug, PartialEq, Eq)]
enum AstAttributeContents {
    AstNode { kind: syn::Path },
    Transparent,
}

impl AstAttributeContents {
    fn from_attributes(attrs: &[syn::Attribute], span: Span) -> Result<Self, TokenStream> {
        let Some(attr) = attrs.iter().find(|a| a.path() == &parse_quote!(ast)) else {
            return Err(quote_spanned! { span =>
                ::core::compile_error!("expected #[ast(...)]")
            });
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
        struct ItemValue {
            #[allow(dead_code)]
            eq_token: Token![=],
            value: TokenStream,
        }

        struct Item {
            key: syn::Path,
            value: Option<ItemValue>,
        }

        impl Parse for Item {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let key = input.parse()?;
                if input.peek(Token![=]) {
                    Ok(Item {
                        key,
                        value: Some(ItemValue {
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
                        }),
                    })
                } else {
                    Ok(Item { key, value: None })
                }
            }
        }

        let items = Punctuated::<Item, Token![,]>::parse_terminated(input)?.into_iter();

        let mut kind = None;
        let mut transparent = None;

        let mut errors = Vec::new();

        for Item { key, value, .. } in items {
            if key == parse_quote!(kind) {
                if kind.is_some() {
                    errors.push(syn::Error::new(key.span(), "duplicate key"));
                }
                kind = Some(value);
            } else if key == parse_quote!(transparent) {
                if transparent.is_some() {
                    errors.push(syn::Error::new(key.span(), "duplicate key"));
                }
                transparent = Some(value);
            } else {
                errors.push(syn::Error::new(key.span(), "unknown key"));
            }
        }

        if kind.is_none() && transparent.is_none() || kind.is_some() && transparent.is_some() {
            errors.push(syn::Error::new(
                Span::call_site(),
                "either #[ast(kind = ...)] or #[ast(transparent)] must be specified",
            ));
        }

        if kind.as_ref().is_some_and(|v| v.is_none()) {
            errors.push(syn::Error::new(Span::call_site(), "kind must have a value"));
        }

        if !errors.is_empty() {
            let mut combined_error = syn::Error::new(input.span(), "invalid #[ast(...)]");
            for error in errors {
                combined_error.combine(error);
            }
            return Err(combined_error);
        }

        Ok(if let Some(kind) = kind {
            AstAttributeContents::AstNode {
                kind: syn::parse2(kind.unwrap().value)?,
            }
        } else {
            AstAttributeContents::Transparent
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

    assert_eq!(
        s,
        AstAttributeContents::AstNode {
            kind: parse_quote!(SOURCE_FILE),
        }
    );
}

fn get_inner_field<'a>(variant: &'a VariantInfo) -> &'a BindingInfo<'a> {
    let [binding] = variant.bindings() else {
        unreachable!()
    };

    binding
}

fn gen_can_cast(input: &AstNodeInput, ast_kind: AstKind) -> TokenStream {
    let trait_ident = ast_kind.trait_ident();

    let tokens = input
        .structure
        .variants()
        .iter()
        .zip(input.variant_attrs.iter())
        .map(|(variant, attr)| match attr {
            AstAttributeContents::AstNode { kind } => {
                quote! {
                    if kind == #kind {
                        return true;
                    }
                }
            }
            AstAttributeContents::Transparent => {
                let ty = &variant.ast().fields.iter().next().unwrap().ty;
                quote! {
                    if <#ty as #trait_ident>::can_cast(kind) {
                        return true;
                    }
                }
            }
        });

    quote! {
        #( #tokens )*
        false
    }
}

fn gen_cast(input: &AstNodeInput, ast_kind: AstKind) -> TokenStream {
    let trait_ident = ast_kind.trait_ident();

    let tokens = input
        .structure
        .variants()
        .iter()
        .zip(input.variant_attrs.iter())
        .map(|(variant, attr)| match attr {
            AstAttributeContents::AstNode { kind } => {
                let construct = variant.construct(|_, _| quote!(syntax));

                quote! {
                    if syntax.kind() == #kind {
                        return Some(#construct);
                    }
                }
            }
            AstAttributeContents::Transparent => {
                let ty = &variant.ast().fields.iter().next().unwrap().ty;
                let construct = variant.construct(|_, _| quote!(inner));
                quote! {
                    if let Some(inner) = <#ty as #trait_ident>::cast(syntax.clone()) {
                        return Some(#construct);
                    }
                }
            }
        });

    quote! {
        #( #tokens )*
        None
    }
}

fn gen_syntax(input: &AstNodeInput, ast_kind: AstKind) -> TokenStream {
    let trait_ident = ast_kind.trait_ident();

    let mut attr = input.variant_attrs.iter();
    let body = input.structure.each_variant(|variant| {
        let attr = attr.next().unwrap();

        let inner_field = get_inner_field(variant);
        match attr {
            AstAttributeContents::AstNode { .. } => {
                quote! { &#inner_field }
            }
            AstAttributeContents::Transparent => {
                let ty = &variant.ast().fields.iter().next().unwrap().ty;
                quote! {
                    <#ty as #trait_ident>::syntax(#inner_field)
                }
            }
        }
    });

    quote! {
        match self {
            #body
        }
    }
}

pub fn impl_ast(structure: Structure, ast_kind: AstKind) -> TokenStream {
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

    let can_cast_impl = gen_can_cast(&input, ast_kind);
    let cast_impl = gen_cast(&input, ast_kind);
    let syntax_impl = gen_syntax(&input, ast_kind);

    let trait_ident = ast_kind.trait_ident();
    let syntax_kind_ty = ast_kind.syntax_kind_ty();
    input.structure.gen_impl(quote! {
        gen impl #AST_SPANNED for @Self {
            fn text_range(&self) -> #TEXT_RANGE {
                <Self as #trait_ident>::syntax(self).text_range()
            }
        }

        gen impl #trait_ident for @Self {
            fn can_cast(kind: #SYNTAX_KIND) -> bool
            where
                Self: Sized,
            { #can_cast_impl }

            fn cast(syntax: #syntax_kind_ty) -> Option<Self>
            where
                Self: Sized,
            { #cast_impl }

            fn syntax(&self) -> &#syntax_kind_ty
            { #syntax_impl }
        }
        gen impl ::core::fmt::Display for @Self {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Display::fmt(self.syntax(), f)
            }
        }
    })
}

#[cfg(test)]
#[test]
fn test_ast_node() {
    use prettyplease::unparse;

    assert_eq!(
        unparse(
            &syn::parse2(impl_ast(
                Structure::try_new(&parse_quote! {
                    #[ast(kind = SOURCE_FILE)]
                    struct SourceFile {
                        syntax: SyntaxNode,
                    }
                })
                .unwrap(),
                AstKind::Node
            ))
            .unwrap()
        ),
        unparse(&parse_quote! {
            #[allow(non_upper_case_globals)]
            const _DERIVE_shin_asm_syntax_AstNode_FOR_SourceFile: () = {
                impl shin_asm::syntax::AstNode for SourceFile {
                    fn can_cast(kind: shin_asm::syntax::SyntaxKind) -> bool
                    where
                        Self: Sized,
                    {
                        if kind == SOURCE_FILE {
                            return true;
                        }
                        false
                    }
                    fn cast(syntax: shin_asm::syntax::SyntaxNode) -> Option<Self>
                    where
                        Self: Sized,
                    {
                        if syntax.kind() == SOURCE_FILE {
                            return Some(SourceFile { syntax: syntax });
                        }
                        None
                    }
                    fn syntax(&self) -> &shin_asm::syntax::SyntaxNode {
                        match self {
                            SourceFile {
                                syntax: ref __binding_0,
                            } => &__binding_0,
                        }
                    }
                }
                impl ::core::fmt::Display for SourceFile {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        ::core::fmt::Display::fmt(self.syntax(), f)
                    }
                }
            };
        })
    );
}

#[cfg(test)]
#[test]
fn test_transparent() {
    use prettyplease::unparse;

    assert_eq!(
        unparse(
            &syn::parse2(impl_ast(
                Structure::try_new(&parse_quote! {
                    enum SourceFileItem {
                        #[ast(transparent)]
                        InstructionBlock(InstructionBlock),
                        #[ast(transparent)]
                        FunctionDefinition(FunctionDefinition),
                    }
                })
                .unwrap(),
                AstKind::Node
            ))
            .unwrap()
        ),
        unparse(&parse_quote! {
            #[allow(non_upper_case_globals)]
            const _DERIVE_shin_asm_syntax_AstNode_FOR_SourceFileItem: () = {
                impl shin_asm::syntax::AstNode for SourceFileItem {
                    fn can_cast(kind: shin_asm::syntax::SyntaxKind) -> bool
                    where
                        Self: Sized,
                    {
                        if <InstructionBlock as shin_asm::syntax::AstNode>::can_cast(kind) {
                            return true;
                        }
                        if <FunctionDefinition as shin_asm::syntax::AstNode>::can_cast(kind) {
                            return true;
                        }
                        false
                    }
                    fn cast(syntax: shin_asm::syntax::SyntaxNode) -> Option<Self>
                    where
                        Self: Sized,
                    {
                        if let Some(inner)
                            = <InstructionBlock as shin_asm::syntax::AstNode>::cast(syntax.clone()) {
                            return Some(SourceFileItem::InstructionBlock(inner));
                        }
                        if let Some(inner)
                            = <FunctionDefinition as shin_asm::syntax::AstNode>::cast(syntax.clone()) {
                            return Some(SourceFileItem::FunctionDefinition(inner));
                        }
                        None
                    }
                    fn syntax(&self) -> &shin_asm::syntax::SyntaxNode {
                        match self {
                            SourceFileItem::InstructionBlock(ref __binding_0) => {
                                <InstructionBlock as shin_asm::syntax::AstNode>::syntax(__binding_0)
                            }
                            SourceFileItem::FunctionDefinition(ref __binding_0) => {
                                <FunctionDefinition as shin_asm::syntax::AstNode>::syntax(__binding_0)
                            }
                        }
                    }
                }
                impl ::core::fmt::Display for SourceFileItem {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        ::core::fmt::Display::fmt(self.syntax(), f)
                    }
                }
            };
        })
    );
}
