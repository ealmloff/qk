use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::collections::HashSet;
use syn::token::Move;
use syn::{parse_quote, Expr, Stmt, Type};

use crate::component::Component;

/// A memo that will automatically update when its dependencies change.
#[derive(Clone)]
pub struct Memo {
    pub id: usize,
    pub ty: Type,
    pub closure: Option<Expr>,
    pub capture: Option<Move>,
    pub subscriptions: HashSet<usize>,
    pub subscribers: HashSet<usize>,
    pub raw_params: Vec<(Ident, Type)>,
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
    pub fn ty(&self, component: &Component) -> TokenStream {
        let ty = &self.ty;
        let types = self.types(component);
        quote! {
            Effect<Box<dyn Fn(#types)>, #ty>
        }
    }

    pub fn type_def(&self, component: &Component) -> TokenStream {
        let ident = self.ident();
        let ty = &self.ty(component);
        quote! {
            #ident: #ty
        }
    }

    pub fn ident(&self) -> Ident {
        Ident::new(&format!("memo_{}", self.id), proc_macro2::Span::call_site())
    }

    fn types(&self, component: &Component) -> TokenStream {
        let mut parameters = Vec::new();
        for id in &self.subscriptions {
            let ty = &component.states[*id].tracked_type();
            parameters.push(quote! {
                #ty
            });
        }
        for (_, ty) in &self.raw_params {
            parameters.push(quote! {
                #ty
            });
        }
        quote! {
            #(#parameters)*
        }
    }

    pub fn parameters(&self, component: &Component) -> TokenStream {
        let states = &component.states;
        let mut parameters = Vec::new();
        for id in &self.subscriptions {
            let state = &states[*id];
            let name = &state.name;
            let ty = &state.ty;
            parameters.push(quote! {
                mut #name: RwTrack<#ty, u8, u8>,
            });
        }
        for (r, ty) in &self.raw_params {
            parameters.push(quote! {
                mut #r: #ty,
            });
        }
        quote! {
            #(#parameters)*
        }
    }

    pub fn construct(&self, component: &Component) -> Stmt {
        let states = &component.states;
        let ident_name = self.ident();
        let closure = &self.closure;
        let private_name = Ident::new(&format!("__{ident_name}"), ident_name.span());
        let parameters = self.parameters(component);
        let types = self.types(component);

        let subscribers = self
            .subscriptions
            .iter()
            .map(|id| states[*id].name.clone())
            .chain(self.raw_params.iter().map(|(r, _)| r).cloned());
        let ty = &self.ty;

        let rw_tracks = self
            .subscriptions
            .iter()
            .map(|id| states[*id].construct_tracked());

        let movability = &self.capture;

        parse_quote! {
            #private_name = {
                tracking.reset_read();
                #( #rw_tracks )*
                let #private_name = Box::new(#movability |#parameters| {
                    #closure
                }) as Box<dyn Fn(#types) -> #ty>;
                let current = #private_name(
                    #(
                        #subscribers,
                    )*
                );
                Effect {
                    rx: #private_name,
                    rx_subscriptions: tracking.read.get(),
                    current,
                }
            };
        }
    }

    pub fn update(&self, component: &Component) -> TokenStream {
        let states = &component.states;
        let ident_name = self.ident();

        let update_fn_name = Ident::new(
            &format!("update_{ident_name}"),
            proc_macro2::Span::call_site(),
        );

        let subscriptions_setup = self
            .subscriptions
            .iter()
            .map(|id| {
                let tracked = states[*id].tracked();
                let name = &states[*id].name;
                quote! {
                    let #name = #tracked;
                }
            })
            .chain(self.raw_params.iter().map(|(name, _)| {
                quote! {
                    let #name = self.#name;
                }
            }));

        let subscriptions: Vec<_> = self
            .subscriptions
            .iter()
            .map(|id| states[*id].name.clone())
            .chain(self.raw_params.iter().map(|(r, _)| r).cloned())
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
