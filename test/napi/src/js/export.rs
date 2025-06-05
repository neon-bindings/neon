use neon::{
    prelude::*,
    types::extract::{Boxed, Error},
};

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
fn rs_renamed_add(a: f64, b: f64) -> f64 {
    simple_add(a, b)
}

#[neon::export(task)]
fn add_task(a: f64, b: f64) -> f64 {
    simple_add(a, b)
}

#[neon::export(task, name = "renamedAddTask")]
fn rs_renamed_add_task(a: f64, b: f64) -> f64 {
    add_task(a, b)
}

#[neon::export(json)]
fn json_sort(mut items: Vec<String>) -> Vec<String> {
    items.sort();
    items
}

#[neon::export(json, name = "renamedJsonSort")]
fn rs_renamed_json_sort(items: Vec<String>) -> Vec<String> {
    json_sort(items)
}

#[neon::export(json, task)]
fn json_sort_task(items: Vec<String>) -> Vec<String> {
    json_sort(items)
}

#[neon::export(json, name = "renamedJsonSortTask", task)]
fn rs_renamed_json_sort_task(items: Vec<String>) -> Vec<String> {
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

#[neon::export]
fn number_with_cx<'cx>(cx: &mut Cx<'cx>, n: f64) -> Handle<'cx, JsNumber> {
    cx.number(n)
}

#[neon::export]
fn simple_self(this: Handle<JsObject>) -> Handle<JsObject> {
    this
}

#[neon::export]
fn boxed_self(Boxed(this): Boxed<String>) -> String {
    this
}

#[neon::export]
fn boxed_string(s: String) -> Boxed<String> {
    Boxed(s)
}

#[neon::export]
fn add_i32(a: i32, b: i32) -> i32 {
    a + b
}

#[neon::export]
fn add_u32(a: u32, b: u32) -> u32 {
    a + b
}
