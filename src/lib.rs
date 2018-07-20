extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::DeriveInput;

mod message;

#[proc_macro_derive(Message, attributes(rtype))]
pub fn message_derive_rtype(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    message::expand(&ast).into()
}
