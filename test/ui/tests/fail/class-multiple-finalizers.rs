struct Example;

#[neon::class]
impl Example {
    fn finalize<'a, C: neon::context::Context<'a>>(self, _cx: &mut C) {}
    fn finalize<'a, C: neon::context::Context<'a>>(self, _cx: &mut C) {}
}

fn main() {}
