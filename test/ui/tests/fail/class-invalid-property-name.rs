struct Example;

#[neon::class]
impl Example {
    #[neon(name = "123invalid")]
    const VALUE: i32 = 42;
}

fn main() {}
