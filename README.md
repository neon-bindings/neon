# Neon-Bridge

Automatically build and load native Rust/[Neon](https://github.com/dherman/neon) modules.

![Screencast](screencast.gif)

# Getting started

Install `neon-bridge` as a global package:

```
npm install -g neon-bridge
```

To create a new Neon project, use `neon-bridge new`:

```
neon-bridge new my-project
```

This will ask you a bunch of questions, and then generate a project skeleton for you. Follow the instructions from there to build and run your project!

# Requirements

So far `neon-bridge` is only working on OS X. You'll need [XCode](https://developer.apple.com/xcode/download/), Node v4 or later, and Rust 1.5 or later.

# Detailed Docs

## API

```javascript
var my_native_module = require('neon-bridge')();
```

You can override defaults by passing an optional options object to the `neon-bridge` module:

| Option    | Description   | Type     | Default                                                                  |
| --------- | ------------- | -------- | ------------------------------------------------------------------------ |
| root      | project root  | string   | nearest containing directory of caller with package.json or node_modules |
| name      | library name  | string   | parse($manifest).package.name                                            |
| manifest  | manifest path | path     | $root/Cargo.toml                                                         |

## CLI

### neon-bridge new

Creates a new Neon project skeleton.

```
neon-bridge new name
```

The `name` is the project name and the name of the subdirectory of the current working directory that will be created.

### neon-bridge build

Builds a Neon project. This command should be part of the `postinstall` script in your `package.json`, which is automatically set up by `neon-bridge new`.

```
neon-bridge build [--rust toolchain] [--debug]
```

* --rust: Use this to specify that [multirust](https://github.com/brson/multirust) should be used instead of the system default Rust installation. The `toolchain` parameter is passed to multirust as the Rust toolchain to use for all build commands.
* --debug: Use this to create a debug build.

# Known Limitations

* I've only gotten this working on OS X.
* Currently, downstream clients of a native Rust module have to have Rust installed on their system in order to build it.
* There's no way to fallback on [precompiled](https://github.com/mapbox/node-pre-gyp) or [portable](http://insertafter.com/en/blog/native-node-module.html) implementations.

I would love to work with people on fixing these limitations!


# License

MIT
