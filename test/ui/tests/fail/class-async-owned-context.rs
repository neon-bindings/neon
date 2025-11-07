struct Example;

#[neon::class]
impl Example {
    #[neon(async)]
    fn method(&self, _cx: FunctionContext) {}
}

fn main() {}
