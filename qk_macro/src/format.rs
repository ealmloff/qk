use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::str::FromStr;
use syn::{
    parse::{Parse, ParseStream},
    *,
};

#[allow(dead_code)] // dumb compiler does not see the struct being used...
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct FormattedText {
    pub source: Option<LitStr>,
    pub segments: Vec<Segment>,
}

impl FormattedText {
    pub fn is_dynamic(&self) -> bool {
        self.segments
            .iter()
            .any(|s| matches!(s, Segment::Formatted(_)))
    }
}

impl FromStr for FormattedText {
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
                                    segment: parse_str(&current_captured)?,
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
                            segment: parse_str(&current_captured)?,
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

impl ToTokens for FormattedText {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut format_literal = String::new();
        let mut dynamic_segments = Vec::new();

        for segment in self.segments.iter() {
            match segment {
                Segment::Literal(s) => {
                    format_literal += &s.replace('{', "{{").replace('}', "}}");
                }
                Segment::Formatted(FormattedSegment {
                    format_args,
                    segment,
                }) => {
                    format_literal += "{:";
                    format_literal += format_args;
                    format_literal += "}";
                    dynamic_segments.push(segment);
                }
            }
        }

        let segment = dynamic_segments.iter();

        let generated = quote! {
            format!(#format_literal #(, #segment)*)
        };
        generated.to_tokens(tokens)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Segment {
    Literal(String),
    Formatted(FormattedSegment),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct FormattedSegment {
    pub format_args: String,
    pub segment: Expr,
}

impl Parse for FormattedText {
    fn parse(input: ParseStream) -> Result<Self> {
        let input: LitStr = input.parse()?;
        let input_str = input.value();
        let mut ifmt = FormattedText::from_str(&input_str)?;
        ifmt.source = Some(input);
        Ok(ifmt)
    }
}
