# Neon

<img align="right" src="neon.jpg" alt="neon"/>A safe Rust abstraction layer for native Node.js modules.

Neon protects all handles to the JavaScript heap, even when they're allocated on the Rust stack, ensuring that objects are always safely tracked by the garbage collector.

# Example

A complete example can be found in the [neon-demo](https://github.com/dherman/neon-demo) repository. The demo makes use of the [rust-bindings](https://www.npmjs.com/package/rust-bindings) npm package, which makes building a Neon module as simple as adding a single line to `package.json`.

## A simple Neon function

A Neon function takes a `Call` object and produces either a handle to a value or the `Throw` constant (meaning a JS exception was thrown). The `Call` object provides access to a memory management scope, which safely manages the rooting of handles to garbage-collected JS values:

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
