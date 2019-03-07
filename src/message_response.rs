use proc_macro2::{Ident, Span, TokenStream};
use syn;

pub fn expand(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let dummy_const = Ident::new(
        &format!("_IMPL_ACT_MSG_RSP_{}", name).to_lowercase(),
        Span::call_site(),
    );

    let (_, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut generics = ast.generics.clone();
    generics.params.push(parse_quote!(_A: actix::Actor));
    generics
        .params
        .push(parse_quote!(_M: actix::Message<Result = #name #ty_generics>));
    let (impl_generics, _, _) = generics.split_for_impl();

    quote! {
        mod #dummy_const {
            use super::*;
            extern crate actix;

            impl #impl_generics actix::dev::MessageResponse<_A, _M> for #name #ty_generics #where_clause {
                fn handle<R: actix::dev::ResponseChannel<_M>>(self, _: &mut _A::Context, tx: Option<R>) {
                    if let Some(tx) = tx {
                        tx.send(self);
                    }
                }
            }
        }
    }
}
