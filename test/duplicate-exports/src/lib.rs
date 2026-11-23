//! An addon whose exports collide by name. Loading it must fail with an error
//! naming every duplicate (#1152). See `test.js`.

mod v1 {
    #[neon::export]
    fn parse(input: String) -> f64 {
        input.parse().unwrap_or(0.0)
    }
}

mod v2 {
    #[neon::export]
    fn parse(input: String, strict: bool) -> String {
        format!("v2: {input} strict={strict}")
    }
}

// Different Rust names, same JS name
#[neon::export(name = "greet")]
fn greet_a() -> String {
    "greet_a".into()
}

#[neon::export(name = "greet")]
fn greet_b() -> String {
    "greet_b".into()
}

// A function and a const under the same JS name
#[neon::export]
fn version() -> String {
    "fn".into()
}

#[neon::export(name = "version")]
const VERSION: &str = "const";

// A class and a static under the same JS name
struct Counter;

#[neon::export(class)]
impl Counter {
    fn new() -> Self {
        Counter
    }
}

#[neon::export(name = "Counter")]
static COUNTER: f64 = 0.0;

// Not a duplicate
#[neon::export]
fn unique() -> f64 {
    1.0
}
