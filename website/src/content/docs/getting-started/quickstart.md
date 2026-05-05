---
title: Quickstart
description: Produce a runnable hello-world Neon module in just a few minutes.
---

This page gets you from zero to a Rust function called from Node.js in
about five minutes. It assumes you've already
[installed the prerequisites](/getting-started/install/). For a slower
walkthrough that explains what each piece is doing, see
[Your first Neon module](/tutorials/first-module/).

## Scaffold the project

```sh
npm init neon@latest hello-neon
cd hello-neon
npm install
```

`create-neon` asks a few questions (license, author, etc.); the
defaults are fine. When it's done you'll have a Rust crate, a
`package.json`, and an empty Node module ready to compile.

## Write a Rust function

Open `src/lib.rs`. The scaffolder dropped in a `hello` function as a
placeholder — replace it with something a little more interesting:

```rust
# assert_eq!(add(2.0, 3.0), 5.0);
#[neon::export]
fn add(a: f64, b: f64) -> f64 {
    a + b
}
```

[`#[neon::export]`](/api/neon/attr.export.html) is doing all the work.
It turns this Rust function into something Node.js can call directly:
arguments are extracted from the JavaScript call site, the return value
is converted back, and the function is registered with the module so
[`require()`](https://nodejs.org/api/modules.html#requireid)-ing it
just works.

## Build the module

```sh
npm run build
```

That compiles the Rust crate to `index.node` next to your
`package.json`. The build script is regular `cargo build` underneath —
no Neon-specific tooling required.

## Call it from Node

Create `example.mjs` next to `package.json`:

```js
import addon from "./index.node";

console.log(addon.add(2, 3));
```

Run it:

```sh
node example.mjs
```

You should see `5`. That's a Rust function executing inside Node's
process without you writing a single line of glue.

## What just happened

- [`#[neon::export]`](/api/neon/attr.export.html) registered `add` with
  Neon's module init code, generated a wrapper that pulls each argument
  out of the JavaScript call site (via
  [`TryFromJs`](/api/neon/types/extract/trait.TryFromJs.html)), and
  converted the return value back into a JS number (via
  [`TryIntoJs`](/api/neon/types/extract/trait.TryIntoJs.html)).
- `npm run build` compiled the crate as a `cdylib` and renamed the
  output to `index.node`, the file extension Node looks for in native
  addons.
- Loading `./index.node` from a JS module evaluated the Rust crate's
  initializer, which exposed every
  [`#[neon::export]`](/api/neon/attr.export.html) on the returned object.

## Where next

- [Your first Neon module](/tutorials/first-module/) — the same
  project, expanded with explanation and a few extra patterns.
- [Pass common types between Rust and JavaScript](/how-to/common-types/)
  — strings, arrays, objects, buffers, and how each one looks on both
  sides.
- [Export async functions](/how-to/async-fn/) — for when your Rust
  needs to `.await` something.
