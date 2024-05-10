use neon::{prelude::*, types::extract::Error};

#[neon::export]
const NUMBER: u8 = 42;

#[neon::export]
static STRING: &str = "Hello, World!";

#[neon::export(name = "renamedString")]
static RENAMED_STRING: &str = STRING;

#[neon::export(json)]
static MESSAGES: &[&str] = &["hello", "neon"];

#[neon::export(name = "renamedMessages", json)]
static RENAMED_MESSAGES: &[&str] = MESSAGES;

#[neon::export]
fn no_args_or_return() {}

#[neon::export]
fn simple_add(a: f64, b: f64) -> f64 {
    a + b
}

#[neon::export(name = "renamedAdd")]
fn renamed_add(a: f64, b: f64) -> f64 {
    simple_add(a, b)
}

#[neon::export(task)]
fn add_task(a: f64, b: f64) -> f64 {
    simple_add(a, b)
}

#[neon::export(task, name = "renamedAddTask")]
fn renamed_add_task(a: f64, b: f64) -> f64 {
    add_task(a, b)
}

#[neon::export(json)]
fn json_sort(mut items: Vec<String>) -> Vec<String> {
    items.sort();
    items
}

#[neon::export(json, name = "renamedJsonSort")]
fn renamed_json_sort(items: Vec<String>) -> Vec<String> {
    json_sort(items)
}

#[neon::export(json, task)]
fn json_sort_task(items: Vec<String>) -> Vec<String> {
    json_sort(items)
}

#[neon::export(json, name = "renamedJsonSortTask", task)]
fn renamed_json_sort_task(items: Vec<String>) -> Vec<String> {
    json_sort(items)
}

#[neon::export]
fn concat_with_cx_and_handle<'cx>(
    cx: &mut FunctionContext<'cx>,
    a: String,
    b: Handle<'cx, JsString>,
) -> Handle<'cx, JsString> {
    let b = b.value(cx);

    cx.string(a + &b)
}

#[neon::export]
fn fail_with_throw(msg: String) -> Result<(), Error> {
    fn always_fails(msg: String) -> Result<(), String> {
        Err(msg)
    }

    // `?` converts `String` into `Error`
    always_fails(msg)?;

    Ok(())
}

#[neon::export(task)]
fn sleep_task(ms: f64) {
    use std::{thread, time::Duration};

    thread::sleep(Duration::from_millis(ms as u64));
}
