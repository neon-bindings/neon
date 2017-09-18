extern crate neon;

use neon::js::JsNumber;

fn main() {
    JsNumber::new(scope, "9000")
    //~^ ERROR E0425
    //     (cannot find value `scope` in this scope)
    //~| ERROR E0308
    //     (mismatched types)
    //~| ERROR E0308
    //     (mismatched types)
}
