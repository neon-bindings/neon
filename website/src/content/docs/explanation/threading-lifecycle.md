---
title: Threading and value lifecycle
description: How Root, Channel, and Deferred coordinate JavaScript values across threads — the choreography behind async exports and the Move work off the main thread tutorial.
status: draft
---

JavaScript runs on a single thread. Neon code is Rust, so it can spawn
background threads whenever you want — but the moment a Rust thread
leaves the JS main thread, it can't touch JS values directly. A
[`Handle<'cx, T>`](/api/neon/handle/struct.Handle.html) is tied to
the [`Cx<'cx>`](/api/neon/context/struct.Cx.html) it came from (see
[Context lifetimes](/explanation/lifetimes/)), and there's no `Cx`
on a worker thread.

To do useful work off the main thread and then talk to JS again, you
need three things:

1. <a id="problem-keep-alive"></a>A way to **keep a JS value alive**
   that isn't tied to the original `'cx` scope.
2. <a id="problem-back-to-main"></a>A way to **schedule code back on
   the main thread** that *can* touch JS values, when you have a
   result to deliver.
3. <a id="problem-settle-promise"></a>When the JS caller is waiting
   on a [`Promise`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise),
   a way to **settle that promise** from a background context.

Neon provides one type per problem.
[`Root<T>`](/api/neon/handle/struct.Root.html) handles
[(1)](#problem-keep-alive).
[`Channel`](/api/neon/event/struct.Channel.html) handles
[(2)](#problem-back-to-main).
[`Deferred`](/api/neon/types/struct.Deferred.html), paired with
[`JsPromise`](/api/neon/types/struct.JsPromise.html), handles
[(3)](#problem-settle-promise). Any subset is useful on its own — a
[`Root`](/api/neon/handle/struct.Root.html) without a
[`Channel`](/api/neon/event/struct.Channel.html) is just a long-lived
JS reference; a [`Channel`](/api/neon/event/struct.Channel.html)
without a [`Deferred`](/api/neon/types/struct.Deferred.html) is
fire-and-forget callback delivery. A common case is all three
combined into a single round-trip: JS calls Rust, Rust spawns a
thread and returns a pending [`Promise`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise),
the thread does its work, and the
[`Promise`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise)
settles with the result.

## `Root`: keeping a JS value alive

A [`Root<T>`](/api/neon/handle/struct.Root.html) is an *owned*
reference to a JS value with no lifetime parameter — the very feature
that [Context lifetimes](/explanation/lifetimes/) introduces it as
the answer to. With no `'cx` constraint, a
[`Root`](/api/neon/handle/struct.Root.html) can outlive the call
that created it for as long as you keep it alive: stash it in a
[`JsBox`](/api/neon/types/struct.JsBox.html)-owned struct or in
Neon's [`LocalKey`](/api/neon/thread/struct.LocalKey.html) cell, and
the GC will not reclaim its referent until the
[`Root`](/api/neon/handle/struct.Root.html) is dropped.

```rust
# use neon::prelude::*;
use neon::thread::LocalKey;

static ON_READY: LocalKey<Root<JsFunction>> = LocalKey::new();

#[neon::export]
fn set_on_ready<'cx>(
    cx: &mut Cx<'cx>,
    callback: Handle<'cx, JsFunction>,
) -> NeonResult<()> {
    ON_READY.get_or_try_init(cx, |cx| Ok(callback.root(cx)))?;
    Ok(())
}
```

[`LocalKey`](/api/neon/thread/struct.LocalKey.html) lives in
[`neon::thread`](/api/neon/thread/index.html) and is specifically
designed for this: each addon *instance* gets its own slot, which
matters because Node's [`worker_threads`](https://nodejs.org/api/worker_threads.html)
can instantiate the same addon multiple times in a single process,
and a [`Root`](/api/neon/handle/struct.Root.html) is only valid in
the JS thread it was rooted on. A plain `static Root<_>` (or
[`OnceLock<Root<_>>`](https://doc.rust-lang.org/std/sync/struct.OnceLock.html))
would compile, but cross-worker access would panic on `into_inner`.
[`std::thread_local!`](https://doc.rust-lang.org/std/macro.thread_local.html)
is also the wrong choice here — JS threads aren't guaranteed to be
1:1 with system threads, so OS-level TLS isn't the right unit of
isolation. See the [`neon::thread`](/api/neon/thread/index.html)
module docs for the full lifecycle story.

Two specific properties matter for the threading story:

- [`Root<T>`](/api/neon/handle/struct.Root.html) is
  [`Send`](https://doc.rust-lang.org/std/marker/trait.Send.html) and
  [`Sync`](https://doc.rust-lang.org/std/marker/trait.Sync.html), so
  you can move one onto a worker thread or share it across threads.
- You can't *use* a [`Root`](/api/neon/handle/struct.Root.html) on a
  worker thread — there's no JS execution context there. The
  [`Root`](/api/neon/handle/struct.Root.html) keeps the value alive
  for the round-trip; the actual access happens back on the main
  thread, with a [`Channel`](/api/neon/event/struct.Channel.html).

## `Channel`: getting back to the main thread

A [`Channel`](/api/neon/event/struct.Channel.html) is a thread-safe
handle to the JS main thread. You get one from
[`cx.channel()`](/api/neon/context/trait.Context.html#method.channel),
clone it freely, and call
[`channel.send(|cx| ...)`](/api/neon/event/struct.Channel.html#method.send)
to schedule a closure that will run on the main thread with a fresh
[`Cx`](/api/neon/context/struct.Cx.html). Inside that closure, you
can finally touch JS values again — including any
[`Root`](/api/neon/handle/struct.Root.html)s you brought along.

```rust
# use neon::prelude::*;
#[neon::export]
fn ping<'cx>(cx: &mut Cx<'cx>, callback: Handle<'cx, JsFunction>) {
    let callback = callback.root(cx);
    let channel = cx.channel();

    std::thread::spawn(move || {
        // ... do background work ...

        // Schedule a closure that runs on the JS main thread.
        channel.send(move |mut cx| {
            let callback = callback.into_inner(&mut cx);
            callback.bind(&mut cx).exec()?;
            Ok(())
        });
    });
}
```

Two things to notice. First, the closure handed to
[`send`](/api/neon/event/struct.Channel.html#method.send) runs with
its own fresh [`Cx`](/api/neon/context/struct.Cx.html) — not the
context you started in. Each scheduled main-thread execution gets a
new one. Second, a live
[`Channel`](/api/neon/event/struct.Channel.html) holds a reference
on the Node event loop, keeping the process alive as long as the
channel exists — the same role
[`setTimeout(...)`](https://developer.mozilla.org/en-US/docs/Web/API/setTimeout)
and friends play on the JS side. If you need a channel that *doesn't*
keep Node alive (e.g. a long-lived progress reporter that shouldn't
block process exit), call
[`channel.unref(cx)`](/api/neon/event/struct.Channel.html#method.unref) —
the analogue of
[`timeout.unref()`](https://nodejs.org/api/timers.html#timeoutunref)
in Node.

## `Deferred` and `JsPromise`: settling the result

[`Root`](/api/neon/handle/struct.Root.html) keeps values alive;
[`Channel`](/api/neon/event/struct.Channel.html) gets you back to JS.
What's missing is the *output* side: how do you tell the JS caller
"here's the result you were waiting for"?

[`cx.promise()`](/api/neon/context/trait.Context.html#method.promise)
returns a paired
[`Deferred`](/api/neon/types/struct.Deferred.html) and a
[`Handle<JsPromise>`](/api/neon/types/struct.JsPromise.html) — the
same producer/consumer split you get from
[`std::sync::mpsc::channel()`](https://doc.rust-lang.org/std/sync/mpsc/fn.channel.html)
returning [`(Sender, Receiver)`](https://doc.rust-lang.org/std/sync/mpsc/struct.Sender.html),
just with one value to deliver instead of a stream. You hand the
[`JsPromise`](/api/neon/types/struct.JsPromise.html) back to JS
immediately — it's pending. Then, when your background work
finishes, you settle the
[`Deferred`](/api/neon/types/struct.Deferred.html) and the JS
[`Promise`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise)
resolves (or rejects) with whatever you give it.

```rust
# use neon::prelude::*;
#[neon::export]
fn delayed_answer<'cx>(cx: &mut Cx<'cx>) -> Handle<'cx, JsPromise> {
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();

    std::thread::spawn(move || {
        // ... do background work ...

        // Schedules a closure on the main thread that resolves
        // (or rejects) the Promise using a fresh `Cx`.
        deferred.settle_with(&channel, |mut cx| Ok(cx.number(42)));
    });

    promise
}
```

[`settle_with`](/api/neon/types/struct.Deferred.html#method.settle_with)
is the convenience that combines the previous two pieces: it uses
the [`Channel`](/api/neon/event/struct.Channel.html) you pass in to
schedule a main-thread closure that runs with a fresh
[`Cx`](/api/neon/context/struct.Cx.html), and the closure's
[`Ok`](https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok)/[`Err`](https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err)
return value resolves or rejects the
[`Deferred`](/api/neon/types/struct.Deferred.html). If you drop a
[`Deferred`](/api/neon/types/struct.Deferred.html) without settling
it, the
[`JsPromise`](/api/neon/types/struct.JsPromise.html) auto-rejects.

## The round-trip

Put the three pieces together and the choreography looks like this:

```mermaid
sequenceDiagram
    autonumber
    participant JS as JavaScript (main thread)
    participant R as Rust (main thread)
    participant W as Rust (worker thread)

    JS->>R: call exported fn(args)
    R->>R: cx.channel() → Channel
    R->>R: cx.promise() → (Deferred, JsPromise)
    R->>R: callback.root(cx) → Root&lt;JsFunction&gt;
    R->>W: std::thread::spawn(move || ...)
    R-->>JS: return JsPromise (pending)
    W->>W: compute / I/O / etc.
    W->>R: deferred.settle_with(&channel, |cx| ...)
    Note over R: closure runs on main thread<br/>with a fresh Cx
    R->>JS: Promise resolves (or rejects)
```

Three lanes, one round-trip. The main-thread Rust step never blocks
the JS thread on the background work — it returns the pending
[`JsPromise`](/api/neon/types/struct.JsPromise.html) immediately and
lets the worker do its thing. When the worker finishes, the
[`Channel`](/api/neon/event/struct.Channel.html) gets it back onto
the main thread for the JS-touching part.

## Why three types and not one

It's tempting to ask why this isn't one combined "async result"
abstraction. Each piece answers a distinct question, and you don't
always need all three:

- **[`Root<T>`](/api/neon/handle/struct.Root.html) alone** — when
  you need to keep a callback alive across multiple invocations but
  you're staying on the main thread. No worker, no
  [`Channel`](/api/neon/event/struct.Channel.html), no
  [`Promise`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise).
- **[`Root`](/api/neon/handle/struct.Root.html) +
  [`Channel`](/api/neon/event/struct.Channel.html)** — fire-and-forget
  callback patterns: a worker thread that periodically invokes a JS
  callback (e.g. a progress reporter). The caller doesn't await
  anything.
- **[`Deferred`](/api/neon/types/struct.Deferred.html) +
  [`Channel`](/api/neon/event/struct.Channel.html)** — single-shot
  async result. The most common case, and what most macros generate.
- **All three** — async result that also needs to call into a
  user-supplied JS callback during the work, not just at the end.

Keeping them separate lets each be useful on its own, and the
[`settle_with`](/api/neon/types/struct.Deferred.html#method.settle_with)
convenience covers the most common composition without forcing it on
everyone.

## What this means for the macros

This page is the choreography. The
[`#[neon::export]`](/api/neon/attr.export.html) macros (in their
various flavors —
[`async fn`](/how-to/async-fn/), [`(async) impl Future`](/how-to/sync-setup-async/),
and the [`task`](/how-to/blocking-work/) flavor for the worker
pool) hide the choreography by generating the
[`Channel`](/api/neon/event/struct.Channel.html) +
[`Deferred`](/api/neon/types/struct.Deferred.html) +
[`Root`](/api/neon/handle/struct.Root.html) plumbing for you.
Knowing what each piece does makes it much easier to read the
expansion when you hit something the macros don't cover and need to
drop down to the primitives directly.

The [*Move work off the main thread*](/tutorials/move-work-off-the-main-thread/)
tutorial is the runnable end-to-end version of the diagram above,
with [`#[neon::export(task)]`](/how-to/blocking-work/) doing the
plumbing. The [*Build a database addon*](/tutorials/build-a-database-addon/)
tutorial does the same with `async fn` exports.

## Where to go next

- The [*async-fn*](/how-to/async-fn/) and
  [*sync-setup-async*](/how-to/sync-setup-async/) how-tos cover the
  two `#[neon::export]` flavors that produce a
  [`Promise`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise).
- The [*Move work off the main thread*](/tutorials/move-work-off-the-main-thread/)
  tutorial walks through a full example end to end.
- The [`Channel`](/api/neon/event/struct.Channel.html) and
  [`Deferred`](/api/neon/types/struct.Deferred.html) rustdoc pages
  cover the edges: `Channel::try_send` for fallible scheduling,
  `Deferred::reject` for explicit error paths, and
  [`JsFuture`](/api/neon/types/struct.JsFuture.html) for awaiting JS
  [`Promise`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise)s
  from Rust.
