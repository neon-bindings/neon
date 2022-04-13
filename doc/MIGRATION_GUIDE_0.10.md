# Neon 0.10 Migration Guide

With the API improvements of Neon 0.10, a few backwards-incompatible changes have been introduced. This migration guide provides all the information you should need to make the (small!) code changes required for upgrading to Neon 0.10.

We have made an effort to minimize these changes, and believe they should typically lead to simpler, clearer code. **If you run into any trouble migrating your code to 0.10, please file an issue or reach out for help on the [community Slack](https://rust-bindings.slack.com/)!**

# Major changes

## Object property access is generic

To improve the ergonomics of the common case of downcasting the result of property access to a specific type, the signature of `Object::get()` has been changed to have a generic return type. This means that code that follows `Object::get()` with `.downcast_or_throw()` or `.downcast::<V>().or_throw()` no longer needs to do so.

**Before:**

```rust
obj.get(&mut cx, "name")?.downcast::<V, _>(&mut cx).or_throw(&mut cx)?
```

**After (option 1):**

```rust
obj.get::<V, _, _>(&mut cx, "name")?
```

**After (option 2):**

```rust
let v: Handle<V> = obj.get(&mut cx, "name")?
```

Since `Object::get()` throws an exception when types don't match, use the new `Object::get_value()` or `Object::get_opt()` methods for cases that accept a wider range of allowable types.

**Before:**

```rust
let field: Option<Handle<JsBoolean>> = obj
    .get(&mut cx, "optionalField")?
    .downcast(&mut cx)
    .ok();
```

**After:**

```rust
let field: Option<Handle<JsBoolean>> = obj.get_opt(&mut cx, "optionalField")?;
```

**Before:**

```rust
let length = obj.get(&mut cx, "length")?;
let length: Option<Handle<JsNumber>> = if length.is_a::<JsNull, _>(&mut cx) {
    None
} else {
    Some(length.downcast_or_throw(&mut cx)?)
};
```

**After:**

```rust
let length = obj.get_value(&mut cx, "length")?;
let length: Option<Handle<JsNumber>> = if length.is_a::<JsNull, _>(&mut cx) {
    None
} else {
    Some(length.downcast_or_throw(&mut cx)?)
};
```


## Layered APIs for function calls

The API for calling (or constructing, i.e. the Neon equivalent of the JavaScript `new` operator) a JS function has been split into two layered alternatives. The existing `.call()` and `.construct()` functions are now a lower-level primitive, which no longer offers automatic downcasting of arguments or result. But Neon 0.10 now offers a more convenient API for calling functions with an options object and method chaining, with the introduction of the `.call_with()` and `.construct_with()` methods.

### Example: Calling a function

**Before:**

```rust
let s: Handle<JsString> = ...;
let n: Handle<JsNumber> = ...;
let args: Vec<Handle<JsValue>> = vec![s.upcast(), n.upcast()];
let this = cx.global();
f.call(&mut cx, this, args)
```

**After (low-level API):**

```rust
let s: Handle<JsString> = cx.string("hello");
let n: Handle<JsNumber> = cx.number(42);
let this = cx.global();
f.call(&mut cx, this, [s.upcast(), n.upcast()])
```

**After (high-level API):**

```rust
f.call_with(&cx)
 .args((cx.string("hello"), cx.number(42)))
 .apply(&mut cx)
```

### Example: Constructing with a function

**Before:**

```rust
let s: Handle<JsString> = ...;
let n: Handle<JsNumber> = ...;
let args: Vec<Handle<JsValue>> = vec![s.upcast(), n.upcast()];
f.construct(&mut cx, args)
```

**After (low-level API):**

```rust
let s: Handle<JsString> = ...;
let n: Handle<JsNumber> = ...;
f.construct(&mut cx, [s.upcast(), n.upcast()])
```

**After (high-level API):**

```rust
let s: Handle<JsString> = ...;
let n: Handle<JsNumber> = ...;
f.construct_with(&cx)
 .args((s, n))
 .apply(&mut cx)
```

## Idiomatic typed arrays

Neon 0.10 replaces the previous typed array API with a more idiomatic API. JavaScript typed arrays (e.g. [`Uint32Array`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Uint32Array)) can be represented with corresponding Rust types (e.g. [`JsTypedArray<u32>`](https://docs.rs/neon/0.10.0-alpha.3/neon/types/struct.JsTypedArray.html)), allowing easy access to their internals as Rust slices.

### Example: Reading a buffer

**Before:**

```rust
let b: Handle<JsArrayBuffer> = ...;
{
    let guard = cx.lock();
    let data = b.borrow(&guard);
    let slice = data.as_slice::<u32>();
    ...
}
```

**After:**

```rust
let b: Handle<JsTypedArray<u32>> = ...;
let slice = b.as_slice(&cx);
```

### Example: Reading and writing buffers

**Before:**

```rust
let b1: Handle<JsArrayBuffer> = ...;
let b2: Handle<JsArrayBuffer> = ...;
{
    let guard = cx.lock();
    let data1 = b1.borrow(&guard);
    let data2 = b2.borrow(&guard);
    let slice1 = data1.as_slice::<u32>();
    let slice2 = data2.as_slice::<u32>();
    ...
}
```

**After:**

```rust
let src_buf: Handle<JsTypedArray<u32>> = ...;
let dst_buf: Handle<JsTypedArray<u32>> = ...;
{
    let lock = cx.lock();
    let src = src_buf.as_slice(&lock).unwrap();
    let dst = dst_buf.as_mut_slice(&lock).unwrap();
    ...
}
```

### Example: Casting buffer types

Previous versions of Neon came with a special datatype for casting the data of an ArrayBuffer, but this had incomplete handling of unaligned data and is deprecated in Neon 0.10. Crates like [bytemuck](https://crates.io/crates/bytemuck) can be used for casting buffer slices.

**Before:**

```rust
let b: Handle<JsArrayBuffer> = ...;
{
    let guard = cx.lock();
    let data = b.borrow(&guard);
    let u8_slice = data.as_slice::<u8>();
    let f32_slice = data.as_slice::<f32>();
    ...
}
```

**After:**

```rust
use bytemuck::cast_slice;

let b: Handle<JsArrayBuffer> = ...;
let u8_slice = b.as_slice(&cx);
let f32_slice: &[f32] = cast_slice(u8_slice);
```

# Minor changes

## Uncaught errors in tasks

Starting with 0.10, uncaught errors (whether Rust panics or JavaScript exceptions) in a task are now captured and reported to Node as an [`unhandledRejection ` event](https://nodejs.org/api/process.html#event-unhandledrejection). Previously, an uncaught JavaScript exception would be ignored. To handle uncaught exceptions, either wrap the body of a task with [`try_catch`](https://docs.rs/neon/0.10.0-alpha.3/neon/context/trait.Context.html#method.try_catch) or, alternatively, capture all uncaught rejections in Node with `process.on("unhandledRejection, (err) => {})`.

**Before:**

```rust
cx.task(|| "hello".to_string())
  .and_then(|mut cx, _| {
      cx.throw_error("ignore me")
  })
```

**After:**

```rust
cx.task(|| "hello".to_string())
  .and_then(|mut cx, _| {
      let _ = cx.try_catch(() {
          cx.throw_error("ignore me")
      });
      Ok(())
  })
```

## `Channel::send` returns `JoinHandle`

The [`Channel::send()`](https://docs.rs/neon/0.10.0-alpha.3/neon/event/struct.Channel.html#method.send) method now returns a [`JoinHandle`](https://docs.rs/neon/0.10.0-alpha.3/neon/event/struct.JoinHandle.html) type instead of `()`, allowing code to optionally and conveniently block on the result with [`JoinHandle::join()`](https://docs.rs/neon/0.10.0-alpha.3/neon/event/struct.JoinHandle.html#method.join). Non-blocking code should usually work with little to no change; sometimes a semicolon may be needed to explicitly ignore the `JoinHandle`.

**Before:**

```rust
fn helper<'a, C: Context<'a>>(cx: &mut C, ch: Channel) {
    ch.send(...)
}
```

**After:**

```rust
fn helper<'a, C: Context<'a>>(cx: &mut C, ch: Channel) {
    ch.send(...);
}
```

## Value types aren't cloneable

Previous versions of Neon had a safety bug in allowing types that implement the [`Value`](https://docs.rs/neon/0.10.0-alpha.3/neon/types/trait.Value.html) trait to be cloned. This has been fixed in Neon 0.10. It should be rare for code to ever need to clone a value. In most cases where this may be occurring, a [`Handle`](https://docs.rs/neon/0.10.0-alpha.3/neon/handle/struct.Handle.html) or reference (`&`) should work. For longer-lived use cases such as storing a value in a Rust static variable, use a [`Root`](https://docs.rs/neon/0.10.0-alpha.3/neon/handle/struct.Root.html).

**Before:**

```rust
fn helper<'a, C: Context<'a>>(cx: &mut C, s: JsString) -> ... {
    ...
}
```

**After:**

```rust
fn helper<'a, C: Context<'a>>(cx: &mut C, s: Handle<JsString>) -> ... {
    ...
}
```

## Context methods now all take `&mut self`

Context methods such as [`execute_scoped`](https://docs.rs/neon/0.10.0-alpha.3/neon/context/trait.Context.html#method.execute_scoped), [`compute_scoped`](https://docs.rs/neon/0.10.0-alpha.3/neon/context/trait.Context.html#method.compute_scoped), and [`lock`](https://docs.rs/neon/0.10.0-alpha.3/neon/context/trait.Context.html#method.lock) all take `&mut self` instead of the previous `&self`. This was necessary for safety and is more consistent with other `Context` methods. In normal usage, this should not require code changes.

## `Throw` is unconstructable

The [`Throw`](https://docs.rs/neon/0.10.0-alpha.3/neon/result/struct.Throw.html) type can no longer be explicitly constructed, and cannot be shared across threads. This makes it harder to accidentally mis-report a [`NeonResult`](https://docs.rs/neon/0.10.0-alpha.3/neon/result/type.NeonResult.html) value by reusing a stale `Throw` value. Existing code that uses `Throw` for application-specific use cases should use a custom struct or enum instead.
