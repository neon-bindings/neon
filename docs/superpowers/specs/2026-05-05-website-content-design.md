# Website content — design spec

**Date:** 2026-05-05
**Status:** draft, awaiting approval
**Prereq:** `2026-05-04-website-starlight-design.md` (the structural site
already exists; this spec covers writing the actual page content for it)

## 1. Goal

The website at `website/` ships with thirty placeholder pages — one
per section of the documentation site. Each placeholder has a real
title, a `description`, a `Status: skeleton` banner, and a one-paragraph
summary of what the finished page is meant to cover. **None of them have
real content.**

This spec defines how we turn those thirty placeholders into a complete,
coherent, testable documentation site.

## 2. Source of truth

All code samples must compile as Rust doctests via the existing
`website/build.rs` harness (`cargo test --doc -p website`). To keep
them honest:

- **API surface:** the local `crates/neon` source in this repo. The
  manifest there reads `1.1.1` but the code is effectively v1.2 — that's
  what readers running `cargo add neon` from `main` will get. Our docs
  target this repo's `HEAD`.
- **Idiomatic patterns:** the [neon-bindings/examples](https://github.com/neon-bindings/examples)
  repository, especially [PR #104](https://github.com/neon-bindings/examples/pull/104).
- **Crate-level reference:** `cargo doc -p neon` output, mounted at
  `/api/` on the site.

Things confirmed about the API while drafting this spec:

- `#[neon::export]` accepts `task`, `async`, `name`, `json`, `context`,
  `this`, applied to functions, consts/statics, and `impl` blocks
  (`#[neon::export(class)]`).
- `#[neon::class]` is real and implemented in this repo.
- Async exports require `set_global_executor` (or the
  `tokio-rt-multi-thread` feature that auto-wires Tokio at module load).
- **No AbortController helper exists** — that how-to documents *manual*
  JS interop, not a built-in API.
- **No streams helper exists** — same caveat.
- MSRV per README is **Rust 1.65**.
- Default Node-API feature is `napi-8`, with `napi-1` … `napi-7` and
  `napi-experimental` available.

## 3. Audience and tone

**Audience:** experienced JavaScript / Node.js developers, comfortable
shipping npm packages, with at least passing familiarity with Rust
syntax (or willing to read the linked Rust Book chapters). They are not
all systems programmers.

**Tone:** [react.dev](https://react.dev/)-style — friendly,
illustrative, direct, second-person ("you"). Short paragraphs, plenty
of inline code, occasional callouts that say *notice that…* or *if
you've used X before…*. Avoid lecturing; show the code, then explain.

**Non-goals:**

- Not a Rust tutorial. Link to the Rust Book where it makes sense.
- Not a Node.js tutorial. Assume `npm`, `package.json`, and ESM.
- Not Node-API documentation. Mention NAPI when it's relevant
  (compatibility, low-level escape hatches), but the docs are about
  Neon, not the underlying engine.

**Terminology:** call the thing we're building a Neon **addon**, not
a Neon module. This mirrors Node's own term for native binaries
([Node-API addons](https://nodejs.org/api/n-api.html)) and avoids the
collision with JavaScript modules (the `.mjs` / `.cjs` files that
load the addon). "Addon-load time," "addon init," "the addon's
exports" — never "module-load time" or "module init." The word
"module" stays reserved for genuine JavaScript-module concepts (a
`.mjs` file, `module.exports`, `import`).

**JavaScript example style:** load addons with CommonJS
[`require()`](https://nodejs.org/api/modules.html#requireid), not ESM
`import`. Node's
[ESM addon support](https://nodejs.org/api/addons.html#loading-addons-using-import)
is gated behind `--experimental-addon-modules`, so
`import addon from "./index.node"` does *not* work out of the box.
Use `.cjs` filenames for runnable JavaScript snippets that load an
addon, and wrap top-level `await` in an async IIFE since `.cjs` is
synchronous.

## 3a. Linking conventions

**Every prose mention of a named item links to its canonical
reference.** A mention is "prose" if the reader could click on it;
mentions inside fenced code blocks are never linked. Link every
occurrence, not just the first — readers land mid-page from search and
deep links.

### Items in the `neon` crate

Link to the matching rustdoc page on the same site under `/api/neon/`.
URL patterns follow rustdoc's conventions:

| Item kind | Example | URL pattern |
|---|---|---|
| Attribute macro | `#[neon::export]` | `/api/neon/attr.export.html` |
| Function macro | `register_module!` | `/api/neon/macro.register_module.html` |
| Struct | `Cx` | `/api/neon/context/struct.Cx.html` |
| Trait | `TryFromJs` | `/api/neon/types/extract/trait.TryFromJs.html` |
| Function | `set_global_executor` | `/api/neon/fn.set_global_executor.html` |
| Module | `extract` | `/api/neon/types/extract/index.html` |

The dev-server proxy and the production build both serve `/api/...`
from the live rustdoc, so these links are always valid.

### Node.js APIs

Link to the official [Node.js documentation](https://nodejs.org/api/).
Useful anchor points:

| Topic | URL |
|---|---|
| `require()` | `https://nodejs.org/api/modules.html#requireid` |
| `module.exports` | `https://nodejs.org/api/modules.html#moduleexports` |
| ESM `import` | `https://nodejs.org/api/esm.html` |
| `Buffer` | `https://nodejs.org/api/buffer.html#class-buffer` |
| `process` | `https://nodejs.org/api/process.html` |
| Worker threads | `https://nodejs.org/api/worker_threads.html` |
| Streams | `https://nodejs.org/api/stream.html` |
| Node-API version matrix | `https://nodejs.org/api/n-api.html#node-api-version-matrix` |

Always link the **stable** docs (`https://nodejs.org/api/...`), not a
specific version (`/docs/v20.x/api/...`).

### Web platform / JavaScript language APIs

Link to [MDN](https://developer.mozilla.org/) for anything that's part
of the web platform or the JavaScript language itself rather than
Node.js-specific.

| Topic | URL |
|---|---|
| `Promise` | `https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise` |
| `AbortController` | `https://developer.mozilla.org/en-US/docs/Web/API/AbortController` |
| `AbortSignal` | `https://developer.mozilla.org/en-US/docs/Web/API/AbortSignal` |
| `fetch` | `https://developer.mozilla.org/en-US/docs/Web/API/Window/fetch` |
| `ArrayBuffer` | `https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer` |
| Typed arrays | `https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray` |
| `async function` | `https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function` |

Rule of thumb: if it's available across browsers and Node, link MDN.
If it's a Node-only API or a Node-specific behavior of a cross-runtime
API, link Node.js.

### Rust ecosystem

Link to canonical sources for the Rust language and ecosystem:

| Source | When |
|---|---|
| The Rust Book (`https://doc.rust-lang.org/book/`) | Language concepts the reader may need to brush up on. |
| The standard library (`https://doc.rust-lang.org/std/`) | `std` types and traits. |
| docs.rs (`https://docs.rs/<crate>/`) | Third-party crates (`tokio`, `serde`, `either`). |

Never copy reference material from any of these into our docs — link
out instead.

## 4. Diátaxis discipline

| Category | Reader's intent | Voice | Length |
|---|---|---|---|
| **Getting started** | "I want to be running in five minutes." | Imperative. Just the steps. | Short (300–500 words). |
| **Tutorials** | "Teach me a new capability by building something." | Story-shaped. Fully runnable from start to finish. Progressive. | Long (1200–2000 words). |
| **How-to guides** | "I have a specific problem. How do I solve it?" | Problem → working solution → "why this works" footer. Self-contained, no narrative. | Medium (500–900 words). |
| **Reference** | "What does this thing accept? What does it return?" | Tables, lists, declarative. No motivation. | Whatever the table needs. |
| **Explanation** | "Help me understand what's happening underneath." | Conceptual essay. Diagrams welcome. Code as illustration, not as recipe. | Medium–long (700–1500 words). |

Each finished page should feel like it belongs to exactly one of these
buckets. If a how-to is becoming a tutorial, split it. If a reference
page is becoming an essay, move the essay to *Explanation*.

## 5. Page-by-page plan

Pages are listed in the order I'll write them. Each entry has:

- **Slug** — the file path under `src/content/docs/`.
- **Working title** — the current title; we'll review for clarity at the
  end of each section, per "titles after content" decision.
- **Bucket** — Diátaxis category.
- **Goal** — what the reader walks away knowing.
- **Notes** — content I already know I want to include, or warnings.

The order is **spine-first**: install → first module → core
explanations → how-tos → reference. Each batch becomes its own commit.

### Batch 1 — Getting started (the spine)

| # | Slug | Bucket | Goal | Notes |
|---|---|---|---|---|
| 1 | `getting-started/install.md` | Getting started | Reader has Rust + Node ready and can run `npm init neon@latest`. | Cover toolchain prereqs (rustup, supported Node versions, platform build tools), then scaffold step. Cross-link to *Supported platforms* reference. |
| 2 | `getting-started/quickstart.md` | Getting started | Reader has a working Neon addon they can call from JS in <5 minutes. | Minimal walkthrough: scaffold → edit one Rust function → build → require from Node. Defers explanations to the *first-module* tutorial. |

### Batch 2 — Tutorials

Each tutorial is fully runnable, end-to-end, and exercised by doctests.

| # | Slug | Bucket | Goal | Notes |
|---|---|---|---|---|
| 3 | `tutorials/first-addon.md` | Tutorial | Reader builds a small but complete Neon addon from scratch and understands every line. | Expand on quickstart — same project, but every concept explained. Introduce `#[neon::export]`, args, return types, and the `npm` build script. |
| 4 | `tutorials/move-work-off-the-main-thread.md` | Tutorial | Reader takes a CPU-bound function and moves it onto Node's worker pool. | Build a small fibonacci-style example. Show before/after blocking. Cover the `task` flavor of `#[neon::export]`. Cross-link to the *Run blocking work on the worker pool* how-to. |
| 5 | `tutorials/build-a-database-addon.md` | Tutorial | Reader builds a SQLite-backed addon: opens a database as a JS class, runs queries returning shaped JS objects, and learns `async fn` exports along the way. | Use `sqlx` + SQLite (no infra needed). Introduces `#[neon::class]` because a connection pool wants a long-lived handle. Lead with `async fn` methods; the tutorial does **not** try to demonstrate `(async) impl Future` or `extract::with` — those have dedicated how-tos (#22, #23). Lead with the `tokio` feature flag (the shorter alias for `tokio-rt-multi-thread`, auto-inits a runtime); mention `set_global_executor` as a footnote linking to the *async-fn* how-to. Errors use `extract::Error`. The class has a `new` constructor that **always throws** with a message pointing at `connect`, plus a free `#[neon::export] async fn connect(path)` that opens the pool, runs schema setup, and returns a fully-initialised `Database` instance — this teaches constructing a JS class instance from Rust and demonstrates the idiomatic "async constructor" pattern (since `#[neon::class]` constructors can't be async). Cross-link to the *async-fn*, *classes*, *errors*, and *serde-json* how-tos. |
| 6 | `tutorials/publish-your-addon-to-npm.md` | Tutorial | Reader takes a working Neon addon and ships it to npm as a package that installs prebuilt binaries on every supported platform without requiring users to have a Rust toolchain. | **Promoted from how-to (#26 in earlier drafts) to a tutorial** because the work is genuinely tutorial-shaped: progressive, multi-step, ends with a published artifact, and impossible to do justice in 500–900 words. Walks through GitHub Actions matrix builds for Linux/macOS/Windows × x64/arm64, building per-platform `.node` binaries, packaging them as npm `optionalDependencies`, the `@neon-rs/load` runtime selector, npm `publish` workflow, and verifying installs work on a clean machine. Cross-link to the *Supported platforms* reference (for the matrix dimensions) and to the *Trade-offs* explanation (which already foreshadows this work as the "standard mitigation" for build complexity). No companion how-to — anyone wanting the recipe reads the tutorial. |

### Batch 3 — Explanation foundations

These come before the bulk of how-tos because how-tos lean on the
mental model these pages establish.

| # | Slug | Bucket | Goal | Notes |
|---|---|---|---|---|
| 7 | `explanation/what-is-neon.md` | Explanation | Reader understands what Neon is, what it competes with (NAPI, WASM, neon-bindings/n-api), and where it sits. | High-level intro. Mention Node-API. Compare/contrast WASM. |
| 8 | `explanation/trade-offs.md` | Explanation | Reader has an honest accounting of Neon's costs to weigh against the upsides covered in *What is Neon?*. | **Re-scoped from "When to use" to "Neon trade-offs"** — `what-is-neon.md` already covers Neon-vs-JS and Neon-vs-WASM positioning, so a second "should I use this?" page would duplicate it. This page is the cost ledger: per-call boundary overhead (only cross when the work warrants it); type choices that avoid serialization (typed arrays/buffers vs `Json<T>`); build complexity (Rust toolchain dep, cross-platform builds); distribution (per-platform `.node` files); debugging across the language boundary; supply-chain and binary-size implications; and a closing list of cases where the costs don't pay off. |
| 9 | `explanation/type-hierarchy.md` | Explanation | Reader understands the `Value` / `Object` trait hierarchy and how concrete `Js*` types relate. | Mermaid diagram. Reference `crates/neon/src/types_docs.rs` which already has prose. |
| 10 | `explanation/lifetimes.md` | Explanation | Reader understands why `Handle<'cx, T>` carries a lifetime, what it protects against, and how the same context-scoped borrow rules show up at runtime via `Lock` for buffer bytes. | **Title set to "Context lifetimes"** (broader than just handles — page also covers `Root<T>` and static-vs-runtime borrow checking on typed arrays). Slug kept as `lifetimes` for short, friendly URLs. Show the "use-after-scope" bug Rust rules out. |
| 11 | `explanation/threading-lifecycle.md` | Explanation | Reader understands `Channel`, `Root`, `Deferred`, and how they cooperate. | Diagram showing main thread → worker thread → main thread round-trip. |
| 12 | `explanation/error-handling.md` | Explanation | Reader understands the model: Rust `Result` ↔ JS exceptions, `extract::Error`, `try_catch`. | Pairs with the *errors* how-to; this page is the why, the how-to is the recipe. |
| 13 | `explanation/export-internals.md` | Explanation | Reader understands what `#[neon::export]` actually does at compile time. | Show generated code (cargo expand–style sketch). Talk about `EXPORTS`, `MAIN`, `linkme`. Aimed at curious power users; OK to be longer. |

### Batch 4 — How-to guides (core)

The reader-most-likely-to-need-it ordering, not alphabetical.

| # | Slug | Bucket | Goal | Notes |
|---|---|---|---|---|
| 14 | `how-to/common-types.md` | How-to | Reader can pass numbers, strings, arrays, objects, buffers across the boundary using `#[neon::export]`. | The "I just want to write a function" page. Cross-link to *serde-json*, *classes*, *streaming* for more advanced shapes. |
| 15 | `how-to/serde-json.md` | How-to | Reader can move structured data using `Json<T>` and the `json` shorthand. | Mention `serde` feature flag. |
| 16 | `how-to/errors.md` | How-to | Reader can throw, catch, and use `extract::Error` with `?`. | Pairs with explanation/error-handling. Recipe-style. |
| 17 | `how-to/cx-access.md` | How-to | Reader can reach `Cx` / `FunctionContext` from inside an exported function. | Show the `context` flavor of `#[neon::export]`. |
| 18 | `how-to/rename-exports.md` | How-to | Reader can give an exported function a JS name that differs from its Rust identifier. | Short page. `#[neon::export(name = "...")]`. Default snake → camel rule. |
| 19 | `how-to/this-methods.md` | How-to | Reader can write a Neon function that behaves as a method on a `this`. | `#[neon::export(this)]`. |
| 20 | `how-to/classes.md` | How-to | Reader can expose a Rust struct as a JS class with `#[neon::class]`. | Show the macro on `impl`, methods, the implicit `RefCell` wrap. |

### Batch 5 — How-to guides (async/threading)

| # | Slug | Bucket | Goal | Notes |
|---|---|---|---|---|
| 21 | `how-to/async-fn.md` | How-to | Reader can export an `async fn` that returns a Promise. | Both auto-init Tokio and explicit `set_global_executor`. Pairs with *async-tokio* tutorial; this page is the no-narrative recipe. **Home of form 1 of the async trio (`async fn`).** |
| 22 | `how-to/sync-setup-async.md` | How-to | Reader can do main-thread setup before async work, returning `impl Future` from an `#[neon::export(async)]` fn. | Already has a real code sample inherited from the homepage. Build the page around it. **Home of form 2 of the async trio (`(async) fn -> impl Future`).** Use the canonical motivator: rooting a JS callback or grabbing a `Channel` before spawning, so the future can call back into JS while running. |
| 23 | `how-to/main-thread-after-async.md` | How-to | Reader can hop back to the main thread with `extract::with` to build a JS-shaped result. | Cross-link the previous two pages. **Home of form 3 of the async trio (`extract::with`).** Show building a JS object with computed keys (e.g. column-keyed result rows) — the case where built-in `TryIntoJs` impls aren't enough. |
| 24 | `how-to/blocking-libuv.md` | How-to | Reader can offload sync work to the libuv pool with `#[neon::export(task)]`. | Recipe form of the *concurrency-libuv* tutorial. |
| 25 | `how-to/abort-controller.md` | How-to | Reader can wire a JS AbortController/AbortSignal into a Tokio `CancellationToken`. | **No first-class API exists.** Document the manual interop pattern from PR #104, with a clear note that this is application-level glue. |
| 26 | `how-to/streaming.md` | How-to | Reader can stream data between Rust and JS in either direction. | **No first-class API.** Document manual interop using callbacks / async iterators / Channel. May be the longest how-to since there's no shorthand. |

### Batch 6 — Reference, changelog, contributing

| # | Slug | Bucket | Goal | Notes |
|---|---|---|---|---|
| 27 | `reference/supported-platforms.md` | Reference | Reader can answer "does Neon work on X?" in <30 seconds. | OS × arch matrix, Node versions, NAPI feature flags, MSRV (1.65). |
| 28 | `reference/cli.md` | Reference | Reader has a complete reference for `create-neon` and `cargo-cp-artifact`. | Flags, args, exit codes. Source: `crates/create-neon/`, `crates/cargo-cp-artifact/`. |
| 29 | `changelog.md` | Reference | Reader sees latest releases with deep links to RELEASES.md. | Inline the most recent release notes; link out for full history. |
| 30 | `contributing.md` | Reference | Reader knows how to file an issue, send a PR, or join the Slack. | Slack invite, link to `CONTRIBUTING.md`, mention the doctest harness so doc PRs are easy. |

## 6. Title cleanup pass

Per "titles after content" decision, we re-read each finished page in
its section and propose any title changes that make titles sharper or
more parallel. Examples I already suspect we'll want:

- `cx-access.md` → "Get a `Cx` inside an exported function" (clearer
  than "Access Cx from an exported function")
- `sync-setup-async.md` → "Sync setup, then async work"
- `main-thread-after-async.md` → "Return to the main thread after `await`"

For each section as we finish it, we propose renames inline; you
approve; the renames roll up into the final
`chore(website): clean up titles` commit listed in §9.

## 7. Validation

Per the "per-page" decision: after writing each page, run

```bash
SKIP_RUSTDOC=1 cargo test --doc -p website
```

and the page must be green before moving on to the next.

After each batch, also run `SKIP_RUSTDOC=1 npm run build --workspace=@neon-rs/website`
to make sure Astro and Starlight are happy too (broken cross-links,
malformed frontmatter, etc.).

## 8. Banner discipline

While writing, the placeholder

```
:::caution[Status: skeleton]
This page is a placeholder. Content forthcoming.
:::
```

is replaced with a "draft" banner

```
:::note[Draft]
This page is a draft pending review.
:::
```

When the user has reviewed and approved a page, the draft banner is
removed entirely. The site never ships a page with no banner *and* no
real content.

## 9. Commit cadence

One commit per batch. Commit message format follows the existing
`feat(website): ...` prefix on this branch:

- `feat(website): write Getting Started pages`
- `feat(website): write Tutorials`
- `feat(website): write Explanation foundations`
- `feat(website): write How-to guides (core)`
- `feat(website): write How-to guides (async + threading)`
- `feat(website): write Reference, Changelog, and Contributing`
- `chore(website): clean up titles across docs sections` (final pass)

## 10. Out of scope

- New pages beyond the existing thirty. If during writing we discover a
  topic that needs its own page, we add a tracking note here and address
  it in a follow-up.
- Diagrams in non-Mermaid formats. Mermaid is enough for the type
  hierarchy and threading-lifecycle pages; anything more complex is
  out of scope for this pass.
- Reference auto-generation. The rustdoc API reference at `/api/` is
  already generated separately and is out of scope here.
- Translating any existing content from `neon-rs.dev`. The old site has
  some prose worth mining for inspiration, but we are writing fresh.
