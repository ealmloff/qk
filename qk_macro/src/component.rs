use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::Parse;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::ItemFn;

use crate::component_visitor::ComponentBuilder;
use crate::component_visitor_mut::ComponentVisitorMut;
use crate::memo::Memo;
use crate::rsx::Elements;
use crate::state::State;

#[derive(Debug)]
pub struct Component {
    pub type_name: Ident,
    pub states: Vec<State>,
    pub memos: Vec<Memo>,
    pub rsx: Elements,
    pub fn_item: ItemFn,
}

impl ToTokens for Component {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let body = &self.fn_item.block.stmts;

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
            .chain(self.memos.iter().map(|memo| memo.type_def(self)))
            .chain(self.rsx.roots.iter().flat_map(|root| {
                root.dynamic_nodes
                    .iter()
                    .map(|dyn_node| dyn_node.type_def())
            }));

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
            }))
            .chain(self.rsx.roots.iter().flat_map(|root| {
                root.dynamic_nodes.iter().map(|dyn_node| {
                    let name = dyn_node.ident();
                    quote! {
                        #name
                    }
                })
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
            }))
            .chain(self.rsx.roots.iter().flat_map(|root| {
                root.dynamic_nodes.iter().map(|dyn_node| {
                    let name = dyn_node.ident();

                    quote! {
                        let mut #name: u32;
                    }
                })
            }));

        tokens.extend(quote! {
            struct #comp_name {
                tracking: DirtyTrackSet<u8, u8>,
                #(#types,)*
            }
            impl #comp_name {
                #(#update_states)*
            }
            let tracking: DirtyTrackSet<u8, u8> = DirtyTrackSet::default();
            #(#ident_init)*
            #(#body)*
            let mut comp = #comp_name {
                tracking,
                #(#create_comp,)*
            };
        })
    }
}

impl Parse for Component {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut f = input.parse::<syn::ItemFn>()?;
        let type_name = f.sig.ident.clone();

        let mut visitor = ComponentBuilder {
            states: Default::default(),
            memos: Default::default(),
            rsx: None,
            fn_item: f.clone(),
            type_name,
            in_reactive: false,
        };

        visitor.visit_item_fn(&f);

        let mut myself = visitor.build();

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
