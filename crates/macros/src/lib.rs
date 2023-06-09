/*
 * Copyright (C) 2023 James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

#![feature(proc_macro_diagnostic)]

use proc_macro::Level::Error;
use proc_macro::{Diagnostic, Level, Span, TokenStream};
use quote::__private::ext::RepToTokensExt;
use quote::{quote, ToTokens};
use std::path::PathBuf;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Meta, Type};

#[proc_macro_derive(CommonFields)]
pub fn conditional_fields_macro(input: TokenStream) -> TokenStream {
    fn error(span: Span) -> TokenStream {
        let err =
            "Derive macro CommonFields requires all enums be variants with at least one field.";
        Diagnostic::spanned(span, Level::Error, err).emit();
        TokenStream::new()
    }

    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Extract the enum name
    let enum_name = input.ident;

    let (item_enum, variants) = match &input.data {
        Data::Enum(item_enum) => (item_enum, &item_enum.variants),
        _ => return error(enum_name.span().unwrap()),
    };

    // Extract the common field names and types from the enum variants
    if variants.is_empty() {
        return error(item_enum.enum_token.span().unwrap());
    }

    let mut global_fields = get_fields_from_variant(&variants[0]);
    if global_fields.is_empty() {
        return error(variants[0].span().unwrap());
    }

    for variant in variants.iter().skip(1) {
        let fields = get_fields_from_variant(variant);
        if fields.is_empty() {
            return error(variant.span().unwrap());
        }

        // Will remove all fields that are not in the current variant
        global_fields.retain(|(field_name, field_type)| {
            fields.iter().any(|(other_field_name, other_field_type)| {
                field_name == other_field_name && field_type == other_field_type
            })
        });
    }

    let common_fields = global_fields;

    // Generate the conditional fields
    let functions = common_fields.iter().map(|(field_name, field_type)| {
        let enum_branches = variants.iter().map(|variant| {
            let variant_name = &variant.ident;
            quote! {
                #enum_name::#variant_name { #field_name, .. } => #field_name,
            }
        });

        quote! {
            #[automatically_derived]
            pub const fn #field_name(&self) -> &#field_type {
                match self {
                    #(#enum_branches)*
                }
            }
        }
    });

    let output = quote! {
        #[automatically_derived]
        impl #enum_name {
            #(#functions)*
        }
    };

    // Convert the generated code back into tokens and return them
    output.into()
}

// #[proc_macro_derive(FromCommand, attributes(command))]
// pub fn from_command(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     let struct_name = &input.ident;
//     let struct_data = match input.data {
//         Data::Struct(data) => data,
//         _ => panic!("FromCommand can only be derived for structs"),
//     };
//
//     let fields = match struct_data.fields {
//         Fields::Named(fields) => fields,
//         _ => {
//             return syn::Error::new(
//                 struct_name.span(),
//                 "FromCommand can only be derived for structs with named fields",
//             )
//             .to_compile_error()
//             .into()
//         }
//     };
//
//     let attributes = &input.attrs;
//     let binary: PathBuf = parse_attribute_path(&attributes, "binary");
//     let args: Vec<String> = parse_attribute_vec(&attributes, "args");
//     let env: Vec<String> = parse_attribute_tuples(&attributes, "env");
// }
//
// // Helper functions to parse attribute values
//
// fn parse_attribute_path<T>(attributes: &[syn::Attribute], name: &str) -> Option<T>
// where
//     T: syn::parse::Parse,
// {
//     for attr in attributes {
//         let meta_list = match &attr.meta {
//             Meta::List(m) => m,
//             _ => continue,
//         };
//
//         let result: Option<T> = None;
//         meta_list.parse_nested_meta(|m| match m {
//             Meta::NameValue(value) => {
//                 if result.is_some() {
//                     return Ok(()); // continue
//                 }
//
//                 if !value.path.is_ident(name) {
//                     return Ok(()); // continue
//                 }
//
//                 if let syn::Expr::Path(path) = value.value {
//                     let parsed = path.parse::<T>();
//                     parsed.map_err(|e| {
//                         Diagnostic::spanned(value.span(), Error, e.to_string()).emit();
//                     });
//                 }
//
//                 Ok(())
//             }
//             _ => Ok(()),
//         })?;
//
//         result;
//     }
//
//     None
// }
//
// fn parse_attribute_vec(attributes: &[syn::Attribute], name: &str) -> Vec<String> {
//     for attr in attributes {
//         if let Ok(syn::Meta::List(meta_list)) = attr.parse_meta() {
//             if meta_list.path.is_ident(name) {
//                 if let Some(syn::NestedMeta::Lit(syn::Lit::Str(lit))) = meta_list.nested.first() {
//                     return lit.value().split(',').map(|s| s.trim().to_string()).collect();
//                 }
//             }
//         }
//     }
//
//     Vec::new()
// }
//
// fn parse_attribute_tuples(attributes: &[syn::Attribute], name: &str) -> Vec<(String, String)> {
//     for attr in attributes {
//         if let Ok(syn::Meta::List(meta_list)) = attr.parse_meta() {
//             if meta_list.path.is_ident(name) {
//                 return meta_list
//                     .nested
//                     .iter()
//                     .filter_map(|meta| {
//                         if let syn::NestedMeta::Meta(syn::Meta::NameValue(value)) = meta {
//                             if let (syn::Lit::Str(key), syn::Lit::Str(val)) =
//                                 (&value.path.segments.last().unwrap().ident, &value.lit)
//                             {
//                                 return Some((key.to_string(), val.value()));
//                             }
//                         }
//                         None
//                     })
//                     .collect();
//             }
//         }
//     }
//
//     Vec::new()
// }

// #[proc_macro_derive(Pathed, attributes(pathed))]
// pub fn pathed_macro(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//
//     let struct_name = input.ident;
//     let name = input
//         .attrs
//         .iter()
//         .filter_map(|a| a.meta.require_list().ok())
//         .find_map(|attr| match attr.meta {
//             Meta::NameValue(ref meta) if meta.path.is_ident("name") => Some(&meta.path),
//             _ => None,
//         })
//         .expect("expected `name` attribute");
//     let type_name = input
//         .attrs
//         .iter()
//         .find_map(|attr| match attr.meta {
//             Meta::Path(ref meta) if meta.is_ident("type") => Some(meta),
//             _ => None,
//         })
//         .expect("expected `type` attribute");
//
//     let output = quote! {
//         impl Pathed<#type_name> for #struct_name {
//             const NAME: &'static str = stringify!(#name);
//
//             fn unique_dir(&self, ref from: Self::TYPE) -> PathBuf {
//                 Self::base_dir(from).join(self)
//             }
//         }
//     };
//
//     output.into()
// }
//
// // Helper function to extract attribute values
// fn get_attribute_value(attrs: &[Attribute], attr_name: &str) -> Option<String> {
//     for attr in attrs {
//         let segments = &attr.path().segments;
//         for segment in segments {
//             let attr_name = segment.ident.to_string();
//             if attr_name != attr_name {
//                 continue;
//             }
//
//             let t = attr.to_token_stream().into_iter().collect::<Vec<TokenTree>>();
//             let stream = if let TokenTree::Group(group) = &t[0] {
//                 group.stream()
//             } else {
//                 unimplemented!()
//             };
//
//
//         }
//
//         let attr = attr.to_owned();
//         let meta = match attr.meta {
//             Meta::List(meta) => meta,
//             _ => continue,
//         };
//
//         let mut value: Option<String> = None;
//         meta.parse_nested_meta(|meta| {
//             if meta.path.is_ident(attr_name) {
//                 let unparsed = meta.value().expect("Failed to get meta value");
//                 let lit = syn::Lit::parse(unparsed).expect("Failed to parse to literal");
//                 let val = match lit {
//                     syn::Lit::Str(lit) => lit.value(),
//                     _ => return Err(meta.error("expected string literal")),
//                 };
//
//                 value = Some(val);
//                 Ok(())
//             } else {
//                 Ok(())
//             }
//         })
//         .expect("failed to parse attribute");
//
//         if let Some(value) = value {
//             return Some(value);
//         }
//     }
//
//     None
// }

// Helper function get get all fields from a enum variant
fn get_fields_from_variant(variant: &syn::Variant) -> Vec<(Ident, &Type)> {
    match &variant.fields {
        Fields::Unit => vec![],
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap().clone();
                let field_type = &field.ty;
                (field_name, field_type)
            })
            .collect::<Vec<(Ident, &Type)>>(),
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let field_name = Ident::new(&format!("field{}", index), field.span());
                let field_type = &field.ty;
                (field_name, field_type)
            })
            .collect::<Vec<(Ident, &Type)>>(),
    }
}

// #[proc_macro_derive(CommandFiller)]
