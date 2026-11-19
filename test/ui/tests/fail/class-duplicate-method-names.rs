#[derive(Clone)]
struct Example;

#[neon::class]
impl Example {
    #[neon(name = "get")]
    fn get_a(&self) -> f64 {
        1.0
    }

    #[neon(name = "get")]
    fn get_b(&self) -> f64 {
        2.0
    }
}

fn main() {}
