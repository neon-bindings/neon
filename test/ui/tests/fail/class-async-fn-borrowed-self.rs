#[derive(Clone)]
struct Example;

#[neon::class]
impl Example {
    async fn method(&self) {}
}

fn main() {}
