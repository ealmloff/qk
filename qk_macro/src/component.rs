use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::Parse;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::{parse_quote, ItemFn};

use crate::component_visitor::ComponentVisitor;
use crate::component_visitor_mut::ComponentVisitorMut;
use crate::memo::Memo;
use crate::state::State;

#[derive(Debug, Clone)]
pub struct Component {
    type_name: Ident,
    pub states: Vec<State>,
    pub memos: Vec<Memo>,
    fn_item: ItemFn,
}

impl ToTokens for Component {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let body = &self.fn_item.block;

        let update_states = self
            .states
            .iter()
            .map(|state| {
                let update = state.update();
                quote! {
                    #update
                }
            })
            .chain(self.memos.iter().map(|memo| {
                let update = memo.update(self);
                quote! {
                    #update
                }
            }));

        let comp_name = &self.type_name;
        let types = self
            .states
            .iter()
            .map(|state| state.type_def())
            .chain(self.memos.iter().map(|memo| memo.type_def(self)));

        let create_comp = self
            .states
            .iter()
            .map(|state| {
                let name = &state.name;
                let private = Ident::new(&format!("__{name}"), name.span());

                quote! {
                    #name: #private
                }
            })
            .chain(self.memos.iter().map(|memo| {
                let name = memo.ident();
                let private = Ident::new(&format!("__{name}"), name.span());

                quote! {
                    #name: #private
                }
            }));

        let ident_init = self
            .states
            .iter()
            .map(|state| {
                let name = &state.name;
                let private = Ident::new(&format!("__{name}"), name.span());
                let ty = &state.ty;

                quote! {
                    let mut #private: #ty;
                }
            })
            .chain(self.memos.iter().map(|memo| {
                let name = memo.ident();
                let private = Ident::new(&format!("__{name}"), name.span());
                let ty = memo.ty(self);

                quote! {
                    let mut #private: #ty;
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
            #(#ident_init)*
            #body
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

        visitor.visit_item_fn(&f);

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

        let mut myself = Self {
            type_name,
            states: visitor.states,
            memos: visitor.memos,
            fn_item: parse_quote!(
                fn placeholder() {}
            ),
        };

        ComponentVisitorMut {
            component: &myself,
            memo_idx: 0,
            state_idx: 0,
        }
        .visit_item_fn_mut(&mut f);

        myself.fn_item = f;

        Ok(myself)
    }
}
