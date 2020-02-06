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

    // Uncomment for testing Regex error messages
//    match RouteGroup3 {
//        "[[:ala:]" => Procedure4,
//    }
}

pub fn main() {}
