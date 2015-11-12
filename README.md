# Nanny

<img align="right" src="silhouette.png" alt="silhouette"/>A Rust library providing a safe API around native abstractions for Node.

Nanny collaborates with the V8 embedding API to safely monitor stack-allocated handles to garbage-collected objects. This ensures that all objects rooted in the stack are safely tracked by the garbage collector.

*A good nanny keeps the nursery safe.*

# Example

A complete example can be found in the [nanny-demo](https://github.com/dherman/nanny-demo) repository. The demo makes use of the [rust-bindings](https://www.npmjs.com/package/rust-bindings) npm package to completely automate the process of building and requiring a Rust module in Node.

## A Node function in Rust

A JS function is represented in Rust as an extern function that takes a `Call` object. The `Call` object provides access to a memory management scope, which safely manages the rooting of handles to garbage-collected JS values:

```rust
fn make_an_array(call: Call) -> JS<Array> {
    let scope = call.scope; // the current scope for rooting handles
    let mut array: Handle<Array> = Array::new(scope, 3);
    array.set(0, Integer::new(scope, 9000));
    array.set(1, Object::new(scope));
    array.set(2, Number::new(scope, 3.14159));
    Ok(array)
}
```

# License

MIT

