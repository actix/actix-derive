// Copyright (c) 2017-present PyActix Project and Contributors

use syn;
use syn::{Attribute, FnArg, Ident, Item, ImplItem, Meta, MethodSig, NestedMeta, Pat, Path, Type, TypePath};
use quote::{Tokens, ToTokens};
use rand::{Rng, thread_rng};


pub fn build_handler(ast: &mut Item, ctx: Option<Path>) -> Tokens {
    if let &mut Item::Impl(ref mut item_impl) = ast {
        if item_impl.trait_.is_none() {
            return impl_handler(&item_impl.self_ty, &mut item_impl.items, ctx)
        }
    }

    panic!("#[handler] can only be used with Impl blocks")
}

fn impl_handler(ty: &Box<Type>, impls: &mut Vec<ImplItem>, ctx: Option<Path>) -> Tokens {
    // get method names in impl block
    let mut handlers = Vec::new();
    for iimpl in impls.iter_mut() {
        if let &mut ImplItem::Method(ref mut method) = iimpl {
            if let Some(handle) = gen_handler(ty, &mut method.sig, &mut method.attrs) {
                handlers.push(handle);
            }
        }
    }

    let n = match ty.as_ref() {
        &Type::Path(ref type_path) => type_path.path.segments.last().unwrap().value().ident.as_ref(),
        _ => "handlers"
    };
    let dummy_const = Ident::from(format!("_impl_handlers_{}_{}", thread_rng().gen::<u32>(), n));

    if let Some(ctx) = ctx {
        quote! {
            #[allow(non_upper_case_globals, unused_attributes,
                    unused_qualifications, unused_variables, unused_imports)]
            const #dummy_const: () = {
                extern crate actix;
                use actix::{Actor, Context, FramedContext};

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
                extern crate actix;
                use actix::{Actor, Context, FramedContext};

                #(#handlers)*
            };
        }
    }
}

fn gen_handler(cls: &Box<Type>, sig: &mut MethodSig, attrs: &mut Vec<Attribute>) -> Option<Tokens> {
    let name = sig.ident;
    if let Some(msg) = parse_attributes(attrs) {
        let mut args = Vec::new();
        for input in sig.decl.inputs.iter() {
            match input {
                &FnArg::Captured(ref arg) => {
                    match arg.pat {
                        Pat::Ident(ref ident) => {
                            if ident.ident.as_ref() == "ctx" {
                                args.push(quote!{ctx,});
                                continue;
                            }

                            if let Type::Path(ref path) = arg.ty {
                                let msg_ty = match msg {
                                    HandlerType::Simple(ref ty) => ty,
                                    HandlerType::Handler(ref ty) => ty,
                                    HandlerType::Stream(ref ty, _) => ty,
                                };
                                let msg_ty: TypePath = syn::parse(msg_ty.into_tokens().into()).unwrap();

                                if *path == msg_ty {
                                    args.push(quote!{msg});
                                    continue;
                                }
                            }

                            args.push(quote!{msg.#name});
                        },
                        _ => panic!("unsupported argument: {:?}", arg.pat),
                    }
                }
                &FnArg::SelfRef(_) | &FnArg::SelfValue(_) => (),
                _ => panic!("unsupported argument: {:?}", name),
            }
        }

        match msg {
            HandlerType::Simple(msg) => Some(quote!{
                impl actix::Handler<#msg> for #cls {
                    fn handle(&mut self, msg: #msg, ctx: &mut Self::Context)
                              -> actix::Response<Self, #msg> {
                        Self::reply(self.#name(#(#args),*))
                    }
                }
            }),
            HandlerType::Handler(msg) => Some(quote!{
                impl actix::Handler<#msg> for #cls {
                    fn handle(&mut self, msg: #msg, ctx: &mut Self::Context)
                              -> actix::Response<Self, #msg> {
                        match self.#name(#(#args),*) {
                            Ok(item) => Self::reply(item),
                            Err(err) => Self::reply_error(err)
                        }
                    }
                }
            }),
            HandlerType::Stream(msg, err) => Some(quote!{
                impl actix::StreamHandler<#msg, #err> for #cls {}
                impl actix::Handler<#msg, #err> for #cls {
                    fn handle(&mut self, msg: #msg, ctx: &mut Self::Context)
                              -> actix::Response<Self, #msg> {
                        match self.#name(#(#args),*) {
                            Ok(item) => Self::reply(item),
                            Err(err) => Self::reply_error(err)
                        }
                    }
                }
            }),
        }
    } else {
        None
    }
}

enum HandlerType {
    Simple(Ident),
    Handler(Ident),
    Stream(Ident, Ident),
}

fn parse_attributes(attrs: &mut Vec<Attribute>) -> Option<HandlerType> {
    let mut result = None;

    for attr in attrs.clone().iter() {
        let meta = attr.interpret_meta()?;
        match meta {
            Meta::List(ref metalist) => {
                let parse_lit = |lit: &syn::Lit| -> Ident {
                    syn::parse(lit.into_tokens().into()).unwrap()
                };

                match metalist.ident.as_ref() {
                    "stream" => {
                        if metalist.nested.len() > 2 {
                            panic!("#[stream(..)] accepts only two argument");
                        }
                        let ident = |nested: &NestedMeta| {
                            match nested {
                                &NestedMeta::Meta(Meta::Word(ref ident)) => ident.clone(),
                                &NestedMeta::Literal(ref lit) => parse_lit(lit),
                                ref val => panic!("{:?} is not supported", val),
                            }
                        };

                        let item = ident(&metalist.nested[0]);
                        let error = ident(&metalist.nested[1]);
                        result = Some(HandlerType::Stream(item, error));
                        attrs.remove_item(attr);
                    },
                    "handler" => {
                        if metalist.nested.len() > 1 {
                            panic!("#[handler(..)] accepts only one argument");
                        }

                        match &metalist.nested[0] {
                            &NestedMeta::Meta(Meta::Word(ref ident)) => result = Some(HandlerType::Handler(ident.clone())),
                            &NestedMeta::Literal(ref lit) => result = Some(HandlerType::Handler(parse_lit(lit))),
                            ref val => panic!("{:?} is not supported", val),
                        }
                        attrs.remove_item(attr);
                    },
                    "simple" => {
                        if metalist.nested.len() > 1 {
                            panic!("#[simple(..)] accepts only one argument");
                        }

                        match &metalist.nested[0] {
                            &NestedMeta::Meta(Meta::Word(ref ident)) => result = Some(HandlerType::Simple(ident.clone())),
                            &NestedMeta::Literal(ref lit) => result = Some(HandlerType::Handler(parse_lit(lit))),
                            ref val => panic!("{:?} is not supported", val),
                        };
                        attrs.remove_item(attr);
                    },
                    _ => (),
                }
            },
            _ => (),
        }
    }

    result
}
