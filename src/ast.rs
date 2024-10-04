/*
** src/ast.rs
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

use proc_macro2::{Span, TokenStream};
use syn::punctuated::Punctuated;
use syn::PathArguments;

#[derive(Clone)]
pub struct Ident(syn::Ident);

impl Ident {
    pub fn new(name: &str) -> Self {
        Self(syn::Ident::new(name, Span::call_site()))
    }
}

impl From<syn::Ident> for Ident {
    fn from(value: syn::Ident) -> Self {
        Self(value)
    }
}

impl From<&syn::Ident> for Ident {
    fn from(value: &syn::Ident) -> Self {
        Self(value.clone())
    }
}

impl From<Ident> for syn::Ident {
    fn from(value: Ident) -> Self {
        value.0
    }
}

pub struct Path {
    segments: Punctuated<syn::PathSegment, syn::token::PathSep>,
}

impl Path {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            segments: Punctuated::new(),
        }
    }

    pub fn with_ident(ident: Ident) -> Self {
        let mut path = Self::new();
        path.push(ident);
        path
    }

    pub fn push(&mut self, ident: Ident) {
        self.segments.push(syn::PathSegment {
            ident: ident.into(),
            arguments: PathArguments::None,
        });
    }

    pub fn push_generic(&mut self, ident: Ident, ty: Type) {
        let mut args = Punctuated::new();
        args.push(syn::GenericArgument::Type(ty.into()));

        self.segments.push(syn::PathSegment {
            ident: ident.into(),
            arguments: PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: syn::Token![<](Span::call_site()),
                args,
                gt_token: syn::Token![>](Span::call_site()),
            }),
        });
    }
}

impl From<Path> for syn::Path {
    fn from(value: Path) -> Self {
        syn::Path {
            leading_colon: None,
            segments: value.segments,
        }
    }
}

pub struct Type(syn::Type);

impl Type {
    pub fn path(path: Path) -> Self {
        let type_path = syn::TypePath {
            qself: None,
            path: path.into(),
        };
        Self(syn::Type::Path(type_path))
    }

    pub fn path_from_ident(ident: Ident) -> Self {
        let path = Path::with_ident(ident);
        Self::path(path)
    }

    pub fn into_option(self) -> Self {
        let mut path = Path::new();
        path.push(Ident::new("std"));
        path.push(Ident::new("option"));
        path.push_generic(Ident::new("Option"), self);

        Self::path(path)
    }
}

impl From<Type> for syn::Type {
    fn from(value: Type) -> Self {
        value.0
    }
}

pub struct Expr(syn::Expr);

impl Expr {
    pub fn tokens(tokens: TokenStream) -> Self {
        Self(syn::Expr::Verbatim(tokens))
    }

    pub fn path(path: Path) -> Self {
        Self(syn::Expr::Path(syn::ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: path.into(),
        }))
    }

    pub fn reference(expr: Self) -> Self {
        Self(syn::Expr::Reference(syn::ExprReference {
            attrs: Vec::new(),
            and_token: syn::Token![&](Span::call_site()),
            mutability: None,
            expr: Box::new(expr.into()),
        }))
    }
}

impl From<Expr> for syn::Expr {
    fn from(value: Expr) -> Self {
        value.0
    }
}
