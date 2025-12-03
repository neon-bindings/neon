struct Example;

#[neon::class]
impl Example {
    #[neon(async)]
    #[neon(task)]
    fn method(&self) {}
}

fn main() {}
