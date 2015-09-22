Automating the process of building native Node modules in Rust.

# Known Limitations

* I've only gotten this working on OS X with io.js >= 3.
* It would be ideal to make Rust available as npm packages, to avoid clients of a native module having to install Rust on their system.
* There's no way to fallback on [precompiled](https://github.com/mapbox/node-pre-gyp) or [portable](http://insertafter.com/en/blog/native-node-module.html) implementations.

I would love to work with people on fixing these limitations!

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

You should have `rust-bindings` in your dependencies, and a `prepublish` script set to run `rust-bindings generate`. This will ensure that the necessary project boilerplate (the `binding.gyp` build manifest and top-level C++ addon file) are generated before publishing.
```json
  ...
  "dependencies": {
    "rust-bindings": "0.0.4"
  },
  "scripts": {
    "prepublish": "rust-bindings generate"
  }
  ...
```

## Building

To work on your native module, you currently have to run `npm install` twice:

```
% npm install
% npm install
```

Clients of your native module don't have to do anything special.

## Requiring

You can easily require your native module from JS without having to specify the build directory; `rust-bindings` figures this out for you:

```javascript
var my_native_module = require('rust-bindings')();
```

# Configuration

currently working:

| Option    | Type                               | Default                                                                  |
| --------- | ---------------------------------- | ------------------------------------------------------------------------ |
| root      | string                             | nearest containing directory of caller with package.json or node_modules |
| name      | string                             | parse($manifest).package.name                                            |

not currently working:

| Option    | Type                               | Default                                                                  |
| --------- | ---------------------------------- | ------------------------------------------------------------------------ |
| mode      | 'debug' or 'release'               | 'release'                                                                |
| manifest  | path                               | $root/Cargo.toml                                                         |
| multirust | 'nightly' or 'stable' or undefined | 'nightly' (eventually will switch to undefined)                          |


# License

MIT
