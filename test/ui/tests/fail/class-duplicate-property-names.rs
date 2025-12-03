struct Example;

#[neon::class]
impl Example {
    #[neon(name = "value")]
    const VALUE1: i32 = 42;

    #[neon(name = "value")]
    const VALUE2: i32 = 43;
}

fn main() {}
