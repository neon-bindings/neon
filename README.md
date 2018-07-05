# Neon

<img align="right" src="neon.jpg" alt="neon"/>

[![Travis Build Status](https://travis-ci.org/neon-bindings/neon.svg?branch=master)](https://travis-ci.org/neon-bindings/neon)
[![Appveyor Build Status](https://ci.appveyor.com/api/projects/status/github/neon-bindings/neon?branch=master&svg=true)](https://ci.appveyor.com/project/dherman/neon)
[![](http://meritbadge.herokuapp.com/neon)](https://crates.io/crates/neon)
[![npm](https://img.shields.io/npm/v/neon-cli.svg)](https://www.npmjs.com/package/neon-cli)

Rust bindings for writing safe and fast native Node.js modules.

# Getting started

Once you have the [platform dependencies](https://guides.neon-bindings.com/getting-started/) installed, getting started is as simple as:

```
$ npm install -g neon-cli
$ neon new my-project
```

See the [Getting Started guide](https://guides.neon-bindings.com/getting-started/) for details.

# Platform Support

### Operating Systems

| Linux  | macOS | Windows |
| ------ | ----- | ------- |
| ✓      | ✓     | ✓       |

### Node.js

| Node 6 | Node 8 | Node 10 |
| ------ | ------ | ------- |
| ✓      | ✓      | ✓       |

Support for [LTS versions of Node](https://github.com/nodejs/LTS#lts-schedule) and current are expected. If you're using a different version of Node and believe it should be supported, let us know.

### Rust

Neon supports Rust stable version 1.18 and higher. We test on the latest stable, beta, and nightly versions of Rust.

# A Taste...

```rust
fn make_an_array(mut cx: FunctionContext) -> JsResult<JsArray> {
    // Create some values:
    let n = cx.number(9000);
    let s = cx.string("hello");
    let b = cx.boolean(true);

    // Create a new array:
    let array: Handle<JsArray> = cx.empty_array()?;

    // Push the values into the array:
    array.set(&mut cx, 0, n)?;
    array.set(&mut cx, 1, s)?;
    array.set(&mut cx, 2, b)?;

    // Return the array:
    Ok(array)
}

register_module!(mut cx, {
    cx.export_function("makeAnArray", make_an_array)?;
})
```

To learn more, check out the [Neon guides](https://guides.neon-bindings.com).

# Get Involved

The Neon community is just getting started and there's tons of fun to be had. Come play! :)

The [Rust Bindings community Slack](https://rust-bindings.slack.com) is open to all; use [the Slackin app](https://rust-bindings-slackin.herokuapp.com) to receive an invitation.

# License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
