struct Example;

impl Example {
    #[neon::export]
    fn borrow(&self) {}

    #[neon::export]
    fn borrow_mut(&mut self) {}

    #[neon::export]
    fn owned(self) {}
}

fn main() {}
