---
title: Move work off the main thread
description: Build a BLAKE3 hashing addon that runs on Node's worker pool, keeping the JavaScript event loop responsive while CPU-bound work runs.
---

Node.js runs JavaScript on a single thread. When that thread is busy
running synchronous code — your code, or a synchronous addon — *nothing
else happens*. Timers don't fire, incoming HTTP requests pile up,
latency spikes. That's why
[`crypto.pbkdf2`](https://nodejs.org/api/crypto.html#cryptopbkdf2password-salt-iterations-keylen-digest-callback),
[`fs.readFile`](https://nodejs.org/api/fs.html#fsreadfilepath-options-callback),
and friends are async by default: under the hood, Node hands the
expensive work to a
[small pool of worker threads](https://nodejs.org/en/learn/asynchronous-work/dont-block-the-event-loop)
and resolves a callback or `Promise` when they're done.

When you ship a Neon addon, you decide whether your function runs on
the JavaScript main thread (fast for cheap calls) or on Node's worker
pool (essential for blocking I/O or anything CPU-heavy).

This tutorial walks through
building a [BLAKE3](https://github.com/BLAKE3-team/BLAKE3) hashing
addon — a modern, very fast cryptographic hash that Node *doesn't*
ship in core — and shows the difference one keyword can make.

## What we're building

By the end you'll have an addon with two exports:

```js
addon.hashSync(buffer)            // => "af13…"  (sync, blocks event loop)
await addon.hash(buffer)          // => "af13…"  (Promise, runs on worker pool)
```

This mirrors the
[`fs.readFile` / `fs.readFileSync`](https://nodejs.org/api/fs.html#fsreadfilepath-options-callback)
pattern: same work, two surfaces, caller picks
based on context.

## A first BLAKE3 hash

Add `blake3` to your `Cargo.toml`:

```toml
[dependencies]
neon = "1"
blake3 = "1"
```

Then in `src/lib.rs`:

```rust
# assert_eq!(
#   hash_sync(b"".to_vec()),
#   "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
# );
# assert_eq!(
#   hash_sync(b"hello".to_vec()),
#   "ea8f163db38682925e4491c5e58d4bb3506ef8c14eb78a86e908c5624a67200f"
# );
#[neon::export]
fn hash_sync(input: Vec<u8>) -> String {
    blake3::hash(&input).to_hex().to_string()
}
```

Two things to call out:

- The `Vec<u8>` parameter accepts JavaScript
  [typed arrays](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray)
  via [`JsTypedArray<u8>`](/api/neon/types/struct.JsTypedArray.html),
  which means
  [`Buffer`](https://nodejs.org/api/buffer.html#class-buffer) and
  [`Uint8Array`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array)
  both work without conversion on the JS side.
- The return value is the BLAKE3 hash as a hex `String`.

Build and try it from JavaScript:

```js
const addon = require("./index.node");

const buf = Buffer.from("hello");
console.log(addon.hashSync(buf));
// => "ea8f163db38682925e4491c5e58d4bb3506ef8c14eb78a86e908c5624a67200f"
```

## Watching the event loop freeze

Hashing a five-byte buffer is fast. Hashing a gigabyte is not. Save
this as `freeze.cjs` next to your built addon:

```js
const addon = require("./index.node");

const huge = Buffer.alloc(2 ** 30, 0x41);

setInterval(() => console.log(`tick ${Date.now()}`), 100);

setTimeout(() => {
  console.log("hashing…");
  console.log(addon.hashSync(huge));
  console.log("done");
}, 500);
```

Run it with `node freeze.cjs`. You'll see four or five `tick` lines
print on schedule, then `hashing…`, then *silence* for a couple of
seconds while BLAKE3 chews through the buffer, then the hash, then
`done`, and finally the `tick`s resume. Those missing ticks are
everything else your process *should* have been doing — handling HTTP
requests, running timers, etc.

That gap is the JavaScript event loop frozen, and it's why we have
[`#[neon::export(task)]`](/api/neon/attr.export.html).

## One keyword on the worker pool

Now add a second export — same body, different attribute:

```rust
# assert_eq!(
#   hash(b"".to_vec()),
#   "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
# );
# assert_eq!(hash(b"hello".to_vec()), hash_sync(b"hello".to_vec()));
# fn hash_sync(input: Vec<u8>) -> String {
#   blake3::hash(&input).to_hex().to_string()
# }
#[neon::export(task)]
fn hash(input: Vec<u8>) -> String {
    hash_sync(input)
}
```

`#[neon::export(task)]` doesn't change what the function *does* — it
changes where it runs and how the result reaches JavaScript. The Rust
body is still a plain synchronous call to `hash_sync`, and from Rust
you can still call `hash(buf)` directly. But on the JavaScript side,
calling `addon.hash(buf)`:

1. Returns immediately with a
   [`Promise`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise),
2. Schedules the actual hashing onto Node's worker pool — the *same*
   pool [`fs.readFile`](https://nodejs.org/api/fs.html#fsreadfilepath-options-callback)
   and [`crypto.pbkdf2`](https://nodejs.org/api/crypto.html#cryptopbkdf2password-salt-iterations-keylen-digest-callback)
   use,
3. Resolves the `Promise` with the return value when the worker
   finishes.

The JavaScript main thread is free while the hash runs (argument
extraction — including the buffer copy into `Vec<u8>` — still happens
on the main thread before the task is scheduled).

```js
const addon = require("./index.node");

const huge = Buffer.alloc(2 ** 30, 0x41);

setInterval(() => console.log(`tick ${Date.now()}`), 100);

setTimeout(async () => {
  console.log("hashing…");
  console.log(await addon.hash(huge));
  console.log("done");
}, 500);
```

Now the `tick`s keep firing on schedule the entire time the hash
runs. That's the entire payoff of `task`.

## Two surfaces, one implementation

You haven't replaced anything — `hashSync` and `hash` both exist on
`addon`, and the `hash` (task) version simply calls the `hash_sync`
version on the worker pool. The caller picks:

```js
addon.hashSync(buf)            // immediate string, blocks the event loop
await addon.hash(buf)          // Promise<string>, doesn't block

await Promise.all(buffers.map(addon.hash))   // hash N buffers in parallel
```

That last line is the real reason to ship both. Every call to
`addon.hash` runs on a *different* worker thread (up to the pool's
size), so an array of buffers hashes concurrently — something you
cannot do from a single-threaded synchronous API.

The pool defaults to four threads. Tune it with the
[`UV_THREADPOOL_SIZE`](https://nodejs.org/api/cli.html#uv-threadpool-sizesize)
environment variable if you're hashing a lot of buffers at once.

## When to use `task` vs `async fn`

`task` is the right tool when your work is **CPU-bound**: hashing,
compression, image processing, parsing. The function body is plain
synchronous Rust; Neon takes care of running it elsewhere.

If your work is **I/O-bound** — talking to a database, calling an
HTTP service, reading a file — you want an `async fn` instead, paired
with an async runtime like
[tokio](https://docs.rs/tokio/latest/tokio/). That's the topic of the
next tutorial,
[Build a database addon](/tutorials/build-a-database-addon/).

A rough decision tree:

- If your function would `.await` something inside it → `async fn`.
- If your function is just a long-running computation that doesn't
  `.await` → `#[neon::export(task)]`.
- If it's fast (sub-millisecond) → leave it on the main thread; the
  worker-pool overhead isn't worth it.

## Where next

- [Build a database addon](/tutorials/build-a-database-addon/) — for
  I/O-bound work that wants to `.await`, paired with `#[neon::class]`.
- [Run blocking work on the libuv pool](/how-to/blocking-libuv/) —
  recipe form of this tutorial, including how to use
  `Channel::send` and `JoinHandle::join` directly when `(task)`
  isn't enough.
- [Pass common types between Rust and JavaScript](/how-to/common-types/) —
  how [`Buffer`](https://nodejs.org/api/buffer.html#class-buffer),
  [`Uint8Array`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array),
  and other typed arrays cross the boundary.
