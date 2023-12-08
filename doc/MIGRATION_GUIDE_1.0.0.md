# Neon 1.0.0 Migration Guide

> **Note:** This migration guide assumes a project is using Neon 0.10 without Node-API backend. If using an older version or the legacy backend, see the [previous migration guide](MIGRATION_GUIDE_0.10.md). 

The Neon 1.0 has stabilized and brings a more consistent and ergonomic API. There are a few (minor) breaking changes and this guide will help walk through them! 

## Removed Traits

A few traits have been removed because they were either redundant or only used for features that no longer exist.

### `Managed`

The `Managed` trait marked values that were _managed_ by the JavaScript VM. It was redundant with the `Value` trait. Trait bounds referencing `Managed` may be either removed or replaced with `Value`.

#### Before

```rust
fn example<V>(h: Handle<V>)
where
    V: Managed,
{
}
```

#### After

```rust
fn example<V>(h: Handle<V>)
where
    V: Value,
{
}
```

### `CallContext`, `This`, and `T` in `JsFunction<T>`

The `This` trait marked values for `cx.this()` in `JsFunction`. However, it was not type checked and could result in a panic at runtime. Instead, `cx.this()` always returns a `JsValue`. Since, `JsFunction`'s `T` parameter had a default, in many cases no changes are necessary. In some cases, the `T` parameter will need to be removed and a `downcast` added.

#### Before

```rust
// `CallContext<JsObject>` is equivalent to `FunctionContext`
fn example(mut cx: CallContext<JsObject>) -> JsResult<JsUndefined> {
    let a = cx.this().get::<JsValue, _, _>(&mut cx, "a")?;

    Ok(cx.undefined())
}
```

#### After

```rust
fn example(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let a = cx.this::<JsObject>()?.get::<JsValue, _, _>(&mut cx, "a")?;

    Ok(cx.undefined())
}
```

### `JsResultExt`

The `JsResultExt` trait provided a `.or_throw(&mut cx)` to allow converting a Rust error into a JavaScript exception. However, it required `T: Value`. It has been replaced with a more generic `ResultExt` trait. Most usages only require replacing `JsResultExt` with `ResultExt`. In some cases, an additional `T: Value` bound will need to be added or removed.

#### Before

```rust
use neon::result::JsResultExt;
```

#### After

```rust
use neon::result::ResultExt;
```

## `usize` indexes and lengths

Neon inconsistently used `u32`, `usize`, and sometimes even `i32` for indexes and lengths. For consistency with Rust, `usize` is used everywhere. Update explicit types to use `usize` and remove type casting. Implicit types do not need to be updated.

#### Before

```rust
fn example(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let arr = cx.empty_array();
    let msg = cx.string("Hello!");

    arr.set(&mut cx, 0u32, msg)?;

    Ok(cx.undefined())
}
```

#### After

```rust
fn example(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let arr = cx.empty_array();
    let msg = cx.string("Hello!");

    arr.set(&mut cx, 0usize, msg)?;

    Ok(cx.undefined())
}
```

## Feature Flags

Neon `0.10` made extensive use of feature flags for features that had not been stabilized (e.g., `try-catch-api`, `channel-api`). All features have been stabilized and the feature flags removed. Resolve by removing these features from the project's `Cargo.toml`.

Two feature flags are still exist: `napi-N` for the Node-API version and `futures` for compatibility between Rust `Future` and JavaScript `Promise`.
