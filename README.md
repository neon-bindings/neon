# Neon

<img align="right" src="neon.jpg" alt="neon"/>

[![Build Status](https://travis-ci.org/neon-bindings/neon.svg?branch=master)](https://travis-ci.org/neon-bindings/neon)
[![](http://meritbadge.herokuapp.com/neon)](https://crates.io/crates/neon)

A safe Rust abstraction layer for native Node.js modules.

Neon protects all handles to the JavaScript heap, even when they're allocated on the Rust stack, ensuring that objects are always safely tracked by the garbage collector.

# Getting started

Install [neon-cli](https://github.com/neon-bindings/neon-cli) as a global npm package:

```
npm install -g neon-cli
```

To create a new Neon project, use `neon new`:

```
neon new my-project
```

This will ask you a few questions and then generate a project skeleton for you. Follow the instructions from there to build and run your project!

# Requirements

### Operating Systems

| Linux  | macOS | Windows |
| ------ | ----- | ------- |
| x      | x     | soon    |

For macOS, you'll need:

* OS X 10.7 or later;
* [XCode](https://developer.apple.com/xcode/download/).

Windows support is on the way. Follow [#122](https://github.com/neon-bindings/neon/pull/122#issuecomment-268957333) to track the progress.

### Rust and Node

|              | Node 4 | Node 6 | Node 7 |
| ------------ | ------ | ------ | ------ |
| Rust stable  | x      | x      | x      |
| Rust beta    | x      | x      | x      |
| Rust nightly |        |        |        |

Support for Rust stable and beta are expected. We do run builds against nightly, but allow them to fail.

Support for [LTS versions of Node](https://github.com/nodejs/LTS#lts-schedule) and current are expected. If you're using a differnt version of Node and believe it should be supported, let us know.

* [Download Node](http://nodejs.org)
* [Download Rust](http://rust-lang.org)

# A Taste...

A Neon function takes a `Call` object and produces a Rust `Result` that's either a JS value or the `Throw` constant (meaning a JS exception was thrown). The `Call` object provides access to a memory management scope, which safely manages the rooting of handles to heap objects:

```rust
fn make_an_array(call: Call) -> JsResult<JsArray> {
    let scope = call.scope; // the current scope for rooting handles
    let array: Handle<JsArray> = JsArray::new(scope, 3);
    try!(array.set(0, JsInteger::new(scope, 9000)));
    try!(array.set(1, JsObject::new(scope)));
    try!(array.set(2, JsNumber::new(scope, 3.14159)));
    Ok(array)
}
```

For a more complete demonstration, try building a hello world with `neon new`, or check out the slightly bigger [word count demo](https://github.com/dherman/wc-demo).

# Get Involved

The Neon community is just getting started and there's tons of fun to be had. Come play! :)

The [Rust Bindings community Slack](https://rust-bindings.slack.com) is open to all; use [the Slackin app](https://rust-bindings-slackin.herokuapp.com) to receive an invitation.

# License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
