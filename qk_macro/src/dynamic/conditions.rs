use quote::ToTokens;
use syn::parse::Parse;
use syn_rsx::Node;

pub struct Condition {
    if_token: syn::Token![if],
    condition: syn::Expr,
    body: syn::Block,
    else_token: Option<syn::Token![else]>,
    else_body: Option<syn::Block>,
}

impl Condition {
    fn parse_from(nodes: impl Iterator<Item = Node>) -> Option<Self> {
        let mut nodes = nodes.peekable();

        nodes.next_if(|node| matches!(node, Node::Element(element) if element.name.to_token_stream().to_string() == "if")) .and_then(|if_element|{
            let Node::Element(if_element) = if_element else{unreachable!()};
            if let &[Node::Block(condition)] = &if_element.attributes.as_slice(){
                todo!()
            }else{None}
         })
        // match value {
        //     Node::Element(element) => {
        //         let for_token = element.name;
        //         let mut attrs = element.attributes.into_iter();
        //         let pat = attrs
        //             .next()
        //             .and_then(|attr| match attr {
        //                 syn_rsx::Node::Attribute(attr) => match (attr.key, attr.value) {
        //                     (NodeName::Path(path), None) => Some(path),
        //                     _ => None,
        //                 },
        //                 _ => None,
        //             })
        //             .ok_or(())?;
        //         let (in_token, iterator) = attrs
        //             .next()
        //             .and_then(|attr| match attr {
        //                 syn_rsx::Node::Attribute(attr) => match (attr.key, attr.value) {
        //                     (NodeName::Path(path), Some(iterator)) => {
        //                         (path.to_token_stream().to_string() == "in")
        //                             .then_some((path, iterator))
        //                     }
        //                     _ => None,
        //                 },
        //                 _ => None,
        //             })
        //             .ok_or(())?;
        //         if attrs.next().is_some() {
        //             return Err(());
        //         }
        //         Ok(Self {
        //             for_token,
        //             pat,
        //             in_token,
        //             iterator,
        //             children: element.children,
        //         })
        //     }
        //     _ => Err(()),
        // }
    }
}

#[test]
fn parses() {
    use quote::quote;
    use syn_rsx::parse2;

    // Create HTML `TokenStream`.
    let tokens = quote! {
        <if {true}>
        </if>
        <elif {true}>
        </elif>
        <else>
        </else>
    };

    // Parse the tokens into a tree of `Node`s.
    let mut nodes = parse2(tokens).unwrap();
    println!("{nodes:#?}");

    // // Convert the `Node`s into a `Loop`.
    // let loop_ = Loop::try_from(nodes.pop().unwrap()).unwrap();
    // println!("{loop_:#?}");
}
