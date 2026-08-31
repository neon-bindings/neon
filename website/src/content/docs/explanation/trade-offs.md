---
title: Neon trade-offs
description: An honest accounting of the costs that come with shipping a Neon addon — boundary overhead, type-conversion choices, build and distribution complexity, and the cases where the costs don't pay off.
status: draft
---

[*What is Neon?*](/explanation/what-is-neon/) makes the case for
using Neon. This page is the counterweight: the costs that come
with that decision. None of these are deal-breakers on their own,
but they're all real, and they all show up in production.

## The boundary has a cost

Every call into Neon — and every value that crosses back — has to
traverse the [Node-API](https://nodejs.org/api/n-api.html) boundary.
Type checks happen, a Rust execution context is set up and torn
down, and the engine has to coordinate with code it can't optimize
through. The cost is small, but it's not zero.

The practical rule: **only cross the boundary when the work warrants
it.** Picture each call as a fast, but not free, network round-trip. You'd
batch ten field fetches, not make ten round-trips for them. The same
applies here. A `#[neon::export]` function that does very
little is *slower* than the equivalent JavaScript, because the
boundary cost outweighs the work. Treating Rust as a "fast helpers"
library — JS code with many small Rust calls — is the most common
Neon mistake, and it tends to perform *worse* than pure JS.

## Choose efficient types

The shape of a value matters as much as its size. Some types cross
the boundary essentially for free; others trigger per-property or
per-element work that's invisible from JavaScript but enormous in a
hot path.

Cross cheaply:

- **Numbers and booleans.** Primitive scalars; nothing to convert.
- **[`Buffer`](https://nodejs.org/api/buffer.html#class-buffer),
  [`ArrayBuffer`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer),
  and [typed arrays](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray).**
  Neon hands you a view into the underlying memory; the bytes don't
  move. For numeric data, `Float64Array` is dramatically cheaper than
  `Array<number>`.

Cost real work:

- **Reading or building a [`JsObject`](/api/neon/types/struct.JsObject.html)
  or [`JsArray`](/api/neon/types/struct.JsArray.html) field by field.**
  Every property access is a separate boundary call. Fine for a few
  fields; expensive for big or deeply-nested structures.

For structured data, **prefer
[`Json<T>`](/api/neon/types/extract/struct.Json.html) over walking
objects and arrays by hand.** It looks like the slow option — every
value is serialized to a JSON string on one side and parsed back on
the other — but in practice that single round-trip through
[`JSON.stringify`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/stringify)
and [`JSON.parse`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/parse)
is **typically faster** than reading or building the equivalent
shape one property at a time.

## Build complexity

Adding Neon means adding a Rust toolchain to the build:

- **Every contributor needs Rust installed.** A first-time
  contributor running `npm install` now also has to set up
  [`rustup`](https://rustup.rs/). The
  [*Prerequisites*](/getting-started/install/) page covers what they
  need.
- **CI needs Rust available.** Most CI providers offer it, but it's
  another thing to configure and another thing that can break a
  release.
- **Cross-platform builds are real work.** Linux glibc versions,
  macOS arm64-vs-x86_64, Windows MSVC. Building once and shipping
  everywhere — the JavaScript norm — is no longer the default.

The standard mitigation is to publish prebuilt binaries from your CI
to npm, so end users never compile anything themselves. The
[*Publish your addon to npm*](/tutorials/publish-your-addon-to-npm/)
tutorial walks through it end to end.

## Distribution complexity

A pure-JavaScript package ships as one tarball that runs everywhere.
A WebAssembly package ships as one `.wasm` file that runs everywhere.
A Neon addon ships as a `.node` file **per platform per architecture**.
Your published package is bigger and your
release pipeline has more moving parts. Neon trades portability for
speed and OS access.

## Debugging across the language boundary

Stack traces stop at the boundary on each side. JavaScript sees an
exception "from the addon"; Rust sees a return value or a panic, but
not the JS call site that produced it. In practice:

- **Errors from Rust** surface as standard JS exceptions with a
  message and a JavaScript stack — but the stack only shows the JS
  call site, never Rust frames. The
  [*Throw and catch errors*](/how-to/errors/) how-to covers the
  recipe for adding the context you need on the way out.
- **Profilers don't see both sides at once.** Node's inspector
  profiles V8, not native code; OS-level profilers like `perf` see
  Rust frames but lose JavaScript context.
- **Logging is the workhorse.** Most teams add structured
  logging on both sides of the boundary and correlate by hand.

## Supply-chain and binary size

You now have **two dependency trees** to audit and update — npm and
Cargo — with separate workflows ([`npm audit`](https://docs.npmjs.com/cli/v10/commands/npm-audit),
[`cargo audit`](https://crates.io/crates/cargo-audit)).

Compiled artifacts also tend to be larger than the equivalent
JavaScript. A small Rust crate plus dependencies can produce a
multi-megabyte `.node` file where the JS equivalent might be a
few kilobytes.

## Is Neon worth it?

- **Direct port of existing JS code.** No — boundary overhead
  alone will eat the win.
- **A hot path V8 already optimizes well.** Probably no. Beating V8
  requires real algorithmic wins, SIMD, or genuinely heavy work.
- **Many small Rust calls in a tight loop.** No, or batch the work
  so each call does substantially more than the boundary cost.
- **A library that only exists in Rust.** Yes — Neon's clearest win.
- **CPU-bound work that blocks the event loop.** Yes — see
  [*Move work off the main thread*](/tutorials/move-work-off-the-main-thread/).
- **Memory-heavy data manipulation.**  Yes, with typed arrays to avoid the copy.

The pattern is the same in every case: cross the boundary
deliberately, do meaningful work on the Rust side, and pick the
right shape for the data — typed arrays for raw bytes, `Json<T>`
for structured values, hand-rolled object access only when you
need it.

## Where to go next

- **You've decided Neon is right.** Start with the
  [quickstart](/getting-started/quickstart/) and the
  [first-addon tutorial](/tutorials/first-addon/).
- **You're still on the fence.** 1Password's writeup of
  [building 1Password for Linux with Neon](https://1password.com/blog/welcoming-linux-to-the-1password-family)
  is a good real-world look at what shipping a Neon-backed product
  involves — Rust core, React frontend, deep OS integration.
