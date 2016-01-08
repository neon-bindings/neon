% The Neon Guide

Neon is a Rust bridge to the [Node.js](http://nodejs.org) platform: an
API (and a set of tools) for implementing binary Node modules in
Rust. Neon makes it possible to write **fast**, **portable**, and
**parallel** modules with [Rust](http://www.rust-lang.org): a language
that makes systems programming safe and fun!

# Why Rust?

The reasons to use Rust for implementing Node addons are simple: it's
safe, it's modern, **and it's fun!**

In general, binary addons can be useful for:

* **performance:** While V8 is a high-quality modern JavaScript
engine, it is generally much easier to get higher performance, and
with greater predictability, from Rust than from high-level dynamic
languages like JavaScript.
* **parallelism:**
* **bindings:** 

# Getting Started

# Hello Node!

```rust
fn hello_node(call: Call) -> JsResult<JsString> {
    JsString::new_or_throw(call.scope, "hello node!")
}
```
