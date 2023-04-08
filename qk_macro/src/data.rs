use proc_macro2::{Ident, TokenStream};
use quote::__private::ext::RepToTokensExt;
use quote::{quote, ToTokens};
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
    type_name: Ident,
    visitor: ComponentVisitor,
}

impl ToTokens for Component {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let init_states = self
            .visitor
            .states
            .iter()
            .map(|state| {
                let init = state.construct();
                quote! {
                    #init
                }
            })
            .chain(self.visitor.memos.iter().map(|memo| {
                let init = memo.construct(&self.visitor.states);
                quote! {
                    #init
                }
            }));
        let update_states = self
            .visitor
            .states
            .iter()
            .map(|state| {
                let update = state.update();
                quote! {
                    #update
                }
            })
            .chain(self.visitor.memos.iter().map(|memo| {
                let update = memo.update(&self.visitor.states);
                quote! {
                    #update
                }
            }));

        let comp_name = &self.type_name;
        let types = self
            .visitor
            .states
            .iter()
            .map(|state| state.type_def())
            .chain(
                self.visitor
                    .memos
                    .iter()
                    .map(|memo| memo.type_def(&self.visitor.states)),
            );

        let create_comp = self
            .visitor
            .states
            .iter()
            .map(|state| {
                let name = &state.name;
                let private = Ident::new(&format!("__{name}"), name.span());

                quote! {
                    #name: #private
                }
            })
            .chain(self.visitor.memos.iter().map(|memo| {
                let name = memo.ident();

                quote! {
                    #name
                }
            }));

        tokens.extend(quote! {
            struct #comp_name {
                #(#types,)*
                tracking: DirtyTrackSet<u8, u8>,
            }
            impl #comp_name {
                #(#update_states)*
            }
            let tracking: DirtyTrackSet<u8, u8> = DirtyTrackSet::default();
            #(#init_states)*
            let mut comp = #comp_name {
                #(#create_comp,)*
                tracking,
            };
        })
    }
}

impl Parse for Component {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut f = input.parse::<syn::ItemFn>()?;
        let type_name = f.sig.ident.clone();

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

        Ok(Self { type_name, visitor })
    }
}

#[test]
fn parses() {
    use syn::parse_str;
    let input = r#"fn Foo(cx: Scope) {
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
        }"#;
    let comp = parse_str::<Component>(input).unwrap();

    println!("{comp:#?}");

    println!(
        "{}",
        quote! {
            #comp
        }
    );

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
    fn type_def(&self) -> TokenStream {
        let name = &self.name;
        let ty = &self.ty;
        quote! {
            #name: #ty
        }
    }

    fn tracked(&self) -> TokenStream {
        let name = &self.name;
        let id = self.id as u8;

        quote! {
            RwTrack {
                data: &mut self.#name,
                tracking: self.tracking.track(#id),
            }
        }
    }

    fn construct(&self) -> TokenStream {
        let name = &self.name;
        let ty = &self.ty;
        let expr = &self.expr;

        let private_name = Ident::new(&format!("__{name}"), name.span());

        quote! {
            let mut #private_name: #ty = #expr;
        }
    }

    fn construct_tracked(&self) -> TokenStream {
        let name = &self.name;
        let private_name = Ident::new(&format!("__{name}"), name.span());
        let id = self.id as u8;
        quote! {
            let mut #name = RwTrack {
                data: &mut #private_name,
                tracking: tracking.track(#id),
            };
        }
    }

    fn tracked_type(&self) -> TokenStream {
        let ty = &self.ty;
        quote! {
            RwTrack<#ty, u8, u8>,
        }
    }

    fn update_fn(&self) -> Ident {
        let name = &self.name;

        Ident::new(&format!("update_{name}"), name.span())
    }

    fn update(&self) -> TokenStream {
        let name = &self.name;
        let id = self.id;
        let id_bits = (1u32 << id) as u8;
        let update_fn_name = self.update_fn();
        let maybe_subscribes = self.subscribers.iter().map(|id| {
            let ident = Ident::new(&format!("memo_{id}",), name.span());
            let ident_update = Ident::new(&format!("update_{ident}",), name.span());
            quote! {
                if self.#ident.rx_subscriptions & #id_bits != 0{
                    self.#ident_update();
                }
            }
        });

        let with_fn_name = Ident::new(&format!("with_{name}"), proc_macro2::Span::call_site());
        let ty = &self.tracked_type();

        quote! {
            fn #update_fn_name(&mut self) {
                if self.tracking.get_write() & #id_bits != 0 {
                    #(#maybe_subscribes)*
                }
            }

            fn #with_fn_name(&mut self, f: impl FnOnce(#ty)) {
                self.tracking.reset_write();
                f(RwTrack {
                    data: &mut self.#name,
                    tracking: self.tracking.track(#id as u8),
                });
                self.#update_fn_name();
            }
        }
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
    fn type_def(&self, states: &[State]) -> TokenStream {
        let ident = self.ident();
        let ty = &self.ty;
        let types = self.types(states);
        quote! {
            #ident: Effect<dyn Fn(#types), #ty>
        }
    }

    fn ident(&self) -> Ident {
        Ident::new(&format!("memo_{}", self.id), proc_macro2::Span::call_site())
    }

    fn types(&self, states: &[State]) -> TokenStream {
        let mut parameters = Vec::new();
        for id in &self.subscriptions {
            let ty = &states[*id].tracked_type();
            parameters.push(quote! {
                #ty
            });
        }
        quote! {
            #(#parameters)*
        }
    }

    fn parameters(&self, states: &[State]) -> TokenStream {
        let mut parameters = Vec::new();
        for id in &self.subscriptions {
            let state = &states[*id];
            let name = &state.name;
            let ty = &state.ty;
            parameters.push(quote! {
                #name: RwTrack<#ty, u8, u8>,
            });
        }
        quote! {
            #(#parameters)*
        }
    }

    fn construct(&self, states: &[State]) -> TokenStream {
        let ident_name = self.ident();
        let closure = &self.closure;
        let private_name = Ident::new(&format!("__{ident_name}"), ident_name.span());
        let parameters = self.parameters(states);
        let types = self.types(states);

        let subscribers = self.subscriptions.iter().map(|id| states[*id].name.clone());
        let ty = &self.ty;

        let rw_tracks = self
            .subscriptions
            .iter()
            .map(|id| states[*id].construct_tracked());

        quote! {
            #( #rw_tracks )*
            tracking.reset_read();
            let #private_name = Box::new(|#parameters| {
                #closure
            }) as Box<dyn Fn(#types) -> #ty>;

            let #ident_name = #private_name(
                #(
                    #subscribers,
                )*
            );

            let #ident_name = Effect {
                rx: #private_name,
                rx_subscriptions: tracking.read.get(),
                current: #ident_name,
            };
        }
    }

    fn call(&self, states: &[State]) -> TokenStream {
        let ident_name = self.ident();
        let private_name = Ident::new(&format!("__{ident_name}"), ident_name.span());

        let subscribers = self.subscriptions.iter().map(|id| states[*id].name.clone());

        quote! {
            #private_name(
                #(
                    #subscribers,
                )*
            );
        }
    }

    fn update(&self, states: &[State]) -> TokenStream {
        let ident_name = self.ident();

        let update_fn_name = Ident::new(
            &format!("update_{ident_name}"),
            proc_macro2::Span::call_site(),
        );

        let subscriptions_setup = self.subscriptions.iter().map(|id| {
            let tracked = states[*id].tracked();
            let name = &states[*id].name;
            quote! {
                let #name = #tracked;
            }
        });
        let subscriptions: Vec<_> = self
            .subscriptions
            .iter()
            .map(|id| states[*id].name.clone())
            .collect();

        let subscriptions_update = self.subscriptions.iter().map(|id| states[*id].update_fn());

        quote! {
            fn #update_fn_name(&mut self) {
                self.tracking.reset_write();
                let old = self.#ident_name.current.clone();
                #(
                    #subscriptions_setup
                )*
                self.#ident_name.current = (self.#ident_name.rx)(
                    #(
                        #subscriptions,
                    )*
                );
                if old != self.#ident_name.current {
                    todo!("handle memo returns");
                }
                #( self.#subscriptions_update(); )*
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

                    if let Some(Expr::Closure(closure)) = i.args.first().cloned() {
                        let closure_body = closure.body;
                        let memo = Memo {
                            id: self.memos.len(),
                            ty: Type::Tuple(TypeTuple {
                                paren_token: Default::default(),
                                elems: Default::default(),
                            }),
                            closure: *closure_body,
                            subscriptions: visitor.subscribed.into_iter().collect(),
                            subscribers: HashSet::new(),
                        };

                        self.memos.push(memo);
                    }
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
                                    id: self.states.len(),
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
