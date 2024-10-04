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

mod ast;

use ast::Path;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse_macro_input;

fn gen_iter_struct(ast: &syn::DeriveInput) -> syn::ItemStruct {
    let enum_name = &ast.ident;
    let enum_type = ast::Type::path_from_ident(enum_name.clone().into());
    let iter_struct_name = ast::Ident::new(&format!("{}Iterator", enum_name));

    let mut named = syn::punctuated::Punctuated::new();
    named.push(syn::Field {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        mutability: syn::FieldMutability::None,
        ident: Some(ast::Ident::new("current").into()),
        colon_token: Some(syn::Token![:](Span::call_site())),
        ty: enum_type.into_option().into(),
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
        ident: iter_struct_name.into(),
        generics: ast.generics.clone(),
        fields,
        semi_token: None,
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

    let mut enum_path = ast::Path::new();
    enum_path.push(enum_ident.into());
    enum_path.push(left.into());

    let pat = syn::Pat::Path(syn::ExprPath {
        attrs: Vec::new(),
        qself: None,
        path: enum_path.into(),
    });
    let body = ast::Expr::tokens(if let Some(ident) = right {
        quote! { Some(#enum_ident::#ident) }
    } else {
        quote! { None }
    });
    syn::Arm {
        attrs: Vec::new(),
        pat,
        guard: None,
        fat_arrow_token: syn::Token![=>](Span::call_site()),
        body: Box::new(body.into()),
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

    // match on &self
    let self_ident = ast::Ident::new("self");
    let self_expr = ast::Expr::path(Path::with_ident(self_ident));
    let match_field = ast::Expr::reference(self_expr);

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
        expr: Box::new(match_field.into()),
        brace_token: syn::token::Brace(Span::call_site()),
        arms,
    }
}

#[proc_macro_derive(CaseIterable)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    let enum_name = &ast.ident;
    let enum_type = ast::Type::path_from_ident(enum_name.into());
    let enum_option_type: syn::Type = enum_type.into_option().into();
    let iter_name: syn::Ident = ast::Ident::new(&format!("{}Iterator", enum_name)).into();

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
    let iter_match_expr = gen_match_expr(enum_name, fields);

    // produce macro output token stream
    let tokens = quote! {
        impl #enum_name {
            pub fn next(&self) -> #enum_option_type {
                #iter_match_expr
            }

            pub fn all_cases() -> #iter_name {
                #iter_name::new(#enum_name::#first_field)
            }
        }

        #iter_struct

        impl #iter_name {
            fn new(from: #enum_name) -> Self {
                Self { current: Some(from) }
            }
        }

        impl Iterator for #iter_name {
            type Item = #enum_name;

            fn next(&mut self) -> Option<Self::Item> {
                if let Some(inner) = &self.current {
                    let new = inner.next();
                    std::mem::replace(&mut self.current, new)
                } else {
                    None
                }
            }
        }
    };
    tokens.into()
}
