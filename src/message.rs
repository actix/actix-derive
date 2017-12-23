use quote;
use syn;

pub const MESSAGE_ATTR: &str = "rtype";

pub fn expand(ast: &syn::DeriveInput) -> quote::Tokens {
    let (item_type, error_type) = {
        match get_attribute_type_multiple(ast, MESSAGE_ATTR) {
            Some(ty) => {
                match ty.len() {
                    1 => (ty[0].clone(), None),
                    2 => (ty[0].clone(), ty[1].clone()),
                    _ => panic!("#[{}(type, type)] takes 2 parameters, given {}", MESSAGE_ATTR, ty.len()),
                }
            },
            None => {
                (None, None)
            }
        }

    };

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let item_type = item_type.unwrap_or(syn::Ty::Tup(vec![]));
    let error_type = error_type.unwrap_or(syn::Ty::Tup(vec![]));
    let dummy_const = syn::Ident::new(format!("_IMPL_ACT_{}", name).to_lowercase());

    quote!{
        mod #dummy_const {
            extern crate actix;

            impl #impl_generics actix::ResponseType for #name #ty_generics #where_clause {
                type Item = #item_type;
                type Error = #error_type;
            }
        }
    }
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

pub fn message_attr(ast: &mut syn::Item, types: Vec<syn::Path>) -> quote::Tokens {
    match ast.node {
        syn::ItemKind::Struct(_, ref gen) => {
            let (item, error) = match types.len() {
                0 => (None, None),
                1 => (Some(syn::Ty::Path(None, types[0].clone())), None),
                2 => (Some(syn::Ty::Path(None, types[0].clone())),
                      Some(syn::Ty::Path(None, types[1].clone()))),
                _ => panic!("#[msg(type, type)] takes 2 parameters, given {}", types.len()),
            };

            let name = &ast.ident;
            let (impl_generics, ty_generics, where_clause) = gen.split_for_impl();

            let item_type = item.unwrap_or(syn::Ty::Tup(vec![]));
            let error_type = error.unwrap_or(syn::Ty::Tup(vec![]));
            let dummy_const = syn::Ident::new(format!("_IMPL_ACT_{}", name).to_lowercase());

            return quote!{
                mod #dummy_const {
                    extern crate actix;

                    impl #impl_generics actix::ResponseType for #name #ty_generics #where_clause {
                        type Item = #item_type;
                        type Error = #error_type;
                    }
                }
            }
        },
        _ => (),
    }
    panic!("#[msg] can only be used with Struct")
}
