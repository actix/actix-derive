use quote;
use syn;
use rand::{Rng, thread_rng};

pub const MESSAGE_ATTR: &str = "rtype";

pub fn expand(ast: &syn::DeriveInput) -> quote::Tokens {
    let item_type = {
        match get_attribute_type_multiple(ast, MESSAGE_ATTR) {
            Some(ty) => {
                match ty.len() {
                    1 => ty[0].clone(),
                    _ => panic!("#[{}(type)] takes 1 parameters, given {}", MESSAGE_ATTR, ty.len()),
                }
            },
            None => None,
        }
    };

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let item_type = item_type.unwrap_or(syn::Ty::Tup(vec![]));
    let dummy_const = syn::Ident::new(format!("_IMPL_ACT_{}", name).to_lowercase());

    quote!{
        mod #dummy_const {
            extern crate actix;

            impl #impl_generics actix::Message for super::#name #ty_generics #where_clause {
                type Result = #item_type;
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
    match *meta_item {
        syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref i)) => {
            let ty = syn::parse::ty(i.as_ref());
            match ty {
                syn::parse::IResult::Done(_, ty) => Some(ty),
                _ => None,
            }
        },
        syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ref ident, ref i)) =>
        {
            if ident == "result" {
                if let &syn::Lit::Str(ref s, _) = i {
                    let ty = syn::parse_type(s).unwrap();
                    return Some(ty);
                }
            }
            panic!("The correct syntax is #[{}(result=\"TYPE\")]", name);
        },
        _ => panic!("The correct syntax is #[{}(type)]", name),
    }
}

pub fn message_attr(ast: &mut syn::Item, types: Vec<syn::Path>) -> quote::Tokens {
    match ast.node {
        syn::ItemKind::Struct(_, ref gen) => {
            let item = match types.len() {
                0 => None,
                1 => Some(syn::Ty::Path(None, types[0].clone())),
                _ => panic!("#[msg(type)] takes 1 parameters, given {}", types.len()),
            };

            let name = &ast.ident;
            let (impl_generics, ty_generics, where_clause) = gen.split_for_impl();

            let item_type = item.unwrap_or(syn::Ty::Tup(vec![]));
            let dummy_const = syn::Ident::new(
                format!("_impl_act_{}_{}", name, thread_rng().gen::<u32>()));

            return quote!{
                #[allow(non_upper_case_globals, unused_attributes,
                        unused_qualifications, unused_variables, unused_imports)]
                const #dummy_const: () = {
                    extern crate actix;

                    impl #impl_generics actix::Message for #name #ty_generics #where_clause {
                        type Result = #item_type;
                    }
                };
            }
        },
        _ => (),
    }
    panic!("#[msg] can only be used with Struct")
}
