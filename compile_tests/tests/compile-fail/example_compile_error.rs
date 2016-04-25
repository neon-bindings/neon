extern crate neon;

use neon::js::JsInteger;

fn main() {
    JsInteger::new(scope, "9000")
    //~^ ERROR unresolved name
    //~| ERROR mismatched types
    //~| ERROR mismatched types
}
