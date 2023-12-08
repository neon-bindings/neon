# <img src="doc/neon.png" alt="neon" width="100%" />

[![Cargo](https://img.shields.io/crates/v/neon.svg)](https://crates.io/crates/neon)
[![npm](https://img.shields.io/npm/v/neon-cli.svg)](https://www.npmjs.com/package/neon-cli)
[![Linux Build Status](https://github.com/neon-bindings/neon/workflows/Test%20on%20Linux/badge.svg)](https://github.com/neon-bindings/neon/actions?query=workflow%3A%22Test+on+Linux%22)
[![macOS Build Status](https://github.com/neon-bindings/neon/workflows/Test%20on%20MacOS/badge.svg)](https://github.com/neon-bindings/neon/actions?query=workflow%3A%22Test+on+MacOS%22)
[![Windows Build Status](https://github.com/neon-bindings/neon/workflows/Test%20on%20Windows/badge.svg)](https://github.com/neon-bindings/neon/actions?query=workflow%3A%22Test+on+Windows%22)

Rust bindings for writing safe and fast native Node.js modules.

## Getting started

Once you have the [platform dependencies](https://neon-bindings.com/docs/quick-start) installed, getting started is as
simple as:

```
$ npm init neon my-project
```

Then see the [Hello World guide](https://neon-bindings.com/docs/hello-world/) for writing your first Hello World in
Neon!

## Docs

See our [Neon fundamentals docs](https://neon-bindings.com/docs/intro) and
our [API docs](https://docs.rs/neon/latest/neon).

## Neon 1.0.0 Migration Guide

The latest version of Neon, 1.0.0, includes several breaking changes in order to fix unsoundness, improve consistency, and add features.

**Read the new [migration guide](doc/MIGRATION_GUIDE_1.0.0.md)** to learn how to port your 
Neon projects to 1.0.0!

## Platform Support

### Operating Systems

| Linux  | macOS | Windows |
| ------ | ----- | ------- |
| ✓      | ✓     | ✓       |

### Node.js

| Node 18 | Node 20 | Node 21 |
|---------|---------|---------|
| ✓       | ✓       | ✓       |

Support for [LTS versions of Node](https://github.com/nodejs/LTS#release-schedule) and current are expected. If you're
using a different version of Node and believe it should be supported, let us know.

### Bun (experimental)

[Bun](https://bun.sh/) is an alternate JavaScript runtime that targets Node compatibility. In many cases Neon modules will work in bun; however, at the time of this writing, some Node-API functions are [not implemented](https://github.com/Jarred-Sumner/bun/issues/158).

### Rust

Neon supports Rust stable version 1.18 and higher. We test on the latest stable, beta, and nightly versions of Rust.

## A Taste...

```rust
fn make_an_array(mut cx: FunctionContext) -> JsResult<JsArray> {
    // Create some values:
    let n = cx.number(9000);
    let s = cx.string("hello");
    let b = cx.boolean(true);

    // Create a new array:
    let array: Handle<JsArray> = cx.empty_array();

    // Push the values into the array:
    array.set(&mut cx, 0, n)?;
    array.set(&mut cx, 1, s)?;
    array.set(&mut cx, 2, b)?;

    // Return the array:
    Ok(array)
}

register_module!(mut cx, {
    cx.export_function("makeAnArray", make_an_array)
})
```

For more examples, see our [examples repo](https://github.com/neon-bindings/examples).

## Get Involved

The Neon community is just getting started and there's tons of fun to be had. Come play! :)

The [Neon Community Slack](https://rust-bindings.slack.com) is open to all;
use [this invite link](https://join.slack.com/t/rust-bindings/shared_invite/zt-1pl5s83xe-ZvXyrzL8vuUmijU~7yiEcg) to receive an invitation.

### Testing Neon

The Neon project is both an [NPM workspace](https://docs.npmjs.com/cli/v8/using-npm/workspaces) and
a [Cargo workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html). The full suite of tests may be executed
by installing and testing the NPM workspace.

```sh
npm install
npm test
```

Individual JavaScript packages may be tested with an `npm` workspace command:

```
npm --workspace=create-neon test
```

Individual Rust crates may be tested with a `cargo` workspace command:

```
cargo test -p neon-build
```

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
