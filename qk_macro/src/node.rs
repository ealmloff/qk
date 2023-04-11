use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::Expr;
use syn_rsx::NodeValueExpr;

use crate::format::FormattedText;
use crate::rsx::Root;

#[derive(Debug)]
pub struct DynamicNode {
    pub id: usize,
    pub path: Vec<TraverseOperation>,
    pub node: DynamicNodeType,
}

impl DynamicNode {
    pub fn ident(&self) -> Ident {
        let id = self.id;
        Ident::new(&format!("__dyn_n_{id}"), proc_macro2::Span::call_site())
    }

    fn update(&self) -> TokenStream {
        let id = self.ident();
        match &self.node {
            DynamicNodeType::Element(element) => {
                let attributes = element.attributes.iter().map(|attribute| {
                    let key = &attribute.key;
                    let value = &attribute.value;
                    if let Some(event) = key.strip_prefix("on") {
                        let as_ident = Ident::new(event, proc_macro2::Span::call_site());
                        quote! {
                            ui.add_listener(#id, qk::events::#as_ident, Box::new(#value));
                        }
                    } else {
                        quote! {
                            ui.set_attribute(#id, #key, &#value);
                        }
                    }
                });

                if !element.children.is_empty() {
                    todo!()
                }

                quote! {
                    #(#attributes)*
                }
            }
            DynamicNodeType::Text(text) => {
                let text = &text.text;
                quote! {
                    ui.set_text(#id, &#text);
                }
            }
            DynamicNodeType::Fragment(_) => {
                todo!()
            }
        }
    }
}

#[derive(Debug)]
pub enum DynamicNodeType {
    Element(DynElement),
    Text(DynText),
    Fragment(DynFragment),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TraverseOperation {
    FirstChild,
    NextSibling,
}

#[derive(Debug)]
pub struct DynElement {
    pub attributes: Vec<DynamicAttribute>,
    pub children: Vec<DynamicNode>,
}

#[derive(Debug)]
pub struct DynamicAttribute {
    pub key: String,
    pub value: Expr,
}

#[derive(Debug)]
pub struct DynText {
    pub text: FormattedText,
}

#[derive(Debug)]
pub struct DynFragment {
    pub children: NodeValueExpr,
}

pub fn update_dyn_nodes(roots: &[Root]) -> proc_macro2::TokenStream {
    #[derive(Debug)]
    struct TraverseNode {
        id: Ident,
        next: Option<Box<TraverseNode>>,
        child: Option<Box<TraverseNode>>,
    }

    impl ToTokens for TraverseNode {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let next = if let Some(next) = &self.next {
                let current_id = self.id.clone();
                if next.id != self.id {
                    let next_id = next.id.clone();
                    quote! {
                        ui.copy(#current_id, #next_id);
                        ui.next_sibling(#next_id);
                        #next
                    }
                } else {
                    quote! {
                        ui.next_sibling(#current_id);
                        #next
                    }
                }
            } else {
                quote! {}
            };

            let child = if let Some(child) = &self.child {
                let current_id = self.id.clone();
                if child.id != self.id {
                    let child_id = child.id.clone();
                    quote! {
                        ui.copy(#current_id, #child_id);
                        ui.first_child(#child_id);
                        #child
                    }
                } else {
                    quote! {
                        ui.first_child(#current_id);
                        #child
                    }
                }
            } else {
                quote! {}
            };

            tokens.extend(quote! {
                #next
                #child
            });
        }
    }

    impl TraverseNode {
        // Create any nodes needed to include this node in the tree
        fn insert(&mut self, id: Ident, path: &[TraverseOperation]) {
            match path {
                [TraverseOperation::FirstChild, ..] => {
                    if self.child.is_none() {
                        self.child = Some(Box::new(TraverseNode {
                            id: id.clone(),
                            next: None,
                            child: None,
                        }));
                    }
                    self.child.as_mut().unwrap().insert(id, &path[1..]);
                }
                [TraverseOperation::NextSibling, ..] => {
                    if self.next.is_none() {
                        self.next = Some(Box::new(TraverseNode {
                            id: id.clone(),
                            next: None,
                            child: None,
                        }));
                    }
                    self.next.as_mut().unwrap().insert(id, &path[1..]);
                }
                [] => {}
            }
        }
    }

    let ids: Vec<_> = roots
        .iter()
        .flat_map(|root| root.dynamic_nodes.iter().map(|node| node.ident()))
        .collect();

    let traverse_roots = roots.iter().map(|root| {
        // The roots must be dynamic
        let root_name = root
            .dynamic_nodes
            .iter()
            .enumerate()
            .find(|(_, node)| node.path.is_empty())
            .map(|(idx, _)| ids[idx].clone())
            .unwrap();

        let mut traverse_root = TraverseNode {
            id: root_name,
            next: None,
            child: None,
        };

        for node in &root.dynamic_nodes {
            let id = ids[node.id].clone();
            traverse_root.insert(id, &node.path);
        }

        traverse_root
    });

    let clone_nodes = roots.iter().enumerate().map(|(i, root)| {
        let root_name = root
            .dynamic_nodes
            .iter()
            .enumerate()
            .find(|(_, node)| node.path.is_empty())
            .map(|(idx, _)| ids[idx].clone())
            .unwrap();

        let idx = syn::Index::from(i);

        quote! {
            ui.clone_node(tmpl.#idx, #root_name);
            roots[#i] = #root_name;
        }
    });

    let update_nodes = roots
        .iter()
        .flat_map(|root| root.dynamic_nodes.iter().map(|node| node.update()));

    quote! {
        // initialize all the variables
        #(
            let #ids = ui.node();
        )*

        // create the root
        let tmpl = get_template(ui);
        #(#clone_nodes)*

        // traverse the tree
        #(#traverse_roots)*

        // update the nodes
        #(
            #update_nodes
        )*
    }
}
