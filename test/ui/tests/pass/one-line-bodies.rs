// The macros re-emit the user's function body inside a generated wrapper. A
// one-line body must not trip `unused_braces` (the braces are the user's own).
#![deny(unused_braces)]

use neon::prelude::*;

#[neon::export]
fn add(a: f64, b: f64) -> f64 { a + b }

#[neon::export]
fn greet() -> String { "hi".into() }

#[neon::export(json)]
fn numbers() -> Vec<f64> { vec![1.0, 2.0] }

#[neon::export(task)]
fn in_task() -> f64 { 1.0 }

#[neon::export]
async fn in_async() -> f64 { 1.0 }

#[neon::export(context)]
fn with_context(cx: &mut FunctionContext) -> f64 { cx.len() as f64 }

struct Point;

#[neon::export(class)]
impl Point {
    fn new() -> Self { Point }

    fn x(&self) -> f64 { 0.0 }
}

#[neon::main]
fn init(mut cx: ModuleContext) -> NeonResult<()> { neon::registered().export(&mut cx) }

fn main() {}
