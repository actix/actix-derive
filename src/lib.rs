#![cfg_attr(actix_nightly, feature(proc_macro,))]

extern crate rand;
extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;

use std::str::FromStr;
use proc_macro::TokenStream;
use quote::{Tokens, ToTokens};

mod actor;
mod message;

#[doc(hidden)]
#[proc_macro_derive(Message, attributes(rtype))]
pub fn message_derive_rtype(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    message::expand(&ast).parse().expect("Expanded output was no correct Rust code")
}

fn get_attribute_type_multiple(ast: &syn::DeriveInput, name: &str) -> Option<Vec<Option<syn::Ty>>>
{
    let attr = ast.attrs.iter().find(|a| a.name() == name);

    if attr.is_none() {
        return None;
    }

    let attr = attr.unwrap();

    if let syn::MetaItem::List(_, ref vec) = attr.value {
        Some(vec.iter().map(|m| meta_item_to_ty(m, name)).collect())
    } else {
        panic!("The correct syntax is #[{}(type, type, ...)]", name);
    }
}

fn meta_item_to_ty(meta_item: &syn::NestedMetaItem, name: &str) -> Option<syn::Ty> {
    if let syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref i)) = *meta_item {
        let ty = syn::parse::ty(i.as_ref());
        match ty {
            syn::parse::IResult::Done(_, ty) => Some(ty),
            _ => None,
        }
    } else {
        panic!("The correct syntax is #[{}(type)]", name);
    }
}

#[cfg(actix_nightly)]
#[proc_macro_attribute]
pub fn actor(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let source = input.to_string();

    let mut ctx = None;
    let attr = attr.to_string();
    if !attr.is_empty() {
        let p = &attr[1..attr.len()-1];
        if !p.is_empty() {
            let p = p.replace('_', "Self");
            ctx = Some(syn::parse_path(&p).expect("Can not parse actor's context type"));
        }
    }

    // Parse the string representation into a syntax tree
    let mut ast = syn::parse_item(&source).unwrap();

    // Build the output
    let expanded = actor::build_handler(&mut ast, ctx);

    // Return the generated impl as a TokenStream
    let mut tokens = Tokens::new();
    ast.to_tokens(&mut tokens);
    let s = String::from(tokens.as_str()) + expanded.as_str();

    TokenStream::from_str(s.as_str()).unwrap()
}
