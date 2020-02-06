use syn::{Visibility, Ident, Attribute, LitStr, Path, Result, Token};
use syn::parse::{Parse, ParseBuffer};
use std::collections::LinkedList;
use syn::group::parse_braces;
use std::iter::FromIterator;
use proc_macro_error::proc_macro::TokenStream;
use regex::Regex;

pub struct Block(pub Vec<Match>);

impl Parse for Block {

    fn parse(input: &ParseBuffer) -> Result<Self> {
        let mut list = LinkedList::new();
        while !input.is_empty() {
            let m = input.parse::<Match>()?;
            list.push_back(m);
        }
        Ok(Block(Vec::from_iter(list.into_iter())))
    }
}

impl Block {

    pub fn check(&self) {
        // Try compiling Regex
        for m in &self.0 {
            for pair in &m.pairs {
                let s = pair.path.value();
                if let Err(e) = Regex::new(&s) {
                    emit_error!(
                        pair.path.span(),
                        e
                    );
                }
            }
        }
    }

    pub fn expand(&self) -> TokenStream {
        // TODO
        TokenStream::new()
    }
}

pub struct Match {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    name: Ident,
    pairs: Vec<Pair>,
}

impl Parse for Match {

    fn parse(input: &ParseBuffer) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let visibility = input.parse::<Visibility>()?;
        input.parse::<Token![match]>()?;
        let name = input.parse::<Ident>()?;
        let pairs = {
            let b = parse_braces(input)?.content;
            let mut p = LinkedList::new();
            while !b.is_empty() {
                let pair = b.parse::<Pair>()?;
                p.push_back(pair);
                let l = b.lookahead1();
                if l.peek(Token![,]) {
                    b.parse::<Token![,]>()?;
                } else if !b.is_empty() {
                    return Err(l.error());
                }
            }
            Vec::from_iter(p.into_iter())
        };
        Ok(Match {
            attrs,
            visibility,
            name,
            pairs,
        })
    }
}

pub struct Pair {
    attrs: Vec<Attribute>,
    path: LitStr,
    proc: Path,
}

impl Parse for Pair {

    fn parse(input: &ParseBuffer) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let path = input.parse::<LitStr>()?;
        input.parse::<Token![=>]>()?;
        let proc = input.parse::<Path>()?;
        Ok(Pair {
            attrs,
            path,
            proc,
        })
    }
}
