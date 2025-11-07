struct Example;

#[neon::class]
impl Example {
    fn method(&self, _ch: &Channel) {}
}

fn main() {}
