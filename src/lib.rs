extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;

use proc_macro::TokenStream;
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
