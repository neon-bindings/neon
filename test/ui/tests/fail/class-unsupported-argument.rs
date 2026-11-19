#[derive(Clone)]
struct Example;

#[neon::class(name = "Renamed")]
impl Example {
    fn new() -> Self {
        Example
    }
}

fn main() {}
