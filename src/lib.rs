#![recursion_limit = "128"]
#![allow(dead_code)]

extern crate proc_macro;
#[macro_use]
extern crate proc_macro_error;

use proc_macro_error::*;
use crate::proc_macro::TokenStream;
use syn::parse_macro_input;

//mod process;

mod response;

#[proc_macro]
#[proc_macro_error]
pub fn response(tokens: TokenStream) -> TokenStream {
    let block = parse_macro_input!(tokens as response::Block);
    block.expand()
}

#[test]
fn response_test() {
    let t = trybuild::TestCases::new();
    t.pass("src/tests/response.rs");
}
