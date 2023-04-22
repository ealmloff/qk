use syn::{parse::Parse, Ident, Pat, PatType, Type};

#[derive(Debug)]
pub struct Prop {
    pub(crate) name: Ident,
    pub(crate) ty: Type,
    pub(crate) options: Vec<PropOption>,
}

impl From<PatType> for Prop {
    fn from(arg: PatType) -> Self {
        Self {
            name: match *arg.pat {
                Pat::Ident(ref pat) => pat.ident.clone(),
                _ => todo!(),
            },
            ty: *arg.ty,
            options: Default::default(),
        }
    }
}

impl Parse for Prop {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let arg: syn::FnArg = input.parse()?;
        if let syn::FnArg::Typed(arg) = arg {
            Ok(arg.into())
        } else {
            Err(syn::Error::new_spanned(arg, "expected typed argument"))
        }
    }
}

#[derive(Debug)]
pub enum PropOption {}
