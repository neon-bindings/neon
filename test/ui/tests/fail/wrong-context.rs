#[neon::export]
fn sync_channel(_ch: Channel) {}

#[neon::export]
fn sync_borrow_channel(_ch: &mut Channel) {}

#[neon::export(async)]
fn async_channel(_ch: Channel) {}

#[neon::export(async)]
fn async_borrow_channel(_ch: &mut Channel) {}

#[neon::export]
async fn async_cx(_cx: Cx) {}

#[neon::export]
async fn async_function_context(_cx: FunctionContext) {}

#[neon::export]
async fn async_cx_ref(_cx: &Cx) {}

#[neon::export]
async fn async_borrow_channel(_cx: &Channel) {}

#[neon::export(context)]
async fn async_borrow_forced_channel(_cx: &String) {}

#[neon::export]
async fn async_function_context_ref(_cx: &FunctionContext) {}

#[neon::export(task)]
fn task_function_context(_cx: FunctionContext) {}

#[neon::export(task)]
fn task_cx_ref(_cx: &Cx) {}

#[neon::export(task)]
fn task_function_context_ref(_cx: &FunctionContext) {}

fn main() {}
