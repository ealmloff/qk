mod data;
mod node;

use node::{
    update_dyn_nodes, DynElement, DynText, DynamicAttribute, DynamicNode, TraverseOperation,
};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use slotmap::{DefaultKey, Key, SlotMap};
use syn::{parse::Parse, parse_macro_input, parse_str, Expr, ExprLit, Lit};
use syn_rsx::{Node, NodeAttribute, NodeElement, NodeText, ParserConfig};

#[proc_macro]
pub fn rsx(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Elements);

    TokenStream::from(quote! {
        #input
    })
}

struct Elements {
    elements: Vec<Node>,
}

impl Parse for Elements {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let elements = syn_rsx::Parser::new(ParserConfig::default()).parse(input)?;

        Ok(Self { elements })
    }
}

impl ToTokens for Elements {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let elements = &self.elements;

        let builder = ElementBuilder::new(elements);

        let creation = builder.creation;
        let roots = builder.roots;

        let return_roots = roots.iter().map(|root| match root {
            Root::Static(id) => node_ident(*id).to_token_stream(),
            Root::Dynamic(id) => id.to_token_stream(),
        });

        let return_type: Vec<_> = roots
            .iter()
            .map(|root| match root {
                Root::Static(_) => quote! { u32 },
                Root::Dynamic(_) => quote! { u32 },
            })
            .collect();

        let update_dynamic_nodes = update_dyn_nodes(&builder.dynamic_nodes);

        tokens.extend(quote! {
            fn get_template<P: PlatformEvents>(mut ui: impl Renderer<P>) -> (#(#return_type),*) {
                static mut TEMPLATE: Option<(#(#return_type),*)> = None;
                match unsafe { TEMPLATE } {
                    Some(id) => id,
                    None => {
                        #creation
                        let ids = (#(#return_roots),*);
                        unsafe { TEMPLATE = Some(ids) };
                        ids
                    }
                }
            }

            #update_dynamic_nodes
        });
    }
}

struct ElementBuilder {
    slots: SlotMap<DefaultKey, ()>,
    roots: Vec<Root>,
    creation: proc_macro2::TokenStream,
    dynamic_nodes: Vec<DynamicNode>,
    current_path: Vec<TraverseOperation>,
}

impl ElementBuilder {
    fn new(elements: &Vec<Node>) -> Self {
        let mut myself = Self {
            slots: SlotMap::new(),
            creation: Default::default(),
            roots: Default::default(),
            dynamic_nodes: Default::default(),
            current_path: Default::default(),
        };

        for element in elements {
            let root = myself.build_node(element);
            myself.roots.extend(root);
        }

        myself
    }

    fn build_node(&mut self, node: &Node) -> Vec<Root> {
        match node {
            Node::Element(el) => vec![Root::Static(self.build_element(el))],
            Node::Attribute(_) => todo!(),
            Node::Text(text) => vec![Root::Static(self.build_text(text))],
            Node::Comment(_) => todo!(),
            Node::Doctype(_) => todo!(),
            Node::Block(_) => todo!(),
            Node::Fragment(_) => todo!(),
        }
    }

    fn build_element(&mut self, element: &NodeElement) -> DefaultKey {
        let NodeElement {
            name,
            attributes,
            children,
        } = element;

        let name = name.to_string();

        let id = self.slots.insert(());
        let ident = node_ident(id);

        self.creation.extend(quote! {
            let #ident = ui.node();
            ui.create_element(#ident, #name);
        });

        let mut dyn_attributes = Vec::new();

        for attr in attributes {
            let Node::Attribute(attr) = attr else {
                panic!("Only attributes are supported here");
            };

            let NodeAttribute { key, value } = attr;

            let key = key.to_string();
            let value = value.as_ref().unwrap().as_ref();

            if key.starts_with("on") {
                dyn_attributes.push(DynamicAttribute {
                    key,
                    value: value.clone(),
                });
            } else if let Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str),
                ..
            }) = &value
            {
                let value = lit_str.value();
                self.creation.extend(quote! {
                    ui.set_attribute(#ident, #key, #value);
                });
            } else {
                dyn_attributes.push(DynamicAttribute {
                    key,
                    value: value.clone(),
                });
            }
        }

        if !dyn_attributes.is_empty() {
            let id = self.dynamic_nodes.len();
            self.dynamic_nodes.push(DynamicNode {
                id,
                path: self.current_path.clone(),
                node: node::DynamicNodeType::Element(DynElement {
                    attributes: dyn_attributes,
                    children: Default::default(),
                }),
            });
        }

        let prev_path = self.current_path.clone();

        self.current_path.push(TraverseOperation::FirstChild);

        for child in children {
            let children = self.build_node(child);
            for child in children {
                self.creation.extend(child.append_children(&ident));
            }
            self.current_path.push(TraverseOperation::NextSibling);
        }

        self.current_path = prev_path;

        id
    }

    fn build_text(&mut self, text: &NodeText) -> DefaultKey {
        let id = self.slots.insert(());
        let ident = node_ident(id);

        let value = text.value.as_ref();

        if let Expr::Lit(ExprLit {
            lit: Lit::Str(lit_str),
            ..
        }) = &value
        {
            let value = lit_str.value();
            if value.starts_with('{') && value.ends_with('}') {
                let id = self.dynamic_nodes.len();
                let value = parse_str(&value[1..value.len() - 1]).unwrap();

                self.dynamic_nodes.push(DynamicNode {
                    id,
                    path: self.current_path.clone(),
                    node: node::DynamicNodeType::Text(DynText { text: value }),
                });

                // create a placeholder
                self.creation.extend(quote! {
                    let #ident = ui.node();
                    ui.create_text(#ident, " ");
                });
            } else {
                self.creation.extend(quote! {
                    let #ident = ui.node();
                    ui.create_text(#ident, #value);
                });
            }
        } else {
            let id = self.dynamic_nodes.len();
            self.dynamic_nodes.push(DynamicNode {
                id,
                path: self.current_path.clone(),
                node: node::DynamicNodeType::Text(DynText {
                    text: value.clone(),
                }),
            });

            // create a placeholder
            self.creation.extend(quote! {
                let #ident = ui.node();
                ui.create_text(#ident, " ");
            });
        }

        id
    }
}

enum Root {
    Static(DefaultKey),
    Dynamic(Ident),
}

impl Root {
    fn append_children(&self, to: &Ident) -> proc_macro2::TokenStream {
        match self {
            Root::Static(key) => {
                let key = node_ident(*key);
                quote! {
                    ui.append_child(#to, #key);
                    ui.return_node(#key);
                }
            }

            Root::Dynamic(dynamic) => quote! {
                ui.append_all(#to, #dynamic);
            },
        }
    }
}

fn node_ident(id: DefaultKey) -> proc_macro2::Ident {
    let id = id.data().as_ffi();
    proc_macro2::Ident::new(&format!("__n_{id}"), proc_macro2::Span::call_site())
}
