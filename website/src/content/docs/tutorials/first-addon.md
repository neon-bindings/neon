---
title: Your first Neon addon
description: Build a small Neon addon from scratch — strings in and out, collections, and error handling, all wired up to JavaScript.
---

The [Quickstart](/getting-started/quickstart/) gets you to "a Rust
function called from Node.js" in five minutes. This tutorial picks up
from there and walks through building a small but realistic Neon
addon — a URL-safe slug generator — exercising the patterns you'll
use in nearly every real project: string arguments, collection
arguments, and fallible functions that throw on error.

By the end, you'll have an addon with several exported Rust functions
that JavaScript can call directly, and you'll understand what each
piece of [`#[neon::export]`](/api/neon/attr.export.html) is doing for
you.

## What we're building

Slugs are the URL-safe, lowercase, hyphen-separated forms of arbitrary
strings — `"Hello, world!"` becomes `"hello-world"`. They're a great
example because:

- The function is genuinely useful and self-contained.
- The input and output are strings, not just numbers — exercising
  Neon's most common conversion path.
- It's easy to extend with more interesting types like `Vec<String>`
  and fallible variants without the example getting unwieldy.

The finished addon exports:

```js
slugify("Hello, World!")          // => "hello-world"
slugify(["A B", "  c__d  "])      // => ["a-b", "c-d"]
slugifyAll(["A B", "  c__d  "])   // => ["a-b", "c-d"]
slugifyStrict("")                 // throws Error
```

## Scaffold the project

If you haven't yet, install the prerequisites in
[Prerequisites](/getting-started/install/). Then:

```sh
npm init neon@latest --app first-neon
cd first-neon
```

The `--app` flag tells [`create-neon`](/reference/cli/) to scaffold a
small standalone application (rather than a publishable library — we
cover that in
[Publish your addon to npm](/tutorials/publish-your-addon-to-npm/) later).
You'll get a project layout like this:

```text
first-neon/
├── Cargo.toml          # Rust crate manifest
├── package.json        # npm scripts and devDependencies
├── src/
│   └── lib.rs          # all your Rust code goes here
└── README.md
```

Open `src/lib.rs`. The scaffolder dropped in a `hello` placeholder —
delete its body, we'll write our own.

## A first export

Replace the contents of `src/lib.rs` with:

```rust
# assert_eq!(slugify("Hello, World!".into()), "hello-world");
# assert_eq!(slugify("  Lots__of---spaces  ".into()), "lots-of-spaces");
#[neon::export]
fn slugify(input: String) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}
```

Then build and call it:

```sh
npm run build
```

```js
const addon = require("./index.node");

console.log(addon.slugify("Hello, World!"));
// => "hello-world"
console.log(addon.slugify("  Lots__of---spaces  "));
// => "lots-of-spaces"
```

A few things worth noticing:

- The Rust function takes `String` and returns `String`. Neon's
  [`TryFromJs`](/api/neon/types/extract/trait.TryFromJs.html) and
  [`TryIntoJs`](/api/neon/types/extract/trait.TryIntoJs.html) traits
  handle the conversions automatically — you'll see this same pattern
  for every primitive and collection type.
- There's no addon init code anywhere. The
  [`#[neon::export]`](/api/neon/attr.export.html) attribute registers
  the function with the addon on its own; you only need to write a
  [`#[neon::main]`](/api/neon/attr.main.html) entry point if you want
  to do extra work at addon-load time (initializing a logger,
  registering an executor, etc.).
- The exported JavaScript name is `slugify`, the same as the Rust
  identifier. If you wanted a different JS-side name (`slugify` →
  `toSlug`, say), see
  [Rename exports](/how-to/rename-exports/).

## Accepting a list

Real applications usually want to slugify many strings at once. JS
arrays don't extract directly into `Vec<T>` — Neon distinguishes
between regular JS arrays and typed arrays — so we use the
[`Array<T>`](/api/neon/types/extract/struct.Array.html) wrapper from
[`neon::types::extract`](/api/neon/types/extract/index.html). Add this
below the first function:

```rust
# let Array(out) = slugify_all(Array(vec![
#   "A B".into(),
#   "  c__d  ".into(),
#   "Hello, World!".into(),
# ]));
# assert_eq!(out, vec!["a-b", "c-d", "hello-world"]);
# fn slugify(s: String) -> String {
#   let mut out = String::with_capacity(s.len());
#   for c in s.chars() {
#     if c.is_ascii_alphanumeric() { out.push(c.to_ascii_lowercase()); }
#     else if !out.ends_with('-') { out.push('-'); }
#   }
#   out.trim_matches('-').to_string()
# }
use neon::types::extract::Array;

#[neon::export]
fn slugify_all(Array(inputs): Array<Vec<String>>) -> Array<Vec<String>> {
    Array(inputs.into_iter().map(slugify).collect())
}
```

Rebuild and try it:

```js
console.log(addon.slugifyAll(["A B", "  c__d  ", "Hello, World!"]));
// => ["a-b", "c-d", "hello-world"]
```

A few things to call out:

- The Rust identifier `slugify_all` is exposed to JavaScript as
  `slugifyAll` — Neon converts `snake_case` to `camelCase` by default.
  See [Rename exports](/how-to/rename-exports/) for overrides.
- `Array<T>` extracts a plain JavaScript array. Bare `Vec<T>` is
  reserved for JavaScript
  [typed arrays](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray)
  via [`JsTypedArray<T>`](/api/neon/types/struct.JsTypedArray.html),
  which is why `slugify_all` wraps `Vec<String>` in `Array<_>`.
- For deeply-nested or schema-driven data, the
  [`Json<T>`](/api/neon/types/extract/struct.Json.html) extractor lets
  any [`serde`](https://docs.rs/serde/latest/serde/)-compatible type
  cross the boundary; see [Use serde with `Json<T>`](/how-to/serde-json/).

## Failing properly

So far the Rust functions can't fail — every input has a sensible
output. (Neon's argument extraction will still throw a
[`TypeError`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypeError)
on the JavaScript side if you pass, say, a number where a string is
expected.) But what if you want to refuse blank input?

The idiomatic way to fail from a Neon function is to return a
`Result<T, E>` where `E` implements
[`TryIntoJs`](/api/neon/types/extract/trait.TryIntoJs.html). The
[`Error`](/api/neon/types/extract/struct.Error.html) type from
[`neon::types::extract`](/api/neon/types/extract/index.html) is the
default choice — it converts a wide range of Rust error types into
the appropriate JavaScript exception
([`Error`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error),
[`TypeError`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypeError),
[`RangeError`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RangeError)).

Add a strict variant:

```rust
# assert_eq!(slugify_strict("Hello".into()).unwrap(), "hello");
# assert!(slugify_strict("".into()).is_err());
# assert!(slugify_strict("!!!".into()).is_err());
# fn slugify(s: String) -> String {
#   let mut out = String::with_capacity(s.len());
#   for c in s.chars() {
#     if c.is_ascii_alphanumeric() { out.push(c.to_ascii_lowercase()); }
#     else if !out.ends_with('-') { out.push('-'); }
#   }
#   out.trim_matches('-').to_string()
# }
use neon::types::extract::Error;

#[neon::export]
fn slugify_strict(input: String) -> Result<String, Error> {
    let slug = slugify(input);
    if slug.is_empty() {
        return Err(Error::from("slug is empty"));
    }
    Ok(slug)
}
```

Now from JavaScript:

```js
console.log(addon.slugifyStrict("Hello"));      // => "hello"
console.log(addon.slugifyStrict(""));           // throws Error("slug is empty")
console.log(addon.slugifyStrict("!!!"));        // throws Error("slug is empty")
console.log(addon.slugifyStrict(42));           // throws TypeError: expected string
```

The throw lands as a real JavaScript
[`Error`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error)
you can `try`/`catch` — no special bridging required.

For the full picture — including how to throw specific JavaScript
error subclasses and how to catch JS-thrown errors from inside Rust —
see [Throw and catch JavaScript errors from Rust](/how-to/errors/).

## Overloading on shape

JavaScript developers often overload function signatures — the same
function might accept *either* a single string or an array of strings,
and return a value of the matching shape. Rust doesn't have function
overloading, but the
[`Either`](https://docs.rs/either/latest/either/enum.Either.html) type
gets you close.

Let's rewrite `slugify` to accept either shape and delegate to
`slugify_strict` or `slugify_all` based on what came in:

```rust
# assert!(matches!(slugify(Either::Left("Hello".into())).unwrap(), Either::Left(s) if s == "hello"));
# assert!(matches!(
#   slugify(Either::Right(Array(vec!["A B".into(), "c d".into()]))).unwrap(),
#   Either::Right(Array(v)) if v == vec!["a-b", "c-d"]
# ));
# assert!(slugify(Either::Left("".into())).is_err());
# fn slugify_one(s: String) -> String {
#   let mut out = String::with_capacity(s.len());
#   for c in s.chars() {
#     if c.is_ascii_alphanumeric() { out.push(c.to_ascii_lowercase()); }
#     else if !out.ends_with('-') { out.push('-'); }
#   }
#   out.trim_matches('-').to_string()
# }
# fn slugify_strict(input: String) -> Result<String, Error> {
#   let s = slugify_one(input);
#   if s.is_empty() { return Err(Error::from("slug is empty")); }
#   Ok(s)
# }
# fn slugify_all(Array(inputs): Array<Vec<String>>) -> Array<Vec<String>> {
#   Array(inputs.into_iter().map(slugify_one).collect())
# }
use either::Either;
use neon::types::extract::{Array, Error};

#[neon::export]
fn slugify(
    input: Either<String, Array<Vec<String>>>,
) -> Result<Either<String, Array<Vec<String>>>, Error> {
    match input {
        Either::Left(s) => Ok(Either::Left(slugify_strict(s)?)),
        Either::Right(arr) => Ok(Either::Right(slugify_all(arr))),
    }
}
```

The same export now handles both shapes from JavaScript and returns
the matching type:

```js
console.log(addon.slugify("Hello, World!"));        // => "hello-world"
console.log(addon.slugify(["A B", "  c__d  "]));    // => ["a-b", "c-d"]
console.log(addon.slugify(""));                     // throws Error("slug is empty")
console.log(addon.slugify(42));                     // throws TypeError
```

## What `#[neon::export]` did for you

You wrote a handful of plain Rust functions and got plain JavaScript
functions, callable from Node.js. Here's the short version of what
[`#[neon::export]`](/api/neon/attr.export.html) handled along the way:

- **Argument extraction.** Each parameter type implements
  [`TryFromJs`](/api/neon/types/extract/trait.TryFromJs.html), and the
  generated wrapper calls it on each argument from the JavaScript call
  site. Mismatched types throw a
  [`TypeError`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypeError)
  automatically.
- **Return conversion.** The return type implements
  [`TryIntoJs`](/api/neon/types/extract/trait.TryIntoJs.html), which
  converts the Rust value back to a JS value. For
  `Result<T, Error>`, the wrapper unwraps `Ok` into a JS value and
  turns `Err` into a thrown exception.
- **Addon registration.** Every
  [`#[neon::export]`](/api/neon/attr.export.html) registers itself with
  the addon's init code, so the resulting `index.node` exposes every
  annotated function on the object you
  [`require()`](https://nodejs.org/api/modules.html#requireid).
- **JavaScript naming.** `snake_case` Rust identifiers become `camelCase`
  JS names by default; `name = "..."` overrides that.

For a longer look at the macro internals, see
[How `#[neon::export]` works](/explanation/export-internals/).

## Where next

- [Move work off the main thread](/tutorials/move-work-off-the-main-thread/) —
  move CPU-bound work off the JavaScript main thread.
- [Build a database addon](/tutorials/build-a-database-addon/) — `async fn`
  exports that return Promises, plus `#[neon::class]` for stateful handles.
- [Pass common types between Rust and JavaScript](/how-to/common-types/)
  — the rest of the conversion vocabulary (numbers, buffers, objects).
- [Throw and catch JavaScript errors from Rust](/how-to/errors/) —
  more on `extract::Error`, `?`, and bridging exceptions.
