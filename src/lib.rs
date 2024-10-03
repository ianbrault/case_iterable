/*
** src/lib.rs
**
** Copyright (c) 2024 Ian Brault.
**
** This program is free software: you can redistribute it and/or modify
** it under the terms of the GNU General Public License as published by
** the Free Software Foundation, version 3.
**
** This program is distributed in the hope that it will be useful, but
** WITHOUT ANY WARRANTY; without even the implied warranty of
** MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
** General Public License for more details.
**
** You should have received a copy of the GNU General Public License
** along with this program. If not, see <http://www.gnu.org/licenses/>.
*/

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse_macro_input;

fn ident_to_type(ident: &syn::Ident) -> syn::Type {
    let mut segments = syn::punctuated::Punctuated::new();
    segments.push(syn::PathSegment {
        ident: ident.clone(),
        arguments: syn::PathArguments::None,
    });
    let path = syn::Path {
        leading_colon: None,
        segments,
    };
    let type_path = syn::TypePath { qself: None, path };
    syn::Type::Path(type_path)
}

fn optionized_type(ident: &syn::Ident) -> syn::Type {
    let enum_type = ident_to_type(ident);
    let mut args = syn::punctuated::Punctuated::new();
    args.push(syn::GenericArgument::Type(enum_type));

    let mut segments = syn::punctuated::Punctuated::new();
    segments.push(syn::PathSegment {
        ident: syn::Ident::new("std", Span::call_site()),
        arguments: syn::PathArguments::None,
    });
    segments.push(syn::PathSegment {
        ident: syn::Ident::new("option", Span::call_site()),
        arguments: syn::PathArguments::None,
    });
    segments.push(syn::PathSegment {
        ident: syn::Ident::new("Option", Span::call_site()),
        arguments: syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: syn::Token![<](Span::call_site()),
            args,
            gt_token: syn::Token![>](Span::call_site()),
        }),
    });

    let path = syn::Path {
        leading_colon: None,
        segments,
    };
    let type_path = syn::TypePath { qself: None, path };
    syn::Type::Path(type_path)
}

fn gen_iter_struct(ast: &syn::DeriveInput) -> syn::ItemStruct {
    let enum_name = &ast.ident;
    let iter_struct_name = syn::Ident::new(&format!("{}Iterator", enum_name), Span::call_site());

    let mut named = syn::punctuated::Punctuated::new();
    named.push(syn::Field {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        mutability: syn::FieldMutability::None,
        ident: Some(syn::Ident::new("current", Span::call_site())),
        colon_token: Some(syn::Token![:](Span::call_site())),
        ty: optionized_type(enum_name),
    });
    let fields_named = syn::FieldsNamed {
        brace_token: syn::token::Brace(Span::call_site()),
        named,
    };
    let fields = syn::Fields::Named(fields_named);

    syn::ItemStruct {
        attrs: ast.attrs.clone(),
        struct_token: syn::Token![struct](Span::call_site()),
        vis: ast.vis.clone(),
        ident: iter_struct_name,
        generics: ast.generics.clone(),
        fields,
        semi_token: None,
    }
}

fn gen_enum_path(enum_ident: &syn::Ident, variant_ident: &syn::Ident) -> syn::Path {
    // path for a fully-resolved enum variant i.e. Foo::Bar
    let mut segments = syn::punctuated::Punctuated::new();
    segments.push(syn::PathSegment {
        ident: enum_ident.clone(),
        arguments: syn::PathArguments::None,
    });
    segments.push(syn::PathSegment {
        ident: variant_ident.clone(),
        arguments: syn::PathArguments::None,
    });
    syn::Path {
        leading_colon: None,
        segments,
    }
}

fn gen_match_arm(
    enum_ident: &syn::Ident,
    left: &syn::Ident,
    right: Option<&syn::Ident>,
) -> syn::Arm {
    // format:
    // Enum::A => Some(Enum::B) if not final
    // Enum::X => None if final
    let pat = syn::Pat::Path(syn::ExprPath {
        attrs: Vec::new(),
        qself: None,
        path: gen_enum_path(enum_ident, left),
    });
    let body = Box::new(syn::Expr::Verbatim(if let Some(ident) = right {
        quote! { Some(#enum_ident::#ident) }
    } else {
        quote! { None }
    }));
    syn::Arm {
        attrs: Vec::new(),
        pat,
        guard: None,
        fat_arrow_token: syn::Token![=>](Span::call_site()),
        body,
        comma: Some(syn::Token![,](Span::call_site())),
    }
}

fn gen_match_expr(enum_ident: &syn::Ident, variants: Vec<&syn::Variant>) -> syn::ExprMatch {
    // spot-check for non-unit enum variants
    for variant in variants.iter() {
        if !matches!(variant.fields, syn::Fields::Unit) {
            panic!("Non-unit variant: {}", variant.ident);
        }
    }

    // iterator self.current is an optional that is unwrapped with variable inner
    let match_field = syn::Expr::Verbatim(quote! { inner });
    let arms = variants
        .iter()
        .enumerate()
        .map(|(i, variant)| {
            // Enum::A => Some(Enum::B) if not final
            // Enum::X => None if final
            let left = &variant.ident;
            let right = if i + 1 == variants.len() {
                None
            } else {
                Some(&variants[i + 1].ident)
            };
            gen_match_arm(enum_ident, left, right)
        })
        .collect::<Vec<_>>();

    syn::ExprMatch {
        attrs: Vec::new(),
        match_token: syn::Token![match](Span::call_site()),
        expr: Box::new(match_field),
        brace_token: syn::token::Brace(Span::call_site()),
        arms,
    }
}

#[proc_macro_derive(CaseIterable)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;
    let iter_name = syn::Ident::new(&format!("{}Iterator", name), name.span());

    // generate the <Enum>Iterator struct definition
    let iter_struct = gen_iter_struct(&ast);
    // select enum variants
    let enum_ref = if let syn::Data::Enum(ref enum_ref) = ast.data {
        enum_ref
    } else {
        panic!("Must be derived on an enum");
    };
    let fields = enum_ref.variants.iter().collect::<Vec<_>>();
    let first_field = &fields.first().expect("No enum fields").ident;
    // and generate the match expression used to select the next enum in the iterator
    let iter_match_expr = gen_match_expr(name, fields);

    // produce macro output token stream
    let tokens = quote! {
        #iter_struct

        impl #iter_name {
            fn new(from: #name) -> Self {
                Self { current: Some(from) }
            }
        }

        impl Iterator for #iter_name {
            type Item = #name;

            fn next(&mut self) -> Option<Self::Item> {
                if let Some(inner) = &self.current {
                    let new = #iter_match_expr;
                    std::mem::replace(&mut self.current, new)
                } else {
                    None
                }
            }
        }

        impl #name {
            fn all_cases() -> #iter_name {
                #iter_name::new(#name::#first_field)
            }
        }
    };
    tokens.into()
}
