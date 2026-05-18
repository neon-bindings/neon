---
title: Context lifetimes
description: Why Neon handles carry a 'cx lifetime, what it represents, the use-after-scope bug it rules out, how to keep a value alive past the call with Root, and how Lock turns the same borrow rules into runtime checks for buffer bytes.
---

:::note[Draft]
This page is a draft pending review.
:::

The [type hierarchy](/explanation/type-hierarchy/) introduced
[`Handle<'cx, T>`](/api/neon/handle/struct.Handle.html) as the way
Rust refers to a JavaScript value. The lifetime parameter `'cx` is
the most distinctive part of that type. This page explains what
`'cx` represents, the bug it stops you from writing, and what to do
when you need a value to outlive it.

## Two scopes meeting at the boundary

JavaScript and Rust both have a notion of *scope*, but they use it
to enforce different rules:

- **JavaScript's** scope is about the garbage collector. Local
  variables in a function keep their values reachable until the
  function returns; once it does, the GC is free to reclaim
  anything that's no longer referenced.
- **Rust's** scope is about ownership. The borrow checker tracks
  which references can be valid where, and refuses to compile code
  that uses a reference past its lifetime.

When JavaScript calls into a Neon function, the engine sets up a
short-lived **handle scope** that protects every JS value the Rust
code touches. As soon as the function returns, that scope ends and
the engine is free to reclaim or relocate any value the Rust code
hasn't explicitly preserved.

Neon ties those two systems together at the type level. The
[`Cx<'cx>`](/api/neon/context/struct.Cx.html) value the engine
hands you on the way in *is* the Rust handle on that JS scope, and
every [`Handle<'cx, T>`](/api/neon/handle/struct.Handle.html) you
get out of it borrows from `cx`. When `cx` goes out of scope, every
handle that came from it does too — and the borrow checker won't
let you keep one around past that point.

## What `'cx` represents

In a function using [`Cx<'cx>`](/api/neon/context/struct.Cx.html):

```rust
# use neon::prelude::*;
#[neon::export]
fn hello<'cx>(cx: &mut Cx<'cx>) -> JsResult<'cx, JsString> {
    Ok(cx.string("hello"))
}
```

`'cx` is the lifetime of the active engine context — one
JavaScript-to-Rust call. Everywhere it appears, it represents the
same fact: *"this thing is only valid for the rest of this call."*

- [`Cx<'cx>`](/api/neon/context/struct.Cx.html) — the context value.
- [`Handle<'cx, T>`](/api/neon/handle/struct.Handle.html) — a
  handle to a JS value that came from `cx`.
- [`JsResult<'cx, T>`](/api/neon/result/type.JsResult.html) — a
  shorthand for `Result<Handle<'cx, T>, Throw>`.

When you return a `JsResult<'cx, JsString>`, the lifetime says
"this handle is still valid by the time my caller looks at it."
The compiler stops you from accidentally returning one that isn't.

## The bug it rules out

[`Context::execute_scoped`](/api/neon/context/trait.Context.html#method.execute_scoped)
runs a closure in a fresh, short-lived scope. Handles created inside
that scope are cleaned up as soon as the closure returns — that's the
whole point of the method.

A reasonable-looking mistake would be to try to *return* one of those
inner handles:

```rust,compile_fail
# use neon::prelude::*;
fn build_then_escape<'cx>(
    cx: &mut Cx<'cx>,
) -> JsResult<'cx, JsString> {
    cx.execute_scoped(|mut inner| {
        // `temp` lives only as long as the inner scope.
        let temp = inner.string("doomed");
        Ok(temp)
        // ERROR: cannot return a handle that borrows from `inner`
    })
}
```

The closure parameter `inner` here isn't a regular
[`Cx`](/api/neon/context/struct.Cx.html); it's a
[`ScopedCx`](/api/neon/context/struct.ScopedCx.html) — a context
flavor that exists specifically to enforce a tighter lifetime
relationship between the inner and outer scopes. That's what catches
the escape attempt.

Without that constraint, this would compile and `temp` would dangle:
the inner scope drops the moment the closure returns, the engine is
told the handles inside it are no longer needed, and the caller would
be left holding a reference to a freed value.

With it, the borrow checker sees that the returned handle borrows
from `inner`, and `inner` doesn't outlive the closure. **The bug
becomes a compile-time error.** You can't ship the broken version
even if you wanted to.

When you genuinely *do* need a result to outlive an inner scope, Neon
provides
[`compute_scoped`](/api/neon/context/trait.Context.html#method.compute_scoped),
which threads the outer lifetime through for you — exactly one handle
gets promoted from the inner scope into the outer one. The lifetime
on the return type is what makes that promotion expressible at all.

This is the reason the lifetime is there. It looks like
ceremony when you're writing function signatures, but it's the
mechanism that makes "JS values from Rust" memory-safe by default.

## When you need a value to outlive the scope

Plenty of real Neon code wants to keep a JS value around longer
than one call: scheduling work on a worker thread, holding a
callback to invoke later, caching a function reference between
calls. It's important that the value doesn't get garbage collected.
The lifetime would normally forbid that — and that's where
[`Root<T>`](/api/neon/handle/struct.Root.html) comes in.

A `Root<T>` is an **owned** reference to a JS value, with no
lifetime parameter:

```rust
# use neon::prelude::*;
fn save_callback<'cx>(
    cx: &mut Cx<'cx>,
    f: Handle<'cx, JsFunction>,
) -> Root<JsFunction> {
    f.root(cx)
}
```

Once you have a `Root<T>`, you can store it in a struct, send it
across threads, hold it indefinitely. The JS engine knows it
exists and won't garbage-collect the underlying value out from
under you.

To turn a `Root<T>` back into something you can call or read, you
ask for a fresh handle in the next available scope:

```rust
# use neon::prelude::*;
fn use_callback<'cx>(
    cx: &mut Cx<'cx>,
    saved: &Root<JsFunction>,
) -> JsResult<'cx, JsValue> {
    let f: Handle<'cx, JsFunction> = saved.to_inner(cx);
    f.call_with(cx).apply(cx)
}
```

The new handle has the new scope's lifetime, and the chain of
guarantees starts over. `Root<T>` is the bridge between "JS value
that must outlive the call" (the threading-lifecycle problem) and
"JS value the borrow checker can safely reason about" (this page's
problem).

The [threading lifecycle](/explanation/threading-lifecycle/) page
covers how [`Root<T>`](/api/neon/handle/struct.Root.html),
[`Channel`](/api/neon/event/struct.Channel.html), and
[`Deferred`](/api/neon/types/struct.Deferred.html) cooperate to let
work flow off the JS thread and back.

## Static vs. runtime checks: borrowing buffer bytes

The lifetime story so far has been about static checks: the
compiler refuses to let a handle escape its scope. Rust's other
borrow rule — *one mutable borrow or many immutable borrows, never
both at once* — also applies to the bytes inside a JavaScript
buffer, but here the picture is more subtle. Two
[`Handle<JsArrayBuffer>`](/api/neon/types/struct.JsArrayBuffer.html)
values can refer to the *same* underlying memory (one as a view of
another, for example), so the compiler can't always prove from the
types alone that two borrows don't overlap.

Neon's [`TypedArray`](/api/neon/types/buffer/trait.TypedArray.html)
trait gives you both options. Pick the static path when the
compiler can prove what you need; pick the runtime path when it
can't.

### Static: [`as_slice`](/api/neon/types/buffer/trait.TypedArray.html#tymethod.as_slice) and [`as_mut_slice`](/api/neon/types/buffer/trait.TypedArray.html#tymethod.as_mut_slice)

The everyday case. Both methods borrow from
[`Cx`](/api/neon/context/struct.Cx.html), so the borrow checker
applies its usual rules — and you pay no runtime cost:

```rust
# use neon::prelude::*;
use neon::types::buffer::TypedArray;

#[neon::export]
fn double<'cx>(
    cx: &mut Cx<'cx>,
    mut array: Handle<'cx, JsUint32Array>,
) {
    for elem in array.as_mut_slice(cx).iter_mut() {
        *elem *= 2;
    }
}
```

[`as_mut_slice`](/api/neon/types/buffer/trait.TypedArray.html#tymethod.as_mut_slice)
takes `&mut cx`, so while the returned slice is live, nothing else
that needs `cx` can run. That's exactly the static guarantee Rust
gives you for any `&mut`: no aliasing, no overlap. The trade-off
is reach: because the rule is enforced through `cx`, you can't have
two such slices live at once.

### Runtime: [`Lock`](/api/neon/types/buffer/struct.Lock.html), [`try_borrow`](/api/neon/types/buffer/trait.TypedArray.html#tymethod.try_borrow), and [`try_borrow_mut`](/api/neon/types/buffer/trait.TypedArray.html#tymethod.try_borrow_mut)

When the static checker is too strict — typically because you need
to look at two regions of the same
[`JsArrayBuffer`](/api/neon/types/struct.JsArrayBuffer.html), or
hand a slice to code that also wants `cx` — Neon offers a runtime
counterpart. A
[`Lock`](/api/neon/types/buffer/struct.Lock.html) freezes the
engine and keeps a ledger of which byte ranges are currently
borrowed, mutably or immutably:

```rust
# use neon::prelude::*;
use neon::types::buffer::TypedArray;

#[neon::export]
fn count_bytes<'cx>(
    cx: &mut Cx<'cx>,
    buf: Handle<'cx, JsArrayBuffer>,
) {
    // Acquiring the lock borrows `cx` for the lock's lifetime;
    // the borrow is released when `lock` goes out of scope.
    let lock = cx.lock();
    let result = buf.try_borrow(&lock);
    if let Ok(bytes) = result {
        println!("{} bytes", bytes.len());
    }
}
```

The borrow rule is the same — *one mutable, or many immutable* —
but now it's checked against the live ledger when you call
[`try_borrow`](/api/neon/types/buffer/trait.TypedArray.html#tymethod.try_borrow)
or
[`try_borrow_mut`](/api/neon/types/buffer/trait.TypedArray.html#tymethod.try_borrow_mut).
A conflict produces a
[`BorrowError`](/api/neon/types/buffer/struct.BorrowError.html)
instead of refusing to compile. The returned
[`Ref<'_, T>`](/api/neon/types/buffer/struct.Ref.html) and
[`RefMut<'_, T>`](/api/neon/types/buffer/struct.RefMut.html) act
like RAII guards: dropping them removes their entry from the ledger
so the next borrow can succeed.

### Picking between them

The two options are the same trade-off as
[`Box<T>`](https://doc.rust-lang.org/std/boxed/struct.Box.html) vs.
[`RefCell<T>`](https://doc.rust-lang.org/std/cell/struct.RefCell.html):

- **Static borrow** — fastest, fewest moving parts, but the
  compiler decides what's allowed. Reach for
  [`as_slice`](/api/neon/types/buffer/trait.TypedArray.html#tymethod.as_slice)
  and
  [`as_mut_slice`](/api/neon/types/buffer/trait.TypedArray.html#tymethod.as_mut_slice)
  by default.
- **Runtime borrow** — slightly more expensive (a small ledger
  check, plus the engine lock), but expressive enough to handle
  cases the static rules reject. Reach for
  [`Lock`](/api/neon/types/buffer/struct.Lock.html) +
  [`try_borrow`](/api/neon/types/buffer/trait.TypedArray.html#tymethod.try_borrow)
  / [`try_borrow_mut`](/api/neon/types/buffer/trait.TypedArray.html#tymethod.try_borrow_mut)
  when you need overlap-checking the borrow checker can't do for
  you.

The unifying idea is the same as the rest of this page: Rust's
guarantees follow the JS value across the boundary, but the
*mechanism* — compile-time or runtime — depends on what the
compiler can see.

## Where the lifetime appears in everyday code

If you write Neon with [`#[neon::export]`](/api/neon/attr.export.html)
and plain Rust types, you may never write `'cx` yourself:

```rust
# use neon::types::extract::Error;
#[neon::export]
fn slugify(input: String) -> Result<String, Error> {
    Ok(input.to_lowercase().replace(' ', "-"))
}
```

The macro generates the surrounding glue that takes a `Cx` and
turns the `String` argument into a Rust value with no lifetime tied
to the engine. You only see `'cx` when you reach for the lower-level
APIs — typically because you need a
[`Cx`](/api/neon/context/struct.Cx.html) inside the function body
([*Get a `Cx` inside an exported function*](/how-to/cx-access/)),
or you're returning a JS-shaped value built by hand. In both
cases, the lifetime is doing the same job: making sure every
handle you produce is still valid when the caller looks at it.

## Where to go next

- **You want to spawn work onto another thread.** The
  [threading lifecycle](/explanation/threading-lifecycle/) page
  covers how `Root<T>`, `Channel`, and `Deferred` cooperate to let
  Rust call back into JS safely.
- **You want the type vocabulary.** The
  [type hierarchy](/explanation/type-hierarchy/) walks through the
  concrete `Js*` types that fill in for the `T` in
  `Handle<'cx, T>`.
- **You want a recipe.** The
  [*Get a `Cx` inside an exported function*](/how-to/cx-access/)
  how-to is the practical companion to this page when you actually
  need to write `'cx`.
