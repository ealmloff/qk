use crate::memo::Memo;
use crate::rsx::Elements;
use crate::state::State;
use proc_macro2::Ident;
use quote::ToTokens;
use std::collections::HashSet;
use syn::visit::{self, Visit};
use syn::{parse2, Expr, Type};
use syn::{ExprPath, Pat, PathArguments, PathSegment, TypeTuple};

#[derive(Default, Debug)]
pub struct ComponentVisitor {
    pub states: Vec<State>,
    pub memos: Vec<Memo>,
    pub rsx: Option<Elements>,
    in_reactive: bool,
}

impl Visit<'_> for ComponentVisitor {
    fn visit_macro(&mut self, mac: &syn::Macro) {
        if dbg!(mac.path.to_token_stream().to_string()) == "rsx" {
            self.rsx = parse2(mac.tokens.clone()).ok();
        }
    }

    fn visit_expr_call(&mut self, i: &syn::ExprCall) {
        if let Expr::Path(ExprPath { path, .. }) = &*i.func {
            if let Some(fn_name) = path.get_ident() {
                if fn_name == "rx" {
                    assert!(!self.in_reactive, "nested reactivity is not supported");
                    let mut visitor = SubscriptionVisitor {
                        states: &self.states,
                        subscribed: Default::default(),
                    };
                    visitor.visit_expr_call(i);

                    if let Some(Expr::Closure(closure)) = i.args.first().cloned() {
                        let closure_body = closure.body;
                        let capture = closure.capture;
                        let memo = Memo {
                            id: self.memos.len(),
                            ty: Type::Tuple(TypeTuple {
                                paren_token: Default::default(),
                                elems: Default::default(),
                            }),
                            closure: Some(*closure_body),
                            capture,
                            subscriptions: visitor.subscribed.into_iter().collect(),
                            subscribers: HashSet::new(),
                        };

                        self.in_reactive = true;
                        self.memos.push(memo);
                        visit::visit_expr_call(self, i);
                        self.in_reactive = false;
                        return;
                    }
                }
            }
        }

        visit::visit_expr_call(self, i);
    }

    fn visit_local(&mut self, i: &syn::Local) {
        if let Pat::Type(pat_ty) = &i.pat {
            if let Type::Path(path) = &*pat_ty.ty {
                let segments = &path.path.segments;
                if let Some(PathSegment {
                    ident,
                    arguments: PathArguments::AngleBracketed(ty),
                }) = segments.first()
                {
                    if segments.len() == 1 && ident == "Rx" {
                        if let Some(syn::GenericArgument::Type(ty)) = ty.args.first() {
                            if let Pat::Ident(name) = &*pat_ty.pat {
                                assert!(!self.in_reactive, "nested reactivity is not supported");
                                let state = State {
                                    id: self.states.len(),
                                    name: name.ident.clone(),
                                    ty: ty.clone(),
                                    expr: *i.init.as_ref().unwrap().1.clone(),
                                    subscribers: Default::default(),
                                };

                                self.in_reactive = true;
                                self.states.push(state);
                                visit::visit_local(self, i);
                                self.in_reactive = false;
                                return;
                            }
                        }
                    }
                }
            }
        }

        visit::visit_local(self, i);
    }
}

#[derive(Debug)]
struct SubscriptionVisitor<'a> {
    states: &'a Vec<State>,
    subscribed: Vec<usize>,
}

impl<'a, 'b> Visit<'a> for SubscriptionVisitor<'b> {
    fn visit_ident(&mut self, i: &'a Ident) {
        if let Some(name) = self.states.iter().find(|s| &s.name == i) {
            self.subscribed.push(name.id);
        }

        syn::visit::visit_ident(self, i);
    }
}
