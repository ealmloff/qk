use syn::parse::Parse;

pub struct Condition {
    if_token: syn::Token![if],
    condition: syn::Expr,
    body: syn::Block,
    else_token: Option<syn::Token![else]>,
    else_body: Option<syn::Block>,
}

impl Parse for Condition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let if_token = input.parse()?;
        let condition = input.parse()?;
        let body = input.parse()?;

        let mut else_token = None;
        let mut else_body = None;
        if let Ok(parsed_else_token) = input.parse::<syn::Token![else]>() {
            let parsed_else_body = input.parse()?;
            else_token = Some(parsed_else_token);
            else_body = Some(parsed_else_body);
        }
        Ok(Self {
            if_token,
            condition,
            body,
            else_token,
            else_body,
        })
    }
}
