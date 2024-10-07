#[neon::export]
fn owned_cx(_cx: Cx) {}

#[neon::export]
fn owned_function_cx(_cx: FunctionContext) {}

#[neon::export]
fn ref_cx(_cx: &Cx) {}

#[neon::export]
fn ref_function_cx(_cx: &FunctionContext) {}

#[neon::export(context)]
fn forced_cx(_cx: String) {}

#[neon::export(context)]
fn forced_ref_cx(_cx: &String) {}

fn main() {}
