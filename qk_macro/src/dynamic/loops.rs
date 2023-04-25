use syn::parse::Parse;

pub struct Loop {
    for_token: syn::Token![for],
    pat: syn::Pat,
    in_token: syn::Token![in],
    iterator: syn::Expr,
    body: syn::Block,
}

impl Parse for Loop {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            for_token: input.parse()?,
            pat: input.parse()?,
            in_token: input.parse()?,
            iterator: input.parse()?,
            body: input.parse()?,
        })
    }
}
