use quote;
use quote::ToTokens;
use syn;
use rand::{Rng, thread_rng};

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

    let item_type = match item_type {
        Some(ty) => quote!{ type Item = #ty; },
        None => quote!{ type Item = (); },
    };

    let error_type = match error_type {
        Some(ty) => quote!{ type Error = #ty; },
        None => quote!{ type Error = (); },
    };

    let dummy_const = syn::Ident::from(format!("_IMPL_ACT_{}", name).to_lowercase());

    quote!{
        mod #dummy_const {
            extern crate actix;

            impl #impl_generics actix::ResponseType for #name #ty_generics #where_clause {
                #item_type
                #error_type
            }
        }
    }
}

fn get_attribute_type_multiple(ast: &syn::DeriveInput, name: &str) -> Option<Vec<Option<syn::TypePath>>>
{
    let attr = ast.attrs.iter().find(|a| {
        let a = a.interpret_meta();
        match a {
            Some(ref meta) if meta.name() == name => true,
            _ => false,
        }})?;
    let attr = attr.interpret_meta()?;

    if let syn::Meta::List(ref list) = attr {
        Some(list.nested.iter().map(|m| meta_item_to_ty(m, name)).collect())
    } else {
        panic!("The correct syntax is #[{}(type, type, ...)]", name);
    }
}

fn meta_item_to_ty(meta_item: &syn::NestedMeta, name: &str) -> Option<syn::TypePath> {
    if let syn::NestedMeta::Meta(syn::Meta::Word(ref i)) = *meta_item {
        syn::parse(i.into_tokens().into()).ok()
    } else {
        panic!("The correct syntax is #[{}(type)]", name);
    }
}

pub fn message_attr(ast: &mut syn::Item, types: Vec<syn::TypePath>) -> quote::Tokens {
    match ast {
        &mut syn::Item::Struct(ref ast) => {
            let (item, error) = match types.len() {
                0 => (None, None),
                1 => (Some(types[0].clone()), None),
                2 => (Some(types[0].clone()),
                      Some(types[1].clone())),
                _ => panic!("#[msg(type, type)] takes 2 parameters, given {}", types.len()),
            };

            let name = &ast.ident;
            let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

            let item_type = item.unwrap_or(parse_quote!{ () });
            let error_type = error.unwrap_or(parse_quote!{ () });
            let dummy_const = syn::Ident::from(format!("_impl_act_{}_{}", name, thread_rng().gen::<u32>()));

            return quote!{
                #[allow(non_upper_case_globals, unused_attributes,
                        unused_qualifications, unused_variables, unused_imports)]
                const #dummy_const: () = {
                    extern crate actix;

                    impl #impl_generics actix::ResponseType for #name #ty_generics #where_clause {
                        type Item = #item_type;
                        type Error = #error_type;
                    }
                };
            }
        },
        _ => (),
    }
    panic!("#[msg] can only be used with Struct")
}
