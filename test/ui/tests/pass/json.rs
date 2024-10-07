#[neon::export(json)]
fn wrap_with_json(v: Vec<String>) -> Vec<String> {
    v
}

fn main() {}
