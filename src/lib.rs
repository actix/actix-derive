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

#[cfg(actix_nightly)]
#[proc_macro_attribute]
pub fn msg(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let source = input.to_string();

    let mut types = Vec::new();
    let attr = attr.to_string();
    if !attr.is_empty() {
        let p = &attr[1..attr.len()-1];
        if !p.is_empty() {
            for p in p.split(',') {
                types.push(
                    syn::parse_path(&p).expect("Can not parse actor's context type"));
            }
        }
    }

    // Parse the string representation into a syntax tree
    let mut ast = syn::parse_item(&source).unwrap();

    // Build the output
    let expanded = message::message_attr(&mut ast, types);

    // Return the generated impl as a TokenStream
    let mut tokens = Tokens::new();
    ast.to_tokens(&mut tokens);
    let s = String::from(tokens.as_str()) + expanded.as_str();

    TokenStream::from_str(s.as_str()).unwrap()
}
