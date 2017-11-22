use quote;
use syn;

pub const MESSAGE_ATTR: &str = "Message";

pub fn expand(ast: &syn::DeriveInput) -> quote::Tokens {
let (item_type, error_type) = {
        match ::get_attribute_type_multiple(ast, MESSAGE_ATTR) {
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

    quote!{
        impl #impl_generics ResponseType for #name #ty_generics #where_clause {
            type Item = #item_type;
            type Error = #error_type;
        }
    }
}
