#[neon::export]
fn misspelled(#[neon(ts_typ = "number")] x: f64) -> f64 {
    x
}

struct Point;

#[neon::class]
impl Point {
    fn new() -> Self {
        Point
    }

    fn scale(&self, #[neon(ts_typ = "number")] factor: f64) -> f64 {
        factor
    }
}

fn main() {}
