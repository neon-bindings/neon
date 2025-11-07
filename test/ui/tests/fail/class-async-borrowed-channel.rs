struct Example;

#[neon::class]
impl Example {
    #[neon(async)]
    fn method(&self, _ch: &Channel) {}
}

fn main() {}
