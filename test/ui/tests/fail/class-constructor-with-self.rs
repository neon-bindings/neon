struct Example;

#[neon::class]
impl Example {
    fn new(self) -> Self {
        self
    }
}

fn main() {}
