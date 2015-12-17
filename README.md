# Rust Bindings

Automatically build and load native Rust/[Neon](https://github.com/dherman/neon) modules.

# Example

See the [neon-demo](https://github.com/dherman/neon-demo) repository for a simple but complete example of a native Rust/[Neon](https://github.com/dherman/neon) module built with `neon-bridge`.


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

You should have `neon-bridge` in your dependencies, and a `postinstall` script set to run `neon-bridge build`. This will ensure that the necessary project boilerplate (the `binding.gyp` build manifest and top-level C++ addon file) are generated before publishing.
```json
  ...
  "dependencies": {
    "neon-bridge": "0.0.10"
  },
  "scripts": {
    "postinstall": "neon-bridge build"
  }
  ...
```

If you want a debug build, change the `postinstall` command to `"neon-bridge build --debug"`.

## Building

Build your native module simply by running `npm install` from the project directory.

Clients who depend on your native module, directly or indirectly, don't have to do anything special. (However, they do have to have the required build tools installed on their machine. Hopefully we can improve this situation in the future.)

## Requiring

You can easily require your native module from JS without having to specify the build directory; `neon-bridge` figures this out for you:

```javascript
var my_native_module = require('neon-bridge')();
```

You can override defaults by passing an optional options object to the `neon-bridge` module:

| Option    | Description   | Type     | Default                                                                  |
| --------- | ------------- | -------- | ------------------------------------------------------------------------ |
| root      | project root  | string   | nearest containing directory of caller with package.json or node_modules |
| name      | library name  | string   | parse($manifest).package.name                                            |
| manifest  | manifest path | path     | $root/Cargo.toml                                                         |


# Setup

**Note: this is currently only working on OS X.**

### OS X

* [XCode](https://developer.apple.com/xcode/download/)
* Node: v4 or later. I recommend using [nvm](https://github.com/creationix/nvm#install-script):

```
% nvm install 4
```

Optional:

* [multirust](https://github.com/brson/multirust#quick-installation)

Install multirust if you want to use a different version of Rust than the system default. To use a non-default Rust version, change the `postinstall` command to `"neon-bridge build --rust <toolchain>"` where \<toolchain\> is the Rust toolchain you want multirust to use (e.g. "nightly").


# Known Limitations

* I've only gotten this working on OS X.
* Currently, downstream clients of a native Rust module have to have Rust installed on their system in order to build it.
* There's no way to fallback on [precompiled](https://github.com/mapbox/node-pre-gyp) or [portable](http://insertafter.com/en/blog/native-node-module.html) implementations.

I would love to work with people on fixing these limitations!


# License

MIT
