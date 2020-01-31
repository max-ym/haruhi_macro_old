use syn::{Path, Attribute, LitInt, Visibility, Ident};
use syn::parse::{Parse, ParseBuffer, Result};
use syn::spanned::Spanned;
use syn::Token;
use std::collections::LinkedList;
use proc_macro_error::proc_macro::TokenStream;
use syn::ext::IdentExt;
use syn::group::parse_braces;

pub struct Pair {
    code: LitInt,
    data_ty: Path,
    doc: Vec<Attribute>,
}

impl Parse for Pair {

    fn parse(input: &ParseBuffer) -> Result<Self> {
        let doc = {
            let mut doc = Vec::with_capacity(2);
            let attrs = input.call(Attribute::parse_outer)?;
            for attr in attrs {
                let is_doc = {
                    let mut result = false;
                    if attr.path.segments.len() == 1 {
                        if attr.path.segments.first().unwrap().ident.to_string() == "doc" {
                            result = true;
                        }
                    }
                    result
                };
                if is_doc {
                    doc.push(attr);
                } else {
                    emit_error!(
                        attr.span(),
                        "Unexpected attribute type"
                    );
                }
            }
            doc
        };

        let code = input.parse::<LitInt>()?;
        input.parse::<Token![=>]>()?;
        let data_ty = input.parse::<Path>()?;

        Ok(Pair {
            code,
            data_ty,
            doc,
        })
    }
}

pub struct Body(pub LinkedList<Pair>);

impl Parse for Body {

    fn parse(input: &ParseBuffer) -> Result<Self> {
        let input = parse_braces(input)?.content;
        let mut pairs = LinkedList::new();
        while !input.is_empty() {
            let pair = input.parse::<Pair>()?;
            pairs.push_back(pair);
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else if !input.is_empty() {
                return Err(lookahead.error());
            }
        }
        Ok(Body(pairs))
    }
}

pub struct Match {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    name: Ident,
    body: Body,
}

impl Parse for Match {

    fn parse(input: &ParseBuffer) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let visibility = input.parse::<Visibility>()?;
        let operation = input.call(Ident::parse_any)?;
        if operation.to_string() != "match" {
            abort!(
                operation.span(),
                "Unknown operation. Expected `match`"
            );
        }
        let name = input.parse::<Ident>()?;
        let body = input.parse::<Body>()?;
        if body.0.is_empty() {
            emit_warning!(
                name.span(),
                "Empty match is unusable. Route handler which returns this Response type \
                will have no variant to use as a result of its execution"
            );
        }
        Ok(Match {
            attrs,
            visibility,
            name,
            body
        })
    }
}

pub struct Block(pub LinkedList<Match>);

impl Parse for Block {

    fn parse(input: &ParseBuffer) -> Result<Self> {
        let mut list = LinkedList::new();
        while !input.is_empty() {
            let block = input.parse::<Match>()?;
            list.push_back(block);
        }
        Ok(Block(list))
    }
}

impl Block {

    pub fn expand(&self) -> TokenStream {
        // TODO
        TokenStream::new()
    }
}
