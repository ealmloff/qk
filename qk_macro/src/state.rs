use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::collections::HashSet;
use syn::parse_quote;
use syn::{Expr, Type};

/// State that belongs to a component.
#[derive(Clone)]
pub struct State {
    pub id: usize,
    pub name: Ident,
    pub ty: Type,
    pub expr: Expr,
    pub subscribers: HashSet<usize>,
}

impl State {
    pub fn type_def(&self) -> TokenStream {
        let name = &self.name;
        let ty = &self.ty;
        quote! {
            #name: #ty
        }
    }

    pub fn tracked(&self) -> TokenStream {
        let name = &self.name;
        let id = self.id as u8;

        quote! {
            RwTrack {
                data: &mut self.#name,
                tracking: self.tracking.track(#id),
            }
        }
    }

    pub fn construct(&self) -> syn::Stmt {
        let name = &self.name;
        let expr = &self.expr;

        let private_name = Ident::new(&format!("__{name}"), name.span());

        parse_quote! {
            #private_name = #expr;
        }
    }

    pub fn construct_tracked(&self) -> TokenStream {
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

    pub fn tracked_type(&self) -> TokenStream {
        let ty = &self.ty;
        quote! {
            RwTrack<#ty, u8, u8>,
        }
    }

    pub fn update_fn(&self) -> Ident {
        let name = &self.name;

        Ident::new(&format!("update_{name}"), name.span())
    }

    pub fn update(&self) -> TokenStream {
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
