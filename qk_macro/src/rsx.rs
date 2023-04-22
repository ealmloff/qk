use std::str::FromStr;

use crate::{
    component::Component,
    component_visitor::ComponentBuilder,
    format::{FormattedSegment, FormattedText, Segment},
    node::{
        self, update_dyn_nodes, DynElement, DynText, DynamicAttribute, DynamicNode,
        TraverseOperation,
    },
};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use slotmap::{DefaultKey, Key, SlotMap};
use syn::{parse::Parse, parse_quote, Expr, ExprLit, Lit, Stmt};
use syn_rsx::{Node, NodeAttribute, NodeElement, NodeText, ParserConfig};

#[derive(Debug)]
pub struct Elements {
    slots: SlotMap<DefaultKey, ()>,
    pub roots: Vec<Root>,
    creation: proc_macro2::TokenStream,
    current_path: Vec<TraverseOperation>,
    memo_ids: Vec<usize>,
}

impl Parse for Elements {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let elements = syn_rsx::Parser::new(ParserConfig::default()).parse(input)?;

        Ok(Elements::new(&elements))
    }
}

impl ToTokens for Elements {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let get_template_fn = self.get_template_fn();
        let update_dynamic_nodes = update_dyn_nodes(&self.roots);

        tokens.extend(quote! {
            #get_template_fn
            #update_dynamic_nodes
        });
    }
}

impl Elements {
    fn new(elements: &[Node]) -> Self {
        let mut myself = Self {
            slots: SlotMap::new(),
            creation: Default::default(),
            roots: Default::default(),
            current_path: Default::default(),
            memo_ids: Default::default(),
        };

        for (idx, element) in elements.iter().enumerate() {
            let mut root = Root {
                idx,
                dynamic_nodes: Default::default(),
                root_name: None,
            };
            let nodes = myself.build_node(&mut root, element, true);
            assert_eq!(nodes.len(), 1);
            root.root_name = Some(match &nodes[0] {
                QkNode::Static(id) => node_ident(*id).to_token_stream(),
                QkNode::Dynamic(id) => id.to_token_stream(),
            });
            myself.roots.push(root);
        }

        myself
    }

    pub fn construct_memos(&mut self, builder: &mut ComponentBuilder) {
        // Go through all dynamic nodes and create memos for them
        for root in &mut self.roots {
            for dyn_node in &mut root.dynamic_nodes {
                if let Some(update) = dyn_node.update() {
                    let memo_id = builder.memo(
                        None,
                        update,
                        Some(parse_quote! { move }),
                        vec![(parse_quote!(ui), parse_quote!(&mut R))],
                    );
                    self.memo_ids.push(memo_id);
                }
            }
        }
    }

    pub fn update_memos(&self, comp: &Component) -> Stmt {
        let mut update = TokenStream::new();

        for memo_id in &self.memo_ids {
            let memo = &comp.memos[*memo_id].construct(comp);
            update.extend(quote! {
                #memo
            });
        }

        parse_quote! {
            {
                #update
            }
        }
    }

    fn get_template_fn(&self) -> TokenStream {
        let creation = &self.creation;
        let roots = &self.roots;

        let return_roots: Vec<_> = roots
            .iter()
            .map(|root| root.root_name.as_ref().unwrap())
            .collect();

        let arr_len = roots.len();

        let tuple_idxs: Vec<_> = (0..roots.len()).collect();

        let create_arr = (0..roots.len()).map(|_| quote! { std::sync::atomic::AtomicU32::new(0) });

        quote! {
            fn get_template<R2: qk::renderer::Renderer<R2> + qk::events::PlatformEvents>(mut ui: &mut R2) -> &'static [std::sync::atomic::AtomicU32; #arr_len] {
                static TEMPLATE: [std::sync::atomic::AtomicU32; #arr_len] = [#(#create_arr,)*];

                if unsafe{TEMPLATE.get_unchecked(0)}.load(std::sync::atomic::Ordering::Relaxed) == 0 {
                    #creation
                    // Safety: Checked index at compile time
                    unsafe {
                        #(
                            TEMPLATE.get_unchecked(#tuple_idxs).store(#return_roots, std::sync::atomic::Ordering::Relaxed);
                        )*
                    }
                }

                &TEMPLATE
            }
        }
    }

    fn build_node(&mut self, root: &mut Root, node: &Node, force_dyn: bool) -> Vec<QkNode> {
        match node {
            Node::Element(el) => vec![QkNode::Static(self.build_element(root, el, force_dyn))],
            Node::Attribute(_) => todo!(),
            Node::Text(text) => {
                vec![QkNode::Static(self.build_text(root, text, force_dyn))]
            }
            Node::Comment(_) => todo!(),
            Node::Doctype(_) => todo!(),
            Node::Block(_) => todo!(),
            Node::Fragment(_) => todo!(),
        }
    }

    fn build_element(
        &mut self,
        root: &mut Root,
        element: &NodeElement,
        force_dyn: bool,
    ) -> DefaultKey {
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
                let str_value = lit_str.value();
                let value = FormattedText::from_str(&str_value).unwrap();
                if value.is_dynamic() {
                    dyn_attributes.push(DynamicAttribute {
                        key,
                        value: parse_quote! {#value},
                    });
                } else {
                    self.creation.extend(quote! {
                        ui.set_attribute(#ident, #key, #str_value);
                    });
                }
            } else {
                dyn_attributes.push(DynamicAttribute {
                    key,
                    value: value.clone(),
                });
            }
        }

        if !dyn_attributes.is_empty() || force_dyn {
            let id = root.dynamic_nodes.len();
            root.dynamic_nodes.push(DynamicNode {
                root_id: root.idx,
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
            let children = self.build_node(root, child, false);
            for child in children {
                self.creation.extend(child.append_children(&ident));
            }
            self.current_path.push(TraverseOperation::NextSibling);
        }

        self.current_path = prev_path;

        id
    }

    fn build_text(&mut self, root: &mut Root, text: &NodeText, force_dyn: bool) -> DefaultKey {
        let id = self.slots.insert(());
        let ident = node_ident(id);

        let value = text.value.as_ref();

        if let Expr::Lit(ExprLit {
            lit: Lit::Str(lit_str),
            ..
        }) = &value
        {
            let value = FormattedText::from_str(&lit_str.value()).unwrap();
            if value.is_dynamic() {
                let id = root.dynamic_nodes.len();

                root.dynamic_nodes.push(DynamicNode {
                    root_id: root.idx,
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
                if force_dyn {
                    let id = root.dynamic_nodes.len();

                    root.dynamic_nodes.push(DynamicNode {
                        root_id: root.idx,
                        id,
                        path: self.current_path.clone(),
                        node: node::DynamicNodeType::Text(DynText { text: value }),
                    });
                }
                self.creation.extend(quote! {
                    let #ident = ui.node();
                    ui.create_text(#ident, #lit_str);
                });
            }
        } else {
            let id = root.dynamic_nodes.len();
            root.dynamic_nodes.push(DynamicNode {
                root_id: root.idx,
                id,
                path: self.current_path.clone(),
                node: node::DynamicNodeType::Text(DynText {
                    text: FormattedText {
                        source: None,
                        segments: vec![Segment::Formatted(FormattedSegment {
                            segment: value.clone(),
                            format_args: String::new(),
                        })],
                    },
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

#[derive(Debug)]
pub struct Root {
    pub idx: usize,
    pub dynamic_nodes: Vec<DynamicNode>,
    pub root_name: Option<TokenStream>,
}

impl Root {
    pub fn root_ident(&self) -> proc_macro2::Ident {
        self.dynamic_nodes
            .iter()
            .find(|n| n.path.is_empty())
            .unwrap()
            .ident()
    }
}

pub enum QkNode {
    Static(DefaultKey),
    Dynamic(Ident),
}

impl QkNode {
    fn append_children(&self, to: &Ident) -> proc_macro2::TokenStream {
        match self {
            QkNode::Static(key) => {
                let key = node_ident(*key);
                quote! {
                    ui.append_child(#to, #key);
                    ui.return_node(#key);
                }
            }

            QkNode::Dynamic(dynamic) => quote! {
                ui.append_all(#to, #dynamic);
            },
        }
    }
}

fn node_ident(id: DefaultKey) -> proc_macro2::Ident {
    let id = id.data().as_ffi();
    proc_macro2::Ident::new(&format!("__n_{id}"), proc_macro2::Span::call_site())
}
