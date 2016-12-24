% Hacking on Neon

This document provides some information about the internals of Neon,
for people interested in contributing and who want to understand a
little more about how Neon works under the hood.


# Node

Node is the JavaScript runtime. It's built on V8.

### Resources

* [Node C/C++ Addons Docs](https://nodejs.org/api/addons.html)
* [Node ABI Table](https://nodejs.org/en/download/releases/)


# V8

V8 is Chrome's JavaScript engine. It's written in C++.

### Resources

* [V8 Getting Started with Embedding](https://github.com/v8/v8/wiki/Getting%20Started%20with%20Embedding)
* [V8 Embedder's Guide](https://github.com/v8/v8/wiki/Embedder's%20Guide)
* [V8 API Docs by Node Version](https://v8docs.nodesource.com/)


# NAN

NAN stands for Native Abstractions for Node.js. It normalizes changes in V8 and
Node core so native addons compile against different versions of Node.js.

### Resources

* [NAN API Docs](https://github.com/nodejs/nan/)


# `neon-sys` Crate

`neon-sys` is the glue that is exposing Node and V8 C++ APIs for use in Neon.

### Resources

* [`neon-sys` Crate Docs](https://docs.neon-bindings.com/neon_sys/index.html)
* [Cargo Build Script Support](http://doc.crates.io/build-script.html)
  * [`-sys` Packages](http://doc.crates.io/build-script.html#-sys-packages)


# `cslice` Crate

`cslice` is a library of slices with a stable ABI for interfacing with C.

### Resources

* [`cslice` Crate Docs](https://docs.neon-bindings.com/cslice/index.html)
