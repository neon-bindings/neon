#[derive(Clone)]
struct Example;

#[neon::class]
impl Example {
    #[neon(name = "constructor")]
    fn ctor(&self) -> f64 {
        1.0
    }
}

fn main() {}
