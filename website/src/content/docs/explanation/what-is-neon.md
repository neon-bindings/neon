---
title: What is Neon?
description: A high-level introduction to Neon and its relationship to Node-API, plus how it compares to writing the same code in JavaScript or WebAssembly.
---

:::note[Draft]
This page is a draft pending review.
:::

Neon lets you write [Node.js](https://nodejs.org/) addons in
[Rust](https://www.rust-lang.org/). You write a Rust function, mark
it with [`#[neon::export]`](/api/neon/attr.export.html), build a
`.node` file, and load it from JavaScript with
[`require()`](https://nodejs.org/api/modules.html#requireid). From
the outside, the result looks and feels like any other npm package.

```rust
use neon::types::extract::Error;

#[neon::export]
fn slugify(input: String) -> Result<String, Error> {
    Ok(input.to_lowercase().replace(' ', "-"))
}
```

```js
const { slugify } = require("./index.node");
console.log(slugify("Hello world")); // => "hello-world"
```

That tiny example covers most of what makes Neon distinctive: a
plain Rust function, plain Rust types in and out, and minimal
glue between the two.

## What problem does Neon solve?

Node.js is a great runtime for I/O-heavy code, full stack development, and
much more. But, you need to go beyond JavaScript to:

- run code that's CPU-bound and would freeze the event loop
- reuse a Rust crate that has no JavaScript equivalent (a SIMD-accelerated
  parser, a hardware driver, a battle-tested cryptographic
  primitive)
- match the performance of a native binary without rewriting the
  rest of your application

Options include:

1. Write the addon in C or C++ against the Node-API headers. Fast,
   but risks memory-safety bugs and doesn't benefit from the Rust ecosystem.
2. Compile to [WebAssembly](https://webassembly.org/) and load it
   from Node. Safe and portable, but constrained: WASM has its own
   sandbox, its own memory, and its own rules about what it can
   touch.
3. Shell out to a native binary over a pipe or socket. Always
   possible, but has a high overhead and has its own risks.

Neon is a fourth option: the same `.node` file Node would load if
you wrote it in C, but produced from Rust code that the compiler has
already proven memory-safe. You get the same runtime integration as
a C addon — direct access to JavaScript values, callbacks, classes,
buffers — and the same correctness guarantees as any other Rust code.

## What you get

A Neon addon is a regular Node-API addon as far as Node is
concerned. That means:

- **Native speed.** The Rust code runs as compiled machine code in
  the same process as your JavaScript. No IPC, no separate runtime to manage.
- **Direct interop.** Rust functions accept JavaScript values
  directly. Strings, numbers, arrays, buffers, even classes — they
  cross the boundary as themselves, not as JSON or some other
  encoded form. See the [type hierarchy](/explanation/type-hierarchy/)
  for the shape of the JS-value vocabulary.
- **Promises, threads, and async.** Long-running work goes onto a
  worker thread or a [Tokio](https://docs.rs/tokio) runtime, and the
  caller gets back a
  [`Promise`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise).
  See the [tutorial on async work](/tutorials/build-a-database-addon/)
  for an end-to-end example.
- **Memory safety.** Rust guarantees still apply. Use-after-free,
  double-free, data races on shared mutable state — Neon doesn't ask
  you to opt out of any of it. The boundary between "your Rust
  code" and "JavaScript" is the only place you have to think about
  unfamiliar rules, and even there Neon's API is designed so that
  the compiler catches the common mistakes. The
  [page on lifetimes](/explanation/lifetimes/) walks through how.
- **Idiomatic Rust.** Errors are
  [`Result`](https://doc.rust-lang.org/std/result/). Optional
  values are [`Option`](https://doc.rust-lang.org/std/option/).
  Async functions are `async fn`. You don't write a different style
  of Rust to use Neon; you write the same Rust you'd write
  anywhere else.

## How it relates to Node-API

Neon is built on
[Node-API](https://nodejs.org/api/n-api.html) (sometimes called
N-API), the stable C interface that Node exposes for native addons.
Node-API is what makes a `.node` file work across Node versions
without recompilation, and what lets the same addon work on Node,
[Bun](https://bun.sh/), and [Electron](https://www.electronjs.org/).

Neon wraps that C interface in a safe Rust API. You don't see
Node-API directly unless you go looking for it, but it's the reason
your addon doesn't break when Node ships a new major version. The
[supported platforms reference](/reference/supported-platforms/)
lists the Node-API levels Neon targets and the platforms each one
covers.

## When Neon vs. JavaScript

If pure JavaScript is fast enough for your problem, write JavaScript.
The runtime is mature, the ecosystem is enormous, and you don't have
to think about a build toolchain.

Neon earns its keep when JavaScript hits a wall:

- **CPU-bound work that blocks the event loop.** Hashing every file
  in a directory, decoding video frames, running a parser over a
  multi-megabyte input. JavaScript can do these things, but it
  pauses every other request on the same process while it does. The
  [*Move work off the main thread* tutorial](/tutorials/move-work-off-the-main-thread/)
  walks through the canonical fix.
- **Reusing an existing Rust library.** If the canonical
  implementation of what you need is a Rust crate, Neon is the
  shortest path to using it from Node. No glue, no shelling out, no
  porting.
- **Memory-bound code.** Large buffers, zero-copy parsing, in-place
  transformations — the kinds of patterns where you'd reach for a
  `Buffer` or a typed array in JavaScript and immediately bump into
  garbage-collection pauses.

For everything else — orchestrating HTTP calls, transforming JSON,
gluing services together — JavaScript is the right tool.

## When Neon vs. WebAssembly

WebAssembly is the other way to run Rust from Node. The choice
isn't always obvious, and the two technologies coexist happily in
the same project.

Reach for **WebAssembly** when:

- You need to run the same code in the browser *and* in Node.
  WebAssembly is a portable target; a Neon addon is not.
- You want the strong sandbox WebAssembly provides. WASM modules
  can only touch the memory you hand them, which is a feature when
  you're loading untrusted code.
- You're shipping pure computation with a small, well-typed
  surface area — image filters, hashing, parsing — and the cost of
  copying inputs and outputs across the WASM boundary is acceptable.

Reach for **Neon** when:

- You need direct access to Node and the operating system: file
  descriptors, raw network sockets, threads, environment variables,
  `Buffer`s the rest of your JavaScript already holds.
- You want to call existing Rust crates that depend on the standard
  library, on threading, on syscalls, or on platform-specific
  APIs — many of which don't compile to WASM unmodified.
- You have CPU bound work that needs real threading and not just concurrency.

A reasonable rule of thumb: if it could run in a browser tab, lean
WASM; if it has to run in a server process and talk to the rest of
Node, lean Neon.

## Where to go next

- **You want to ship something.** Read the
  [quickstart](/getting-started/quickstart/), then work through
  [*Your first Neon addon*](/tutorials/first-addon/).
- **You're still evaluating.** Read
  [*Neon trade-offs*](/explanation/trade-offs/) for an honest look
  at the costs that come with the upsides described above.
- **You want the mental model first.** Skip ahead to the
  [type hierarchy](/explanation/type-hierarchy/) and
  [lifetimes](/explanation/lifetimes/) pages — they explain how
  Neon's Rust API is shaped and why.
