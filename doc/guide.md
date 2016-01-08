% The Neon Guide

Neon is a Rust bridge to the [Node.js](http://nodejs.org) platform: an
API (and a set of tools) for implementing binary Node modules in
Rust. Neon makes it possible to write **fast**, **portable**, and
**parallel** modules with [Rust](http://www.rust-lang.org): a language
that makes systems programming safe and fun!

# Why Rust?

The reasons to use Rust for implementing Node addons are simple: it's
safe, it's modern, **and it's fun!**

Binary addons written with Neon can be useful for:

* **performance:** While V8 is a high-quality modern JavaScript
engine, it is generally much easier to get higher performance, and
with greater predictability, from Rust than from high-level dynamic
languages like JavaScript.

* **parallelism:** Neon provides access to Rust's [_fearless
concurrency and
parallelism_](http://blog.rust-lang.org/2015/04/10/Fearless-Concurrency.html),
which allows you to take advantage of modern multicore hardware when
operating on certain types of JavaScript data.

* **bindings:** Neon is a great way to connect the NPM and Cargo
ecosystems to make packages built for one available to the other.

# Getting Started

Getting started with Neon is easy. You'll need a recent Node
installation (4.x or later), a recent Rust installation (1.5 or
later), and for OS X developers, you'll also need
[XCode](https://developer.apple.com/xcode/).

Install the Neon command-line tool with:

```
$ npm install -g neon-cli
```

That's it—you're ready to create your first Neon project!

# Creating a Project

TODO.

```
neon new hello-node
```

# Hello Node!

TODO.

```rust
fn hello_node(call: Call) -> JsResult<JsString> {
    JsString::new_or_throw(call.scope, "hello node!")
}
```

# Getting Acquainted With Rust

TODO.

# JavaScript Data

TODO.

# Memory Management

TODO.

# Control Flow

TODO.

# Where Do I Go From Here?

Now that you're up and running, the next thing you'll want to do is
start playing with the Neon API. The [API Documentation](./neon) is a
good place to poke around and see what kinds of things you can do with
JavaScript data in Neon.

If you get stuck, **don't suffer in silence!** Come ask your questions
on our [community Slack](http://rustbridge.slack.com)—just grab an
[automatic invite](http://rustbridge-community-slackin.herokuapp.com/)
and then join us in `#neon`!
