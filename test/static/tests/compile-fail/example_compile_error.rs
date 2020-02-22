extern crate neon;

use neon::types::JsNumber;

fn main() {
    JsNumber::new(cx, "9000")
}
