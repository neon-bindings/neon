use std::future::Future;

use neon::{
    context::{Context, Cx, FunctionContext},
    event::Channel,
    handle::Handle,
    types::{extract::Boxed, JsString},
};

type Ch = Channel;
type FnCx<'cx> = FunctionContext<'cx>;

#[neon::export]
fn sync_nothing() {}

#[neon::export]
fn sync_function_context(_cx: &mut FunctionContext) {}

#[neon::export]
fn sync_cx(_cx: &mut Cx) {}

#[neon::export(context)]
fn sync_cx_forced(_cx: &mut FnCx) {}

#[neon::export]
fn sync_cx_lifetimes<'cx>(cx: &mut Cx<'cx>) -> Handle<'cx, JsString> {
    cx.string("Hello, World!")
}

#[neon::export]
fn sync_this(this: Vec<u8>) {
    let _ = this;
}

#[neon::export(this)]
fn sync_this_forced(_this: Vec<u8>) {}

#[neon::export]
fn sync_cx_and_this(_cx: &mut Cx, this: Vec<u8>) {
    let _ = this;
}

#[neon::export]
fn sync_cx_and_this_and_args(_cx: &mut Cx, this: Vec<u8>, _a: String) {
    let _ = this;
}

#[neon::export]
fn boxed_this(Boxed(this): Boxed<String>) {
    let _ = this;
}

#[neon::export]
async fn async_nothing() {}

#[neon::export]
async fn async_channel(_ch: Channel) {}

#[neon::export(context)]
async fn async_channel_forced(_ch: Ch) {}

#[neon::export]
async fn async_channel_and_arg(_ch: Channel, _a: String) {}

#[neon::export]
async fn async_no_channel(_a: String) {}

#[neon::export]
async fn async_this(this: Vec<u8>) {
    let _ = this;
}

#[neon::export(this)]
async fn async_this_forced(_this: Vec<u8>) {}

#[neon::export]
async fn async_this_args(this: Vec<u8>, _a: String) {
    let _ = this;
}

#[neon::export]
async fn async_this_and_channel(_ch: Channel, this: Vec<u8>) {
    let _ = this;
}

#[neon::export]
async fn async_this_and_channel_args(_ch: Channel, this: Vec<u8>, _a: String, _b: String) {
    let _ = this;
}

#[neon::export(task)]
fn task_nothing() {}

#[neon::export(task)]
fn task_channel(_ch: Channel) {}

#[neon::export(context, task)]
fn task_channel_forced(_ch: Ch) {}

#[neon::export(task)]
fn task_channel_and_arg(_ch: Channel, _a: String) {}

#[neon::export(task)]
fn task_no_channel(_a: String) {}

#[neon::export(task)]
fn task_this(this: Vec<u8>) {
    let _ = this;
}

#[neon::export(task, this)]
fn task_this_forced(_this: Vec<u8>) {}

#[neon::export(task)]
fn task_this_args(this: Vec<u8>, _a: String) {
    let _ = this;
}

#[neon::export(task)]
fn task_this_and_channel(_ch: Channel, this: Vec<u8>) {
    let _ = this;
}

#[neon::export(task)]
fn task_this_and_channel_args(_ch: Channel, this: Vec<u8>, _a: String, _b: String) {
    let _ = this;
}

#[neon::export(async)]
fn impl_async_context(_cx: &mut Cx) -> impl Future<Output = ()> {
    async {}
}

fn main() {}
