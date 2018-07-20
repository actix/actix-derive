use proc_macro2::{Ident, Span, TokenStream};
use syn;

pub const MESSAGE_ATTR: &str = "rtype";

pub fn expand(ast: &syn::DeriveInput) -> TokenStream {
    let item_type = {
        match get_attribute_type_multiple(ast, MESSAGE_ATTR) {
            Some(ty) => match ty.len() {
                1 => ty[0].clone(),
                _ => panic!(
                    "#[{}(type)] takes 1 parameters, given {}",
                    MESSAGE_ATTR,
                    ty.len()
                ),
            },
            None => None,
        }
    };

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let item_type = match item_type {
        Some(ty) => quote!{ type Result = #ty; },
        None => quote!{ type Result = (); },
    };

    let dummy_const = Ident::new(
        &format!("_IMPL_ACT_{}", name).to_lowercase(),
        Span::call_site(),
    );

    quote!{
        mod #dummy_const {
            extern crate actix;

            impl #impl_generics actix::Message for super::#name #ty_generics #where_clause {
                #item_type
            }
        }
    }
}

fn get_attribute_type_multiple(
    ast: &syn::DeriveInput, name: &str,
) -> Option<Vec<Option<syn::Type>>> {
    let attr = ast.attrs.iter().find(|a| {
        let a = a.interpret_meta();
        match a {
            Some(ref meta) if meta.name() == name => true,
            _ => false,
        }
    })?;
    let attr = attr.interpret_meta()?;

    if let syn::Meta::List(ref list) = attr {
        Some(
            list.nested
                .iter()
                .map(|m| meta_item_to_ty(m, name))
                .collect(),
        )
    } else {
        panic!("The correct syntax is #[{}(type, type, ...)]", name);
    }
}

fn meta_item_to_ty(meta_item: &syn::NestedMeta, name: &str) -> Option<syn::Type> {
    match *meta_item {
        syn::NestedMeta::Meta(syn::Meta::Word(ref i)) => {
            if let Ok(ty) = syn::parse_str::<syn::Type>(&i.to_string()) {
                Some(ty)
            } else {
                panic!("The correct syntax is #[{}(type)]", name);
            }
        }
        syn::NestedMeta::Meta(syn::Meta::NameValue(ref val)) => {
            if val.ident == "result" {
                if let syn::Lit::Str(ref s) = val.lit {
                    if let Ok(ty) = syn::parse_str::<syn::Type>(&s.value().to_string()) {
                        return Some(ty);
                    } else {
                        panic!("The correct syntax is #[{}(type)]", name);
                    }
                }
            }
            panic!("The correct syntax is #[{}(result=\"TYPE\")]", name);
        }
        syn::NestedMeta::Literal(syn::Lit::Str(ref s)) => {
            if let Ok(ty) = syn::parse_str::<syn::Type>(&s.value().to_string()) {
                return Some(ty);
            } else {
                panic!("The correct syntax is #[{}(type)]", name);
            }
        }
        _ => panic!("The correct syntax is #[{}(type)]", name),
    }
}
