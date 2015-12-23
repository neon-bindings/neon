# Neon

<img align="right" src="neon.jpg" alt="neon"/>A safe Rust abstraction layer for native Node.js modules.

Neon protects all handles to the JavaScript heap, even when they're allocated on the Rust stack, ensuring that objects are always safely tracked by the garbage collector.

# Getting started

Install [neon-cli](https://github.com/dherman/neon-cli) as a global npm package:

```
npm install -g neon-cli
```

To create a new Neon project, use `neon new`:

```
neon new my-project
```

This will ask you a few questions and then generate a project skeleton for you. Follow the instructions from there to build and run your project!

# Requirements

So far Neon is only working on OS X. You'll need [XCode](https://developer.apple.com/xcode/download/), Node v4 or later, and Rust 1.5 or later.

# A Taste...

A Neon function takes a `Call` object and produces either a handle to a value or the `Throw` constant (meaning a JS exception was thrown). The `Call` object provides access to a memory management scope, which safely manages the rooting of handles to heap objects:

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

For a more complete demonstration, try building a hello world with `neon new`, or check out the slightly bigger [word count demo](https://github.com/dherman/wc-demo).

# Help Wanted

I'm looking for collaborators! I've created a [community slack](http://neon-bridge.slack.com) and a `#neon-bridge` IRC channel on freenode. Come play :)

# Known Limitations

* I've only gotten this working on OS X.
* Currently, downstream clients of a native Rust module have to have Rust installed on their system in order to build it.
* There's no way to fallback on [precompiled](https://github.com/mapbox/node-pre-gyp) or [portable](http://insertafter.com/en/blog/native-node-module.html) implementations.

I would love to work with people on fixing these limitations!

# License

MIT
