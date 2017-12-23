// Copyright (c) 2017-present PyActix Project and Contributors

use syn;
use quote::Tokens;
use rand::{Rng, thread_rng};


pub fn build_handler(ast: &mut syn::Item, ctx: Option<syn::Path>) -> Tokens {
    match ast.node {
        syn::ItemKind::Impl(_, _, _, ref path, ref ty, ref mut impl_items) => {
            if path.is_none() {
                return impl_handler(ty, impl_items, ctx)
            }
        },
        _ => (),
    }
    panic!("#[handler] can only be used with Impl blocks")
}

fn impl_handler(ty: &Box<syn::Ty>, impls: &mut Vec<syn::ImplItem>, ctx: Option<syn::Path>)
                -> Tokens
{
    // get method names in impl block
    let mut handlers = Vec::new();
    for iimpl in impls.iter_mut() {
        match iimpl.node {
            syn::ImplItemKind::Method(ref mut sig, _) => {
                if let Some(handle) = gen_handler(ty, &iimpl.ident, sig, &mut iimpl.attrs) {
                    handlers.push(handle)
                }
            },
            _ => (),
        }
    }

    let n = match ty.as_ref() {
        &syn::Ty::Path(_, ref p) => {
            p.segments.last().as_ref().unwrap().ident.as_ref()
        }
        _ => "handlers"
    };

    let dummy_const = syn::Ident::new(
        format!("_impl_handlers_{}_{}", thread_rng().gen::<u32>(), n));

    if let Some(ctx) = ctx {
        quote! {
            #[allow(non_upper_case_globals, unused_attributes,
                    unused_qualifications, unused_variables, unused_imports)]
            const #dummy_const: () = {
                extern crate actix as _actix;
                use actix::{Context, FramedContext};

                impl Actor for #ty {
                    type Context = #ctx;
                }

                #(#handlers)*
            };
        }
    } else {
        quote! {
            #[allow(non_upper_case_globals, unused_attributes,
                    unused_qualifications, unused_variables, unused_imports)]
            const #dummy_const: () = {
                extern crate actix as _actix;

                #(#handlers)*
            };
        }
    }
}

fn gen_handler(cls: &Box<syn::Ty>, name: &syn::Ident,
               sig: &mut syn::MethodSig, attrs: &mut Vec<syn::Attribute>) -> Option<Tokens>
{
    if let Some(msg) = parse_attributes(attrs) {
        // list arguments
        let mut args = Vec::new();
        for input in sig.decl.inputs.iter() {
            match input {
                &syn::FnArg::Captured(ref pat, _) => {
                    match pat {
                        &syn::Pat::Ident(_, ref name, _) =>
                            if name.as_ref() == "ctx" {
                                args.push(quote!{ctx,});
                            } else {
                                args.push(quote!{msg.#name});
                            }
                        _ =>
                            panic!("unsupported argument: {:?}", pat),
                    }
                }
                &syn::FnArg::SelfRef(_, _) | &syn::FnArg::SelfValue(_) => (),
                &syn::FnArg::Ignored(_) => panic!("ignored argument: {:?}", name),
            }
        }

        match msg {
            HandlerType::Simple(msg) => Some(quote!{
                impl _actix::Handler<#msg> for #cls {
                    fn handle(&mut self, msg: #msg, ctx: &mut Self::Context)
                              -> _actix::Response<Self, #msg> {
                        Ok(self.#name(#(#args),*)).into()
                    }
                }
            }),
            HandlerType::Handler(msg) => Some(quote!{
                impl _actix::Handler<#msg> for #cls {
                    fn handle(&mut self, msg: #msg, ctx: &mut Self::Context)
                              -> _actix::Response<Self, #msg> {
                        self.#name(#(#args),*).into()
                    }
                }
            }),
        }
    } else {
        None
    }
}

enum HandlerType {
    Simple(syn::Ident),
    Handler(syn::Ident),
}

fn parse_attributes(attrs: &mut Vec<syn::Attribute>) -> Option<HandlerType> {
    let mut result = None;
    let mut new_attrs = Vec::new();

    for attr in attrs.iter() {
        match attr.value {
            syn::MetaItem::List(ref name, ref meta) => {
                match name.as_ref() {
                    "handler" => {
                        if meta.len() > 1 {
                            panic!("#[handler(..)] accepts only one argument");
                        }
                        match &meta[0] {
                            &syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref ident)) => {
                                result = Some(HandlerType::Handler(ident.clone()));
                                break
                            },
                            &syn::NestedMetaItem::Literal(ref lit) => {
                                let s = quote!{ #lit }.to_string();
                                result = Some(
                                    HandlerType::Handler(syn::Ident::from(&s[1..s.len()-1])));
                                break
                            },
                            ref val => panic!("{:?} is not supported", val),
                        }
                    },
                    "simple" => {
                        if meta.len() > 1 {
                            panic!("#[simple(..)] accepts only one argument");
                        }
                        match &meta[0] {
                            &syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref ident)) => {
                                result = Some(HandlerType::Simple(ident.clone()));
                                break
                            },
                            &syn::NestedMetaItem::Literal(ref lit) => {
                                let s = quote!{ #lit }.to_string();
                                result = Some(
                                    HandlerType::Simple(syn::Ident::from(&s[1..s.len()-1])));
                                break
                            },
                            ref val => panic!("{:?} is not supported", val),
                        }
                    },
                    _ => new_attrs.push(attr.clone()),
                }
            },
            _ => new_attrs.push(attr.clone()),
        }
    }
    attrs.clear();
    attrs.extend(new_attrs);

    result
}
