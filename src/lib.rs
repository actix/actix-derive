#![recursion_limit="128"]
#![feature(nll)]
#![feature(vec_remove_item)]
#![cfg_attr(actix_nightly, feature(proc_macro,))]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use] extern crate syn;
#[macro_use] extern crate quote;
extern crate rand;

use proc_macro::TokenStream;
use syn::{DeriveInput, Ident};
use syn::punctuated::Punctuated;
use syn::synom::Synom;
use quote::ToTokens;

mod actor;
mod message;

#[doc(hidden)]
#[proc_macro_derive(Message, attributes(rtype))]
pub fn message_derive_rtype(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    message::expand(&input).into()
}

#[cfg(actix_nightly)]
#[proc_macro_attribute]
pub fn actor(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut ctx = None;
    let mut attr: syn::Expr = syn::parse(attr).unwrap();
    if let syn::Expr::Paren(ref mut paren) = attr {
        if let syn::Expr::Path(ref mut path) = *paren.expr {
            for path in path.path.segments.iter_mut() {
                if let syn::PathArguments::AngleBracketed(ref mut args) = path.arguments {
                    for arg in args.args.iter_mut() {
                        if let &mut syn::GenericArgument::Type(syn::Type::Infer(_)) = arg {
                            std::mem::swap(arg, &mut parse_quote!(Self));
                        }
                    }
                    let path = path.clone();
                    ctx = Some(parse_quote!(#path));
                }
            }
        }
    }

    let mut ast = syn::parse(input).unwrap();
    let expanded = actor::build_handler(&mut ast, ctx);

    let ast = quote!{#ast #expanded};
    // have to erase hygene because of: https://github.com/rust-lang/rust/issues/46489
    //ast.into()
    ast.to_string().parse().unwrap()
}

struct Args {
    vars: Vec<syn::TypePath>,
}

impl Synom for Args {
    named!(parse -> Self, map!(
        parens!(Punctuated::<Ident, Token![,]>::parse_terminated_nonempty),
        |(_parens, vars)| Args {
            vars: vars.into_iter().map(|i| syn::parse(i.into_tokens().into()).unwrap()).collect(),
        }
    ));
}

#[cfg(actix_nightly)]
#[proc_macro_attribute]
pub fn msg(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr: Args = syn::parse(attr).unwrap();
    let mut ast: syn::Item = syn::parse(input).unwrap();
    let expanded = message::message_attr(&mut ast, attr.vars);

    let ast = quote!{#ast #expanded};
    ast.into()
}
