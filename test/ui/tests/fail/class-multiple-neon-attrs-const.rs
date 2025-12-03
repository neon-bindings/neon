struct Example;

#[neon::class]
impl Example {
    #[neon(name = "foo")]
    #[neon(json)]
    const VALUE: i32 = 42;
}

fn main() {}
