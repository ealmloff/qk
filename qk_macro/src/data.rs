use proc_macro2::{Ident, TokenStream};
use quote::__private::ext::RepToTokensExt;
use quote::quote;
use std::collections::HashSet;
use syn::ext::IdentExt;
use syn::visit::Visit;
use syn::visit_mut::{self, VisitMut};
use syn::{braced, parse::Parse, Expr, Type};
use syn::{
    Block, ExprBlock, ExprLet, ExprPath, Pat, PatTuple, PathArguments, PathSegment, TypeTuple,
};

#[derive(Debug)]
struct Component {
    visitor: ComponentVisitor,
}

impl Parse for Component {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut f = input.parse::<syn::ItemFn>()?;

        let mut visitor = ComponentVisitor::default();

        visitor.visit_item_fn_mut(&mut f);

        // Resolve subscribers
        let memos = &mut visitor.memos;
        let states = &mut visitor.states;
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

        for state in states {
            let mut subscribers = Vec::new();
            for other in memos.iter() {
                if other.subscriptions.contains(&state.id) {
                    subscribers.push(other.id);
                }
            }
            state.subscribers = subscribers.into_iter().collect();
        }

        Ok(Self { visitor })
    }
}

#[test]
fn parses() {
    use syn::parse_str;
    let input = r#"
        fn Foo(cx: Scope) {
            let x: Rx<i32> = 0;
            let y: Rx<i32> = 0;
            rx(|| {
                let x = *x;
                println!("{x}");
            });
            rx(|| {
                let x = *x;
                let y = *y;
                println!("{x}");
            });
            *x = *x + 1;
        }
        "#;
    // fn Foo(cx: Scope) {
    //     let x: Rx<i32> = 0;
    //     let y: Rx<i32> = 0;
    //     // what is rx here?
    //     rx(|| {
    //         let x = get_x();
    //         println!("{x}");
    //     })
    //     rx(|| {
    //         let x = x();
    //         println!("{x}");
    //     })
    //     set_x(get_x() + 1);
    // }
    // ->
    // fn Foo(cx: Scope) {
    //     let x: &mut i32 = &mut 0;
    //     let y: &mut i32 = &mut 0;
    //     // what is rx here?
    //     let x = get_x();
    //     println!("{x}");
    //     let x = x();
    //     println!("{x}");
    //
    //     Foo {
    //         x,
    //         y,
    //     }
    // }
    let comp = parse_str::<Component>(input).unwrap();

    println!("{comp:#?}");
    panic!()
}

/// State that belongs to a component.
struct State {
    id: usize,
    name: Ident,
    ty: Type,
    expr: Expr,
    subscribers: HashSet<usize>,
}

impl State {
    fn construct(&self) -> TokenStream {
        let name = &self.name;
        let ty = &self.ty;
        let expr = &self.expr;
        quote!(let #name: #ty = #expr;)
    }

    fn update(&self) -> TokenStream {
        let name = &self.name;
        let expr = &self.expr;
        quote!(self.#name = #expr;)
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("id", &self.id)
            .field("name", &self.name.to_string())
            .field("ty", &{
                let ty = &self.ty;
                quote!(#ty).to_string()
            })
            .field("expr", &{
                let expr = &self.expr;
                quote!(#expr).to_string()
            })
            .field("subscribers", &self.subscribers)
            .finish()
    }
}

/// A memo that will automatically update when its dependencies change.
struct Memo {
    id: usize,
    ty: Type,
    closure: Expr,
    subscriptions: HashSet<usize>,
    subscribers: HashSet<usize>,
}

impl std::fmt::Debug for Memo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Memo")
            .field("id", &self.id)
            .field("ty", &{
                let ty = &self.ty;
                quote!(#ty).to_string()
            })
            .field("block", &{
                let block = &self.closure;
                quote!(#block).to_string()
            })
            .field("subscriptions", &self.subscriptions)
            .field("subscribers", &self.subscribers)
            .finish()
    }
}

impl Memo {
    fn ident(&self) -> Ident {
        Ident::new(&format!("memo_{}", self.id), proc_macro2::Span::call_site())
    }

    fn update(&self) -> TokenStream {
        let ident_name = self.ident();

        let update_fn_name = Ident::new(
            &format!("update_{ident_name}"),
            proc_macro2::Span::call_site(),
        );

        let block = &self.closure;

        quote! {
            fn #update_fn_name(&mut self) {
                self.#ident_name = #block;
            }
        }
    }
}

#[derive(Default, Debug)]
struct ComponentVisitor {
    states: Vec<State>,
    memos: Vec<Memo>,
}

impl VisitMut for ComponentVisitor {
    fn visit_expr_call_mut(&mut self, i: &mut syn::ExprCall) {
        if let Expr::Path(ExprPath { path, .. }) = &*i.func {
            if let Some(fn_name) = path.get_ident() {
                if fn_name == "rx" {
                    let mut visitor = SubscriptionVisitor {
                        states: &self.states,
                        subscribed: Default::default(),
                    };
                    visitor.visit_expr_call(i);

                    let memo = Memo {
                        id: self.memos.len() + self.states.len(),
                        ty: Type::Tuple(TypeTuple {
                            paren_token: Default::default(),
                            elems: Default::default(),
                        }),
                        closure: i.args.first().unwrap().clone(),
                        subscriptions: visitor.subscribed.into_iter().collect(),
                        subscribers: HashSet::new(),
                    };

                    self.memos.push(memo);
                }
            }
        }

        visit_mut::visit_expr_call_mut(self, i);
    }

    fn visit_local_mut(&mut self, i: &mut syn::Local) {
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
                                let state = State {
                                    id: self.states.len() + self.memos.len(),
                                    name: name.ident.clone(),
                                    ty: ty.clone(),
                                    expr: *i.init.as_ref().unwrap().1.clone(),
                                    subscribers: Default::default(),
                                };

                                self.states.push(state);
                            }
                        }
                    }
                }
            }
        }

        syn::visit_mut::visit_local_mut(self, i);
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
