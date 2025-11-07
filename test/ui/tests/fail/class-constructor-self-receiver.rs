struct Example;

#[neon::class]
impl Example {
    fn new(&self) -> Self {
        Example
    }
}

fn main() {}
