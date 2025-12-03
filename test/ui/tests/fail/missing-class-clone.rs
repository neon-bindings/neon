struct Point {
    x: i32,
    y: i32,
}

#[neon::class]
impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    fn equals(&self, other: Point) -> bool {
        self.x == other.x && self.y == other.y
    }
}

fn main() {}
