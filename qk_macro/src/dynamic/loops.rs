use quote::ToTokens;
use syn::ExprPath;
use syn_rsx::NodeValueExpr;
use syn_rsx::{Node, NodeName};

#[derive(Debug)]
pub struct Loop {
    for_token: NodeName,
    pat: ExprPath,
    in_token: ExprPath,
    iterator: NodeValueExpr,
    children: Vec<Node>,
}

#[test]
fn parses() {
    use quote::quote;
    use syn_rsx::parse2;

    // Create HTML `TokenStream`.
    let tokens = quote! {
        <for x in {0..10}>
        </for>
    };

    // Parse the tokens into a tree of `Node`s.
    let mut nodes = parse2(tokens).unwrap();
    println!("{nodes:#?}");

    // Convert the `Node`s into a `Loop`.
    let loop_ = Loop::try_from(nodes.pop().unwrap()).unwrap();
    println!("{loop_:#?}");
}

impl TryFrom<Node> for Loop {
    type Error = ();

    fn try_from(value: Node) -> Result<Self, Self::Error> {
        match value {
            Node::Element(element) => {
                let for_token = element.name;
                let mut attrs = element.attributes.into_iter();
                let pat = attrs
                    .next()
                    .and_then(|attr| match attr {
                        syn_rsx::Node::Attribute(attr) => match (attr.key, attr.value) {
                            (NodeName::Path(path), None) => Some(path),
                            _ => None,
                        },
                        _ => None,
                    })
                    .ok_or(())?;
                let in_token = attrs
                    .next()
                    .and_then(|attr| match attr {
                        syn_rsx::Node::Attribute(attr) => match (attr.key, attr.value) {
                            (NodeName::Path(path), None) => {
                                (path.to_token_stream().to_string() == "in").then_some(path)
                            }
                            _ => None,
                        },
                        _ => None,
                    })
                    .ok_or(())?;
                let iterator = attrs
                    .next()
                    .and_then(|attr| match attr {
                        syn_rsx::Node::Block(block) => Some(block.value),
                        _ => None,
                    })
                    .ok_or(())?;
                if attrs.next().is_some() {
                    return Err(());
                }
                Ok(Self {
                    for_token,
                    pat,
                    in_token,
                    iterator,
                    children: element.children,
                })
            }
            _ => Err(()),
        }
    }
}
