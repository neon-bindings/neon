extern crate neon;

use neon::js::JsNumber;

fn main() {
    JsNumber::new(cx, "9000")
    //~^ ERROR E0425
    //     (cannot find value `cx` in this scope)
    //~| ERROR E0277
    //     (trait bound f64: From<&str> is not satisfied)
    //~| ERROR E0308
    //     (mismatched types)
}
