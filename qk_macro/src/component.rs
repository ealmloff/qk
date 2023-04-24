use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::Parse;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::ItemFn;

use crate::component_visitor::ComponentBuilder;
use crate::component_visitor_mut::ComponentVisitorMut;
use crate::memo::Memo;
use crate::prop::Prop;
use crate::rsx::Elements;
use crate::state::State;

#[derive(Debug)]
pub struct Component {
    pub type_name: Ident,
    pub states: Vec<State>,
    pub memos: Vec<Memo>,
    pub rsx: Elements,
    pub fn_item: ItemFn,
    pub prop_items: Vec<Prop>,
}

impl Component {
    fn prop_name(&self) -> Ident {
        Ident::new(&format!("{}", self.type_name), self.type_name.span())
    }

    fn props_struct(&self) -> TokenStream {
        let struct_name = self.prop_name();

        let fields = self.prop_items.iter().map(|state| {
            let name = &state.name;
            let ty = &state.ty;

            quote! {
                #name: #ty
            }
        });

        quote! {
            struct #struct_name {
                #(#fields,)*
            }
        }
    }

    fn comp_name(&self) -> Ident {
        Ident::new(&format!("{}State", self.type_name), self.type_name.span())
    }
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
            .chain(self.memos.iter().filter_map(|memo| {
                (!memo.runs_once()).then(|| {
                    let update = memo.update(self);
                    quote! {
                        #update
                    }
                })
            }));

        let comp_name = self.comp_name();
        let types = self
            .states
            .iter()
            .map(|state| state.type_def())
            .chain(
                self.memos
                    .iter()
                    .filter_map(|memo| (!memo.runs_once()).then(|| memo.type_def(self))),
            )
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
            .chain(self.memos.iter().filter_map(|memo| {
                (!memo.runs_once()).then(|| {
                    let name = memo.ident();
                    let private = Ident::new(&format!("__{name}"), name.span());

                    quote! {
                        #name: #private
                    }
                })
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
            .chain(self.memos.iter().filter_map(|memo| {
                (!memo.runs_once()).then(|| {
                    let name = memo.ident();
                    let private = Ident::new(&format!("__{name}"), name.span());
                    let ty = memo.ty(self);

                    quote! {
                        let mut #private: #ty;
                    }
                })
            }))
            .chain(self.rsx.roots.iter().flat_map(|root| {
                root.dynamic_nodes.iter().map(|dyn_node| {
                    let name = dyn_node.ident();

                    quote! {
                        let mut #name: u32 = 0;
                    }
                })
            }));

        let roots = self.rsx.roots.iter().map(|root| {
            let name = root.root_ident();
            quote! {
                #name
            }
        });

        let listeners = self.rsx.roots.iter().map(|root|{
            let dynamic_nodes = &root.dynamic_nodes;

            let listeners = dynamic_nodes.iter().filter_map(|dyn_node|{
                dyn_node.listeners(&self.states, &self.comp_name())
            }).map(|listener|{
                quote! {
                    #listener
                }
            });

            quote! {
                #(#listeners)*
            }
        });

        let prop_name = self.prop_name();
        let props_struct = self.props_struct();

        tokens.extend(quote! {
            #props_struct

            struct #comp_name<R: qk::renderer::Renderer<R> + qk::events::PlatformEvents> {
                tracking: DirtyTrackSet<u8, u8>,
                ui: R,
                #(#types,)*
            }
            impl<R: qk::renderer::Renderer<R> + qk::events::PlatformEvents> #comp_name<R> {
                #(#update_states)*
            }

            impl<R: qk::renderer::Renderer<R> + qk::events::PlatformEvents + Clone + 'static> qk::component::Component<R, R> for #prop_name {
                type State = std::rc::Rc<std::cell::RefCell<#comp_name<R>>>;
                
                fn create(mut ui: R, props: Self) -> Self::State {
                    let tracking: DirtyTrackSet<u8, u8> = DirtyTrackSet::default();
                    let ui = &mut ui;
                    #(#ident_init)*
                    #(#body)*
                    let mut comp = #comp_name {
                        tracking,
                        ui: ui.clone(),
                        #(#create_comp,)*
                    };

                    let comp = std::rc::Rc::new(std::cell::RefCell::new(comp));

                    #(#listeners)*

                    comp
                }
            }

            impl<R: qk::renderer::Renderer<R> + qk::events::PlatformEvents> qk::component::ComponentState<R, R> for #comp_name<R> {
                fn roots(&self) -> Vec<u32> {
                    vec![#(self.#roots,)*]
                }
            }
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
