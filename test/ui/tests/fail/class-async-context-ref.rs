use neon::prelude::*;

struct Example;

#[neon::class]
impl Example {
    #[neon(async)]
    fn method(&self, _cx: &mut FunctionContext) {}
}

fn main() {}
