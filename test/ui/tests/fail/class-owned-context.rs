use neon::prelude::*;

struct Example;

#[neon::class]
impl Example {
    fn method(&self, _cx: FunctionContext) {}
}

fn main() {}
