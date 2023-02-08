use crate::sanitization::{
    BIN_READ, BIN_WRITE, COMMAND_RESULT, FROM_VM_CTX, FROM_VM_CTX_DEFAULT, MEMORY_ADDRESS, VM_CTX,
};
use darling::FromMeta;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use synstructure::{Structure, VariantInfo};

#[derive(FromMeta, Default)]
struct CommandFieldMeta {
    #[darling(default)]
    dest: bool,
    #[darling(default)]
    rty: Option<String>,
}

struct CommandField {
    field: syn::Field,
    meta: CommandFieldMeta,
}

#[derive(FromMeta)]
struct CommandVariantMeta {
    opcode: u8,
}

struct CommandVariant {
    name: syn::Ident,
    meta: CommandVariantMeta,
    fields: Vec<CommandField>,
    doc: Option<syn::Attribute>,
}

fn parse_command_variant(input: &VariantInfo) -> CommandVariant {
    let fields = input
        .ast()
        .fields
        .into_iter()
        .map(|field| {
            let meta = field
                .attrs
                .iter()
                .map(|a| a.parse_meta().unwrap())
                .filter(|m| m.path().is_ident("cmd"))
                .map(|m| CommandFieldMeta::from_meta(&m).unwrap())
                .at_most_one()
                .map_err(|_| {
                    syn::Error::new_spanned(field, "Only one #[cmd] attribute is allowed per field")
                })
                .unwrap()
                .unwrap_or_default();
            CommandField {
                field: field.clone(),
                meta,
            }
        })
        .collect();

    let meta = input
        .ast()
        .attrs
        .iter()
        .map(|a| a.parse_meta().unwrap())
        .filter(|m| m.path().is_ident("cmd"))
        .map(|m| CommandVariantMeta::from_meta(&m).unwrap())
        .at_most_one()
        .map_err(|_| {
            syn::Error::new_spanned(
                input.ast().ident,
                "Only one #[cmd] attribute is allowed per variant",
            )
        })
        .unwrap()
        .unwrap();

    let doc = input
        .ast()
        .attrs
        .iter()
        .find(|a| a.path.is_ident("doc"))
        .cloned();

    CommandVariant {
        name: input.ast().ident.clone(),
        meta,
        fields,
        doc,
    }
}

enum TokenKind {
    Unit,
    DestinationAddress(syn::Ident),
}

impl CommandVariant {
    fn get_token_kind(&self) -> TokenKind {
        let dest_field = self
            .fields
            .iter()
            .filter(|f| f.meta.dest)
            .at_most_one()
            .map_err(|_| "Only one field can be marked as a destination")
            .unwrap();

        if let Some(field) = dest_field {
            TokenKind::DestinationAddress(field.field.ident.clone().unwrap())
        } else {
            TokenKind::Unit
        }
    }
}

impl CommandField {
    pub fn runtime_type(&self) -> TokenStream {
        if let Some(ref rty) = self.meta.rty {
            let rty = syn::parse_str::<syn::Type>(rty).unwrap();
            quote!(#rty)
        } else {
            let ty = &self.field.ty;
            quote! {
                <#ty as #FROM_VM_CTX_DEFAULT>::Output
            }
        }
    }

    pub fn as_conv_trait(&self) -> TokenStream {
        if let Some(ref rty) = self.meta.rty {
            let ty = &self.field.ty;
            let rty = syn::parse_str::<syn::Type>(rty).unwrap();
            quote!(<#rty as #FROM_VM_CTX<#ty>>)
        } else {
            let ty = &self.field.ty;
            quote! {
                <#ty as #FROM_VM_CTX_DEFAULT>
            }
        }
    }
}

fn codegen_command_runtime_type(input: &CommandVariant) -> TokenStream {
    let name = &input.name;
    let name_str = name.to_string();
    let fields = input.fields.iter().filter(|f| !f.meta.dest).map(|f| {
        let ident = f.field.ident.as_ref().unwrap();
        let rty = f.runtime_type();
        quote! {
            pub #ident: #rty
        }
    });
    let token_kind = input.get_token_kind();
    let make_token = match token_kind {
        TokenKind::Unit => {
            quote! {
                super::token::#name::new()
            }
        }
        TokenKind::DestinationAddress(field) => {
            quote! {
                super::token::#name::new(input.#field)
            }
        }
    };
    let arms = input.fields.iter().filter(|f| !f.meta.dest).map(|f| {
        let ident = f.field.ident.as_ref().unwrap();
        let cty = f.as_conv_trait();
        quote! {
            #ident: #cty::from_vm_ctx(ctx, input.#ident)
        }
    });
    let display = input
        .fields
        .iter()
        .filter(|f| !f.meta.dest)
        .enumerate()
        .map(|(i, f)| {
            let ident = f.field.ident.as_ref().unwrap();
            if i == 0 {
                quote! {
                    write!(f, " {:?}", self.#ident)?;
                }
            } else {
                quote! {
                    write!(f, ", {:?}", self.#ident)?;
                }
            }
        });
    let doc = input
        .doc
        .as_ref()
        .map(|a| quote!(#a))
        .unwrap_or_else(|| quote!());

    quote! {
        #[derive(Debug)]
        #doc
        pub struct #name {
            pub token: super::token::#name,
            #(#fields),*
        }

        impl #FROM_VM_CTX<super::compiletime::#name> for #name {
            fn from_vm_ctx(ctx: &#VM_CTX, input: super::compiletime::#name) -> Self {
                Self {
                    token: #make_token,
                    #(#arms),*
                }
            }
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", #name_str)?;
                #(#display)*
                Ok(())
            }
        }
    }
}

fn codegen_command_compiletime_type(input: &CommandVariant) -> TokenStream {
    let name = &input.name;
    let fields = input.fields.iter().map(|f| {
        let ident = f.field.ident.as_ref().unwrap();
        let ty = &f.field.ty;
        quote! {
            pub #ident: #ty
        }
    });

    let magic = input.meta.opcode;

    let doc = input
        .doc
        .as_ref()
        .map(|a| quote!(#a))
        .unwrap_or_else(|| quote!());

    quote! {
        #[derive(#BIN_READ, #BIN_WRITE, Debug)]
        #doc
        #[brw(little, magic(#magic))]
        pub struct #name {
            #(#fields),*
        }
    }
}

fn codegen_command_token_type(input: &CommandVariant) -> TokenStream {
    let name = &input.name;
    match input.get_token_kind() {
        TokenKind::Unit => {
            quote! {
                #[derive(Debug)]
                pub struct #name(());
                impl #name {
                    pub(super) fn new() -> Self {
                        Self(())
                    }

                    pub fn finish(self) -> #COMMAND_RESULT {
                        #COMMAND_RESULT::None
                    }
                }
            }
        }
        TokenKind::DestinationAddress(_) => {
            quote! {
                #[derive(Debug)]
                pub struct #name(#MEMORY_ADDRESS);
                impl #name {
                    pub(super) fn new(addr: #MEMORY_ADDRESS) -> Self {
                        Self(addr)
                    }

                    pub fn finish(self, value: i32) -> #COMMAND_RESULT {
                        #COMMAND_RESULT::WriteMemory(self.0, value)
                    }
                }
            }
        }
    }
}

pub fn impl_command(input: Structure) -> TokenStream {
    let variants = input
        .variants()
        .iter()
        .map(parse_command_variant)
        .collect::<Vec<_>>();

    let runtime_types: TokenStream = variants.iter().map(codegen_command_runtime_type).collect();

    let compiletime_types: TokenStream = variants
        .iter()
        .map(codegen_command_compiletime_type)
        .collect();

    let token_types: TokenStream = variants.iter().map(codegen_command_token_type).collect();

    let variant_names: Vec<TokenStream> = variants
        .iter()
        .map(|v| {
            let ident = &v.name;
            quote!(#ident)
        })
        .collect();

    // this is for some reason necessary... Otherwise a strange error in the quote! machinery pops out
    let from_vm_ctx = &FROM_VM_CTX;

    quote! {
        /// This module contains compile-time representation of commands.
        ///
        /// This mostly means that the `token` field is not present and `NumberSpec` is stored as-is.
        pub mod compiletime {
            use super::*;
            #compiletime_types
        }

        /// This module contains compile-time representation of commands.
        ///
        /// Unlike compile-time representation, this one contains `token` field and `NumberSpec` values are resolved to `i32` or other strongly-typed values.
        pub mod runtime {
            use super::*;
            #runtime_types
        }

        /// This module contains types for command tokens.
        ///
        /// Each command has a corresponding token type that is used to finish the command.
        ///
        /// The idea is to enforce in compile-time that the commands that require writing to memory (like `SELECT` or `SGET`) do write to memory.
        pub mod token {
            #token_types
        }

        /// Enum over all possible commands (compile-time representation).
        #[derive(#BIN_READ, #BIN_WRITE, Debug)]
        pub enum CompiletimeCommand {
            #(#variant_names(compiletime::#variant_names)),*
        }

        /// Enum over all possible commands (runtime representation).
        #[derive(Debug)]
        pub enum RuntimeCommand {
            #(#variant_names(runtime::#variant_names)),*
        }

        impl #from_vm_ctx<CompiletimeCommand> for RuntimeCommand {
            #[inline]
            fn from_vm_ctx(ctx: &#VM_CTX, input: CompiletimeCommand) -> Self {
                match input {
                    #(CompiletimeCommand::#variant_names(v) => RuntimeCommand::#variant_names(#from_vm_ctx::from_vm_ctx(ctx, v))),*
                }
            }
        }
        impl #FROM_VM_CTX_DEFAULT for CompiletimeCommand {
            type Output = RuntimeCommand;
        }
        impl std::fmt::Display for RuntimeCommand {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(RuntimeCommand::#variant_names(v) => write!(f, "{}", v)),*
                }
            }
        }
    }
}
