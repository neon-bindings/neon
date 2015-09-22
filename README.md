Automating the process of building native Node modules in Rust.

# Known Limitations

* I've only gotten this working on OS X with io.js >= 3.
* It would be ideal to make Rust available as npm packages, to avoid clients of a native module having to install Rust on their system.
* There's no way to fallback on [precompiled](https://github.com/mapbox/node-pre-gyp) or [portable](http://insertafter.com/en/blog/native-node-module.html) implementations.

I would love to work with people on fixing these limitations!

# Usage

Set up your project as both a node package and a Rust project. Rust source files go in `src` as usual and Node source files go wherever you like, such as the root directory or the `lib` directory:

```
├── package.json
├── Cargo.toml
├── src/
└── lib/
```

Make sure you have io.js and Rust installed. Unless you override the default `multirust` configuration option, you need to have [multirust](https://github.com/brson/multirust) installed as well.

Unless you provide a `name` configuration option, your `Cargo.toml` file must include a package name, which will be used as the name of the native module. Your `Cargo.toml` can include any dependencies you like. You don't need to specify a `[lib]` section; this is automated.

The only thing you need in your `package.json` file is the `rust-bindings` dependency.

Building the native module is completely automated by `rust-bindings`:

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
