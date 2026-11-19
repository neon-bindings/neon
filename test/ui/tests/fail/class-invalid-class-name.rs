#[derive(Clone)]
struct Example;

#[neon::export(class, name = "my-class")]
impl Example {
    fn new() -> Self {
        Example
    }
}

fn main() {}
