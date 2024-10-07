#[neon::export(foo)]
static STRING: &str = "";

#[neon::export(foo)]
fn unsupported() {}

fn main() {}
