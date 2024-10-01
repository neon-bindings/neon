#[neon::export]
static STATIC_STRING: &str = "";

#[neon::export]
const CONST_NUMBER: f64 = 42.0;

#[neon::export]
static STATIC_ARR: &[f64] = &[42.0];

#[neon::export(json)]
static ARR_OF_ARR: &[&[f64]] = &[&[42.0]];

fn main() {}
