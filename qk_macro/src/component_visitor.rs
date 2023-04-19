use crate::component::Component;
use crate::memo::Memo;
use crate::rsx::Elements;
use crate::state::State;
use proc_macro2::Ident;
use quote::ToTokens;
use syn::visit::{self, Visit};
use syn::ItemFn;
use syn::{parse2, Expr, Token, Type};
use syn::{ExprPath, Pat, PathArguments, PathSegment, TypeTuple};

#[derive(Debug)]
pub struct ComponentBuilder {
    pub states: Vec<State>,
    pub memos: Vec<Memo>,
    pub rsx: Option<Elements>,
    pub fn_item: ItemFn,
    pub type_name: Ident,
    pub in_reactive: bool,
}

impl ComponentBuilder {
    pub fn state(&mut self, name: Ident, ty: Type, expr: Expr) {
        self.states.push(State {
            id: self.states.len(),
            name,
            ty,
            expr,
            subscribers: Default::default(),
        })
    }

    pub fn memo(
        &mut self,
        ty: Option<Type>,
        closure: Expr,
        capture: Option<Token![move]>,
        raw_params: Vec<(Ident, Type)>,
    ) -> usize {
        let ty = ty.unwrap_or(Type::Tuple(TypeTuple {
            paren_token: Default::default(),
            elems: Default::default(),
        }));

        let mut visitor = SubscriptionVisitor {
            states: &self.states,
            subscribed: Default::default(),
        };
        visitor.visit_expr(&closure);

        let id = self.memos.len();
        self.memos.push(Memo {
            id,
            ty,
            closure: Some(closure),
            capture,
            subscriptions: visitor.subscribed.into_iter().collect(),
            subscribers: Default::default(),
            raw_params,
        });

        id
    }

    pub fn build(self) -> Component {
        let Self {
            mut states,
            mut memos,
            rsx,
            fn_item,
            type_name,
            ..
        } = self;
        let rsx = rsx.expect("rsx macro is required");

        // Resolve subscribers
        for i in 0..memos.len() {
            let memo = &memos[i];
            let mut subscribers = Vec::new();
            for other in memos.iter() {
                if other.subscriptions.contains(&memo.id) {
                    subscribers.push(memo.id);
                }
            }
            memos[i].subscribers = subscribers.into_iter().collect();
        }

        for state in &mut states {
            let mut subscribers = Vec::new();
            for other in memos.iter() {
                if other.subscriptions.contains(&state.id) {
                    subscribers.push(other.id);
                }
            }
            state.subscribers = subscribers.into_iter().collect();
        }

        Component {
            type_name,
            states,
            memos,
            rsx,
            fn_item,
        }
    }
}

impl Visit<'_> for ComponentBuilder {
    fn visit_macro(&mut self, mac: &syn::Macro) {
        if mac.path.to_token_stream().to_string() == "rsx" {
            if let Ok(mut rsx) = parse2::<Elements>(mac.tokens.clone()) {
                rsx.construct_memos(self);
                self.rsx = Some(rsx);
            }
        }
    }

    fn visit_expr_call(&mut self, i: &syn::ExprCall) {
        if let Expr::Path(ExprPath { path, .. }) = &*i.func {
            if let Some(fn_name) = path.get_ident() {
                if fn_name == "rx" {
                    assert!(!self.in_reactive, "nested reactivity is not supported");

                    if let Some(Expr::Closure(closure)) = i.args.first().cloned() {
                        self.memo(None, *closure.body, closure.capture, Default::default());

                        self.in_reactive = true;
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
                                self.state(
                                    name.ident.clone(),
                                    ty.clone(),
                                    *i.init.as_ref().unwrap().1.clone(),
                                );

                                self.in_reactive = true;
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
