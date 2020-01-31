extern crate haruhi_macro;
use haruhi_macro::route;

route! {

    pub match RouteGroup1 {
        "/" => Procedure1,
        "/simple" => Procedure2,
        "/*/end" => Procedure3,
    }

    /// DOC
    match RouteGroup2 {
        "/" => Procedure1,

        /// Doc
        "/simple" => Procedure2,
        "/*/end" => Procedure3
    }
}

pub fn main() {}
