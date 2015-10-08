# Rust Bindings

An npm package that automates the process of building native Node modules in Rust.

# Example

See the [nanny-demo](https://github.com/dherman/nanny-demo) repository for a simple but complete example of a native Rust module built with `rust-bindings`.


# Usage

## Project Structure

Set up your project as both a node package and a Rust project. Rust source files go in `src` as usual and Node source files go wherever you like, such as the root directory or the `lib` directory:

```
├── package.json
├── Cargo.toml
├── src/
└── lib/
```

## package.json

You should have `rust-bindings` in your dependencies, and a `postinstall` script set to run `rust-bindings build`. This will ensure that the necessary project boilerplate (the `binding.gyp` build manifest and top-level C++ addon file) are generated before publishing.
```json
  ...
  "dependencies": {
    "rust-bindings": "0.0.5"
  },
  "scripts": {
    "postinstall": "rust-bindings build"
  }
  ...
```

If you want a debug build, change the `postinstall` command to `"rust-bindings build --debug"`.

## Building

Build your native module simply by running `npm install` from the project directory.

Clients who depend on your native module, directly or indirectly, don't have to do anything special. (However, they do have to have the required build tools installed on their machine. Hopefully we can improve this situation in the future.)

## Requiring

You can easily require your native module from JS without having to specify the build directory; `rust-bindings` figures this out for you:

```javascript
var my_native_module = require('rust-bindings')();
```

You can override defaults by passing an optional options object to the `rust-bindings` module:

| Option    | Description   | Type     | Default                                                                  |
| --------- | ------------- | -------- | ------------------------------------------------------------------------ |
| root      | project root  | string   | nearest containing directory of caller with package.json or node_modules |
| name      | library name  | string   | parse($manifest).package.name                                            |
| manifest  | manifest path | path     | $root/Cargo.toml                                                         |


# Setup

**Note: this is currently only working on OS X.**

### OS X

* [XCode](https://developer.apple.com/xcode/download/)
* Node: io.js v3 or later. I recommend using [nvm](https://github.com/creationix/nvm#install-script):

```
% nvm install iojs
```

* [multirust](https://github.com/brson/multirust#quick-installation)

*Right now multirust is a mandatory dependency because it's used to run on Rust nightly by default. Once the [fix for a jemalloc linking bug](https://github.com/rust-lang/rust/pull/27400) makes it through the trains to stable, multirust will be an optional dependency and rust-bindings will default to the system Rust compiler.*


# Known Limitations

* I've only gotten this working on OS X with io.js >= 3.
* It would be ideal to make Rust available as npm packages, to avoid clients of a native module having to install Rust on their system.
* There's no way to fallback on [precompiled](https://github.com/mapbox/node-pre-gyp) or [portable](http://insertafter.com/en/blog/native-node-module.html) implementations.

I would love to work with people on fixing these limitations!


# License

MIT
