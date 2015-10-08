# Nanny

A Rust library providing a safe API around native abstractions for Node.

# Example

A complete example can be found in the [nanny-demo](https://github.com/dherman/nanny-demo) repository. The demo makes use of the [rust-bindings](https://www.npmjs.com/package/rust-bindings) npm package to completely automate the process of building and requiring a Rust module in Node.

## A Node function in Rust

A JS function is represented in Rust as an extern function that takes a reference to a `Call` object. The `Call` object allows you to create memory management scopes, which safely manage the rooting of handles to garbage-collected JS values:

```rust
#[no_mangle]
extern fn make_an_array(call: &Call) {
    let realm = call.realm(); // current VM execution context
    realm.scoped(|scope| {    // create a scope for rooting handles
        let mut array: Handle<Array> = scope.array(3);
        array.set(0, scope.integer(9000));
        array.set(1, scope.object());
        array.set(2, scope.number(3.14159));
        call.activation().set_return(array);
    });
}
```

# License

MIT

