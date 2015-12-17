# Neon

<img align="right" src="neon.png" alt="neon"/>A Rust library providing a safe abstraction layer around the JavaScript engine for native Node.js modules.

Neon protects all handles to the JavaScript heap, even when they're allocated on the Rust stack, ensuring that objects are always safely tracked by the garbage collector.

# Example

A complete example can be found in the [neon-demo](https://github.com/dherman/neon-demo) repository. The demo makes use of the [rust-bindings](https://www.npmjs.com/package/rust-bindings) npm package, which makes building a neon module as simple as adding a single line to `package.json`.

## A Node function in Rust

A JS function is represented in Rust as a function that takes a `Call` object and produces either a JS value or the `Throw` constant, indicating that an exception has been thrown. The `Call` object provides access to a memory management scope, which safely manages the rooting of handles to garbage-collected JS values:

```rust
fn make_an_array(call: Call) -> JS<Array> {
    let scope = call.scope; // the current scope for rooting handles
    let array: Handle<Array> = Array::new(scope, 3);
    try!(array.set(0, Integer::new(scope, 9000)));
    try!(array.set(1, Object::new(scope)));
    try!(array.set(2, Number::new(scope, 3.14159)));
    Ok(array)
}
```

# License

MIT
