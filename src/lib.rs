#![recursion_limit = "128"]
#![allow(dead_code)]

extern crate proc_macro;
#[macro_use]
extern crate proc_macro_error;
extern crate regex;

use proc_macro_error::*;
use crate::proc_macro::TokenStream;
use syn::parse_macro_input;

//mod process;

mod response;

mod route;

#[proc_macro]
#[proc_macro_error]
pub fn response(tokens: TokenStream) -> TokenStream {
    let block = parse_macro_input!(tokens as response::Block);
    block.expand()
}

#[proc_macro_error]
#[proc_macro]
pub fn route(tokens: TokenStream) -> TokenStream {
    let block = parse_macro_input!(tokens as route::Block);
    block.check();
    block.expand()
}

#[test]
fn main_test() {
    let t = trybuild::TestCases::new();
    t.pass("src/tests/*.rs");
}
