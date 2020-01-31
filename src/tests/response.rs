extern crate haruhi_macro;

use haruhi_macro::response;

response! {

    pub match MyResponse1 {}

    /// DOC
    pub match MyResponse2 {
        404 => MyStruct1,
        200 => MyStruct2,
    }

    match MyResponse3 {
        /// DOC1
        /// DOC2
        200 => MyStruct1,

        /// DOC
        201 => MyStruct2
    }
}

pub fn main() {}
