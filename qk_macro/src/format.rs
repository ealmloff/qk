use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use std::str::FromStr;
use syn::{
    parse::{Parse, ParseStream},
    *,
};

#[proc_macro]
pub fn format_diffable_args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(input as Parsed);
    quote!(#parsed).into()
}

#[allow(dead_code)] // dumb compiler does not see the struct being used...
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct Parsed {
    pub source: Option<LitStr>,
    pub segments: Vec<Segment>,
}

impl FromStr for Parsed {
    type Err = syn::Error;

    fn from_str(input: &str) -> Result<Self> {
        let mut chars = input.chars().peekable();
        let mut segments = Vec::new();
        let mut current_literal = String::new();
        while let Some(c) = chars.next() {
            if c == '{' {
                if let Some(c) = chars.next_if(|c| *c == '{') {
                    current_literal.push(c);
                    continue;
                }
                if !current_literal.is_empty() {
                    segments.push(Segment::Literal(current_literal));
                }
                current_literal = String::new();
                let mut current_captured = String::new();
                while let Some(c) = chars.next() {
                    if c == ':' {
                        let mut current_format_args = String::new();
                        for c in chars.by_ref() {
                            if c == '}' {
                                segments.push(Segment::Formatted(FormattedSegment {
                                    format_args: current_format_args,
                                    segment: FormattedSegmentType::parse(&current_captured)?,
                                }));
                                break;
                            }
                            current_format_args.push(c);
                        }
                        break;
                    }
                    if c == '}' {
                        segments.push(Segment::Formatted(FormattedSegment {
                            format_args: String::new(),
                            segment: FormattedSegmentType::parse(&current_captured)?,
                        }));
                        break;
                    }
                    current_captured.push(c);
                }
            } else {
                if '}' == c {
                    if let Some(c) = chars.next_if(|c| *c == '}') {
                        current_literal.push(c);
                        continue;
                    } else {
                        return Err(Error::new(
                            Span::call_site(),
                            "unmatched closing '}' in format string",
                        ));
                    }
                }
                current_literal.push(c);
            }
        }
        if !current_literal.is_empty() {
            segments.push(Segment::Literal(current_literal));
        }
        Ok(Self {
            segments,
            source: None,
        })
    }
}

impl ToTokens for Parsed {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut literals = Vec::new();
        let mut segments = Vec::new();
        let mut last_was_literal = false;
        for segment in self.segments.iter() {
            match segment {
                Segment::Literal(s) => {
                    last_was_literal = true;
                    literals.push(s.replace('{', "{{").replace('}', "}}"))
                }
                Segment::Formatted(FormattedSegment {
                    format_args,
                    segment,
                }) => {
                    if !last_was_literal {
                        literals.push(String::new());
                    }
                    last_was_literal = false;
                    if format_args.is_empty() {
                        segments.push(match segment {
                            FormattedSegmentType::Expr(expr) => quote! {
                                #expr
                            },
                            FormattedSegmentType::Ident(ident) => quote! {
                                #ident
                            },
                        });
                    } else {
                        let mut format_literal = String::new();
                        format_literal += "{";
                        format_literal += ":";
                        format_literal += format_args;
                        format_literal += "}";
                        match segment {
                            FormattedSegmentType::Expr(expr) => segments.push(quote! {
                                bumpalo::format!(in &bump, #format_literal, #expr)
                            }),
                            FormattedSegmentType::Ident(ident) => segments.push(quote! {
                                bumpalo::format!(in &bump, #format_literal, #ident)
                            }),
                        }
                    }
                }
            }
        }

        if !last_was_literal {
            literals.push(String::new());
        }

        let generated = quote! {
            DiffableArguments {
                static_segments: &[#(#literals),*],
                dynamic_segments: bump.alloc_with(|| {
                    [
                        #((&mut &(#segments)).into_entry(&bump)),*
                    ]
                }),
            }
        };
        generated.to_tokens(tokens)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
enum Segment {
    Literal(String),
    Formatted(FormattedSegment),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct FormattedSegment {
    format_args: String,
    segment: FormattedSegmentType,
}

impl ToTokens for FormattedSegment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let (fmt, seg) = (&self.format_args, &self.segment);
        let fmt = format!("{{0:{fmt}}}");
        tokens.append_all(quote! {
            format_args!(#fmt, #seg)
        });
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
enum FormattedSegmentType {
    Expr(Box<Expr>),
    Ident(Ident),
}

impl FormattedSegmentType {
    fn parse(input: &str) -> Result<Self> {
        if let Ok(ident) = parse_str::<Ident>(input) {
            if ident == input {
                return Ok(Self::Ident(ident));
            }
        }
        if let Ok(expr) = parse_str(input) {
            Ok(Self::Expr(Box::new(expr)))
        } else {
            Err(Error::new(
                Span::call_site(),
                "Expected Ident or Expression",
            ))
        }
    }
}

impl ToTokens for FormattedSegmentType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Expr(expr) => expr.to_tokens(tokens),
            Self::Ident(ident) => ident.to_tokens(tokens),
        }
    }
}

impl Parse for Parsed {
    fn parse(input: ParseStream) -> Result<Self> {
        let input: LitStr = input.parse()?;
        let input_str = input.value();
        let mut ifmt = Parsed::from_str(&input_str)?;
        ifmt.source = Some(input);
        Ok(ifmt)
    }
}
