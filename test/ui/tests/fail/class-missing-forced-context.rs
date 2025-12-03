struct Example;

#[neon::class]
impl Example {
    #[neon(context)]
    fn method(&self) {}
}

fn main() {}
