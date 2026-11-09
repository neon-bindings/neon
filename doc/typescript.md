# TypeScript Declaration Generation for Neon

## Status

This is the agreed design, presented as a single coherent design rather than a
change-set, and the PR implements it. Type information for user data types comes
from third-party generators (via an adapter) rather than a Neon-owned derive. The
ts-rs adapter (`neon-ts-rs`) currently lives in-tree for end-to-end dogfooding and
will move to its own repository before release (see
[Crate & repository layout](#crate--repository-layout)).

## Motivation

Neon modules export Rust functions and classes to JavaScript via `#[neon::export]`.
Today, to give TypeScript consumers proper type declarations, a module author must
hand-write a `.d.ts` file (or a TypeScript wrapper) that redeclares every export
with its correct signature — tedious, error-prone, and a real barrier to a good
TypeScript experience.

The [tantivy-node](https://github.com/dherman/tantivy) project is a representative
example: a hand-written [wrapper](https://github.com/dherman/tantivy/blob/main/src/index.cts)
redeclares every function exported by the
[Rust addon](https://github.com/dherman/tantivy/blob/main/crates/tantivy-node/src/lib.rs).
Those raw addon declarations are entirely mechanical — they mirror the Rust
signatures with types translated to TypeScript — yet must be maintained by hand.

The goal is to **automatically generate `.d.ts` declarations** for a Neon module's
exports.

## Scope

A Neon module typically has two layers of TypeScript API:

1. **Raw addon declarations** — the flat functions, classes, and boxed types that
   `#[neon::export]` produces directly.
2. **Idiomatic wrapper API** — hand-authored types that wrap the raw addon into a
   pleasant surface.

Layer (2) is inherently authorial. **This feature targets layer (1)**: correct,
complete declarations for the raw exports. It's entirely mechanical, eliminates
the most tedious part of the workflow, and gives wrapper authors a typed
foundation.

## Design overview

The guiding principle is that **every declaration describes what actually happens
at runtime**, and where Neon can't know, it says `unknown` rather than guessing.

Every value that crosses a Neon export boundary goes through one of two
conversions, and the same Rust type can look different in each:

- **JSON** — `Json<T>`, or anything under `#[neon::export(json)]`. The value is
  serialized with serde, so its TypeScript type is whatever serde_json produces.
- **Bare** — everything else. The value goes through Neon's own `TryFromJs` (for
  arguments) or `TryIntoJs` (for return values).

| Rust type | through JSON | bare argument | bare return |
|---|---|---|---|
| `Vec<f64>` | `number[]` | `Float64Array` | `Float64Array` |
| `Option<T>` | `T \| null` | `T \| null \| undefined` | `T \| undefined` |
| `Arc<T>` | `T` (with serde's `rc` feature) | opaque box | opaque box |
| `()` | `null` | — | `undefined` (rendered `void`) |

Neither conversion changes; the two really do differ, and that's intentional (a bare
`Vec<f64>` *should* be a typed array). What Neon does is pick, for each argument and
return type, the TypeScript type of the conversion that actually runs there.

Responsibility for those types splits along the same line:

- **Neon owns the bare conversions and the export→declaration machinery.** Its
  conversions are sealed — every one is defined by Neon — so Neon knows the exact
  TypeScript type of each. The `#[neon::export]` / class macros collect type
  metadata at compile time; at runtime, `generate()` walks it and renders a `.d.ts`.

- **The JSON shape of user data types (and other foreign types) comes from a
  third-party generator** (ts-rs or specta), reached through a thin *adapter*. Neon
  does **not** own a serde-aware derive macro. Understanding serde's serialization
  (renames, enum tagging, flatten, optionality, generics) is an open-ended
  maintenance treadmill that tracks the evolution of serde and Rust; a mature crate
  like ts-rs already owns it. See [Design rationale](#design-rationale) for why.

The crate picture:

```
neon-typescript          the JSON contract: the TsJson trait + built-in impls
  (no third-party deps)   for std/core types. Rarely changes.
     ▲            ▲
     │            │
   neon        neon-ts-rs / neon-specta     the adapters
(TsFromJs, TsIntoJs,
 the macros, generate())
```

- **`neon-typescript`** — the `TsJson` trait (the JSON shape) and its impls for
  std/core types. No dependency on serde, ts-rs, specta, or `neon`. The contract the
  adapters bind to; deliberately minimal and semver-stable.
- **`neon`** — depends on `neon-typescript`. Defines `TsFromJs` / `TsIntoJs` (the
  bare shapes) next to its conversions, plus `generate()`, auto-attach, and the
  export/class metadata macros.
- **`neon-ts-rs` / `neon-specta`** — adapters. Each depends on `neon-typescript` and
  its upstream (ts-rs / specta) — **not** on `neon`. Versioned independently to track
  the upstream. See [Crate & repository layout](#crate--repository-layout).

## Architecture

### Three traits, one per conversion

Each trait is named after the Neon conversion it describes:

| Conversion that runs | TypeScript trait | Lives in |
|---|---|---|
| serde_json, via `Json<T>` | `TsJson` | `neon-typescript` |
| `TryFromJs` (bare arguments) | `TsFromJs` | `neon` |
| `TryIntoJs` (bare returns) | `TsIntoJs` | `neon` |

All three are deliberately minimal and string-based:

```rust
// neon-typescript
pub trait TsJson {
    /// The TypeScript type *expression*: "number", "string[]", "SearchResult", …
    fn ts_json() -> Cow<'static, str>;

    /// An optional top-level declaration this type needs (an interface or type
    /// alias). `None` for primitives.
    fn ts_json_decl() -> Option<Cow<'static, str>> { None }

    /// Add this type's own declaration plus its transitive declarations to
    /// `decls`, keyed by name for dedup.
    fn ts_json_collect(decls: &mut BTreeMap<String, String>) { /* add ts_json_decl */ }
}

// neon — sealed with the same trait as `TryFromJs` / `TryIntoJs`
pub trait TsFromJs: Sealed {
    fn ts_from_js() -> Cow<'static, str>;
    fn ts_from_js_collect(_decls: &mut BTreeMap<String, String>) {}
}
pub trait TsIntoJs: Sealed {
    fn ts_into_js() -> Cow<'static, str>;
    fn ts_into_js_collect(_decls: &mut BTreeMap<String, String>) {}
}
```

No derive, no serde parsing, no AST lives in any of them. (A structured-AST output is
deferred; see [Alternatives considered §1](#alternatives-considered).) The methods
have distinct names so a type implementing several traits (`String` implements all
three) is never ambiguous.

**Why the traits live where they do.** `TsJson` is what the adapters implement, and
adapters depend only on `neon-typescript` so they can version with their generator;
its std impls (`Option`, `Vec`, maps, tuples) can only be written in the trait's own
crate. `TsFromJs` / `TsIntoJs` describe Neon's own conversions: their impls need
Neon's types and traits (`Binary`, `Handle`, `Boxed`), they're sealed with the same
`Sealed` trait as the conversions, and the orphan rule only allows
`impl<T: Binary> TsFromJs for Vec<T>` if the trait is local to `neon`. Adapters never
touch them.

**The bare side is split by direction; the JSON side isn't.** A bare conversion's
input and output really differ — `TryFromJs` accepts `null` or `undefined` for
`None`, while `TryIntoJs` produces `undefined` — and Neon knows both exactly, so it
describes both. serde can also accept more than it emits (`#[serde(default)]`,
aliases, missing `Option` fields), but those differences come from the user's serde
attributes, which only the generator sees, and generators produce a single type. Using
the serialized shape for an input is stricter than serde rather than looser: it may
reject some valid input, but never accepts invalid input. See
[Alternatives considered §9](#alternatives-considered).

**Transitivity is decentralized.** There is no central tree-walk. Each composite
type's `*_collect` adds its own declaration and then collects from its component
types; the transitive walk emerges from that call chain. `generate()` seeds the walk
from each export's parameter and return types, and the `BTreeMap` keyed by type name
dedups declarations referenced from multiple places.

### Which conversion runs at a position

For each argument and return type, the macro decides which trait to consult:

1. **Peel control flow.** A returned `Result<T, E>` is not data: the `Err` is thrown,
   so only `T` is observable. The macro looks through it (including one-parameter
   aliases like `anyhow::Result<T>` and `NeonResult<T>`). Async and task exports wrap
   the result in `Promise<…>`.
2. **JSON positions** — the item is `#[neon::export(json)]`, or the type is a
   top-level `Json<T>`. The payload `T` is described with `TsJson` (and the adapter's
   `TsJsonExt`, if imported; see below).
3. **Bare positions** — everything else: `TsFromJs` for arguments (looking through
   a reference such as `&Class`, which is extracted from the referent's JS value),
   `TsIntoJs` for returns and static properties.

A function or method with no declared return type is `void` on the bare path and
`null` on the JSON path (serde_json serializes the implicit `()` to `null`).

### Where type information comes from

**JSON shapes (`TsJson`):**

1. **Built-in impls in `neon-typescript`**, matching serde_json:

   | Rust type | `ts_json()` |
   |---|---|
   | `f64`, `i32`, `u64`, `u8`, … | `number` (serde_json emits every number as a JSON number) |
   | `String`, `&str` | `string` |
   | `bool` | `boolean` |
   | `()` | `null` |
   | `Option<T>` | `T \| null` |
   | `Vec<T>`, `HashSet<T>`, `BTreeSet<T>` | `T[]` |
   | `HashMap<K, V>`, `BTreeMap<K, V>` | `Record<string, V>` |
   | `(A, B)`, `(A, B, C)` | `[A, B]`, `[A, B, C]` |
   | `Box<T>`, `Arc<T>`, `Rc<T>`, `RefCell<T>`, `&T` | `T` (serialized transparently) |
   | `Result<T, E>` (as data) | `{ Ok: T } \| { Err: E }` |

   `HashMap` maps to `Record<string, V>` regardless of `K`, because JSON object keys
   are always strings. `Option<T>` is `T | null` because serde serializes `None` as a
   present `null` — see [Alternatives considered §2](#alternatives-considered).

2. **User data types**, via an adapter: the user derives the upstream generator's
   trait plus a trivial *bridge derive* that implements `TsJson` by delegating to it.
   See [Type providers](#type-providers).

3. **Foreign types** (e.g. `IndexMap`, `chrono::DateTime`) used directly as a JSON
   payload: described by the generator when the adapter's `TsJsonExt` is in scope
   (see [Type providers](#type-providers)). Neon owns no per-crate impls.

4. **Hand-written impls** for the rare user who uses no generator crate: a small
   `impl TsJson` (just `ts_json` + `ts_json_decl`). Neon is intentionally out of the
   derive business.

**Bare shapes (`TsFromJs` / `TsIntoJs`)** are implemented in `neon`, next to each
conversion:

| Rust type | argument (`TsFromJs`) | return (`TsIntoJs`) |
|---|---|---|
| `f64`, `i32`, `u32` (and more numbers for returns) | `number` | `number` |
| `String` (`&str`, `&String` for returns) | `string` | `string` |
| `bool` | `boolean` | `boolean` |
| `()` | `unknown` (extraction ignores the argument) | `undefined` |
| `Option<T>` | `T \| null \| undefined` | `T \| undefined` |
| `Result<T, E>` | — | `T` (the `Err` is thrown) |
| `Either<L, R>` | `L \| R` | `L \| R` |
| `Vec<T>` (`[T; N]`, `&[T]` for returns), `T: Binary` | the typed array (`Float64Array`, `BigInt64Array`, …) | same |
| `Array<T>` | `Item[]` (a native JS array) | same |
| `ArrayBuffer<B>`, `Buffer<B>`, typed-array wrappers | `ArrayBuffer`, `Buffer`, `Uint8Array`, … | same |
| `Handle<JsX>`, `Root<JsX>` | the JS type (`string`, `object`, `Date`, `bigint`, …; `JsValue` is `unknown`) | same |
| `Boxed<T>`, `Arc<T>`, `Rc<T>`, `RefCell<T>`, `Ref<T>`, `RefMut<T>` | a branded box (see [Boxed types](#boxed-types)) | same |
| `Json<T>` (nested inside another bare type) | `T`'s JSON shape | same |
| a class | the class name | the class name |

Because the conversions are sealed, this table is closed: every bare conversion has a
shape, and a test can check that none is missing.

### Graceful fallback: `unknown`

Enabling the `typescript` feature must never force a TypeScript impl on every type in
every export — a user typing part of their API should not be blocked by an export
that uses an opaque or un-annotated type. A type with no known TypeScript type is
declared **`unknown`** (not `any`): an `unknown` return must be narrowed before it can
be used, while `any` silently switches off type checking for everything it touches.
The same applies to types the macro can't name in a static context (`impl Trait`,
unsized slices). Being lenient by default is a deliberate, permanent commitment — see
[Alternatives considered §11](#alternatives-considered).

> The "probe" and "rung" vocabulary below is internal mechanism, for maintainers.
> Users never encounter it: the entire user-facing surface is one import,
> `use neon_ts_rs::TsJsonExt as _;`, explained in plain terms under
> [Type providers](#type-providers).

The fallback is achieved with **autoref-specialization probes** in the
macro-generated metadata, so the macros never emit a hard trait bound. There is one
probe per conversion. The JSON probe is a **three-rung ladder**, tried most-specific
first:

```rust
// In neon-typescript. (It lives here, not in `neon`, so that an adapter crate can
// hang its own rung on it; see below.)
pub struct TsJsonProbe<T>(pub PhantomData<T>);

// Rung 1 (most specific): inherent method, available only when T: TsJson.
impl<T: TsJson> TsJsonProbe<T> {
    pub fn ts_json_of(&self) -> Cow<'static, str> { T::ts_json() }
}

// Rung 3 (fallback): reached via autoref for ALL T.
pub trait TsJsonFallback { fn ts_json_of(&self) -> Cow<'static, str>; }
impl<T> TsJsonFallback for &TsJsonProbe<T> {
    fn ts_json_of(&self) -> Cow<'static, str> { "unknown".into() }
}
```

**Rung 1** — the type has a `TsJson` impl (a built-in, a user type via the bridge
derive, or a hand-written impl). **Rung 3** — nothing matches; the type becomes
`unknown`, with no viral bounds and no compile error.

**Rung 2 (the adapter rung)** sits between them and is what lets a *foreign* payload
resolve through the generator instead of degrading to `unknown`. It is provided by the
adapter, because the orphan rule bars anyone but `neon-typescript` from implementing
`TsJson` for a foreign type — and `neon-typescript` can't name `ts_rs::TS`. So the
adapter defines its own trait and implements it for the *bare* probe:

```rust
// In neon-ts-rs. A new, adapter-local trait, so implementing it for a foreign
// type (TsJsonProbe is defined in neon-typescript) is orphan-legal ("you can
// always implement your own trait").
pub trait TsJsonExt { fn ts_json_of(&self) -> Cow<'static, str>; }
impl<T: ts_rs::TS> TsJsonExt for TsJsonProbe<T> {
    fn ts_json_of(&self) -> Cow<'static, str> { neon_ts_rs::ts_json::<T>() }
}
```

The `&`-levels are what order the rungs. The call site is `(&__probe).ts_json_of()`,
so the receiver has type `&TsJsonProbe<T>`. Rung 2's method receiver is *also*
`&TsJsonProbe<T>` (`&self` on `impl … for TsJsonProbe<T>`), the same candidate as
rung 1's inherent method; since inherent methods outrank trait methods at the same
candidate, rung 1 wins whenever it applies and rung 2 catches the rest. Rung 3's
receiver is one autoref deeper (`&&TsJsonProbe<T>`, from `impl … for
&TsJsonProbe<T>`), so it is reached only after both.

The bare probes (`TsFromJsProbe`, `TsIntoJsProbe`, in `neon`) have the same shape
with **two rungs**: the conversion's shape, then `unknown`. There's no adapter rung,
because generators only describe serde shapes, which are never what a bare
conversion produces. Since every bare conversion has a shape, their fallback is only a
safety net.

**Looking through `Json`.** Rung 2 fires on the foreign type itself:
`Json<Foreign>: ts_rs::TS` is false and unfixable (the orphan rule bars anyone from
implementing ts-rs's trait for Neon's `Json`). So the macro instantiates the JSON
probe with the payload `T`, not the `Json<T>` wrapper. This is also why a returned
`Result` is peeled by the macro rather than by a trait impl: inside a generic impl
there's nothing to probe — it would need a `T: TsJson` bound, which excludes exactly
the foreign types rung 2 exists for. The same constraint explains the one known gap:
a `Json` *nested* inside a bare wrapper (e.g. `Option<Json<IndexMap<…>>>`) is
described by `impl<T: TsJson> TsFromJs for Json<T>`, which can't consult the adapter,
so a foreign payload there falls back to `unknown`. Workarounds: wrap the payload in
a newtype that derives `TsJson`, or set the type with `#[neon(ts_type = "...")]`.

**Opting the adapter rung into scope.** Autoref specialization dispatches on an
*unqualified* method call, and for a *trait* method that only resolves if the trait is
`use`d in scope. Rung 1 is an inherent method (found via the fully-qualified probe
path — no import), and rung 3's fallback lives in a crate the macro knows, so the
macro brings it into scope itself (`use … as _;`). It cannot do that for rung 2's
trait, because it doesn't know which adapter the user chose. So the user brings it
in, once per module whose exports carry foreign JSON payloads:

```rust
use neon_ts_rs::TsJsonExt as _;
```

This is the ordinary anonymous-extension-trait idiom (`use std::io::Write as _;`) —
the name never appears anywhere but the import, and the methods are consumed only by
macro-generated code. If the import is absent (or the user uses no adapter),
resolution simply skips rung 2: real type or `unknown`.

### Metadata collection

Behind the `typescript` feature, the `#[neon::export]` / class macros emit a
metadata entry per export into a `linkme::distributed_slice`, matching the existing
`EXPORTS` pattern:

```rust
#[linkme::distributed_slice(TYPE_METADATA)]
static __META_search: ExportMeta = ExportMeta::Function(FunctionMeta {
    name: "search",
    params: &[ParamMeta {
        name: "query",
        // closures, resolved at runtime through the position's probe
        ts_type: || /* probe over the param type */,
        ts_collect: |decls| /* probe collect */,
    }],
    ret_type: || /* probe over the return type */,
    ret_collect: |decls| /* … */,
    is_async: false,
});
```

Using closures that resolve types at runtime (rather than baked-in strings) lets us
lean on Rust's trait resolution for delegation and generics. `generate()` walks
`TYPE_METADATA`, resolves each type, collects transitive declarations, and renders.

The metadata types (`ExportMeta`, `FunctionMeta`, `ParamMeta`, …) live in
`neon::macro_internal`: they're an implementation detail shared by the macros and the
generator, **not** public API. Macro expansions construct them with struct literals
inside user crates, so they could never be marked `#[non_exhaustive]`; keeping them
internal is what lets future work add fields (for example, source locations for
[generation-time reporting](#follow-up-work)) without a breaking change. This is
safe because `neon` pins `neon-macros` to an exact version, so the macros and the
types always change together.

Emission is gated so builds without the feature pay nothing: `neon`'s `typescript`
feature enables `neon-macros/typescript`, and the macros gate metadata emission on
`cfg!(feature = "typescript")` (resolved when `neon-macros` compiles; Cargo unifies
the feature across the build). With the feature off, `#[neon::export]` emits no
metadata statics at all.

### Boxed types

Several bare conversions store a Rust value in a `JsBox<P>`, which appears on the JS
side as an opaque value. Two boxes are interchangeable at runtime only if they hold
the same `P` (the downcast is by type), so the generated TypeScript is a *branded
interface* keyed on `P`:

```typescript
export declare const __neon_tag: unique symbol;
export interface BoxedArcDatabase { readonly [__neon_tag]: 'Arc<Database>' }
```

The payload `P` depends on the conversion: `Boxed<T>` boxes `T`, while a bare
`Arc<T>` boxes `Arc<T>` itself. So `Boxed<Database>` is `BoxedDatabase` and
`Arc<Database>` is `BoxedArcDatabase` — correctly distinct, since passing one where
the other is expected throws — while `Boxed<Arc<Database>>` and `Arc<Database>` share
a brand, since they produce the same `JsBox<Arc<Database>>`. `&RefCell<T>`, `Ref<T>`,
and `RefMut<T>` arguments borrow from a `JsBox<RefCell<T>>`, so they share the brand
of a returned `RefCell<T>`.

The brand string is `std::any::type_name::<P>()` with module paths stripped, and the
interface name is `Boxed` plus that string reduced to an identifier. Boxed types need
no TypeScript impl of their own. Two different types with the same name in different
modules would share a brand (see [Stability](#stability)).

### Class exports

`#[neon::export(class)]` / `#[neon::class]` know the full class structure at compile
time — constructor, methods (with receiver, attributes, return type), and const
properties — enough to emit a complete `declare class`. A class like:

```rust
pub struct Point { x: f64, y: f64 }

#[neon::export(class)]
impl Point {
    #[neon(name = "maxCoordinate")]
    const MAX_COORD: f64 = 1000.0;
    pub fn new(x: f64, y: f64) -> Self { /* … */ }
    pub fn distance(&self, other: &Self) -> f64 { /* … */ }
    pub fn translate(&mut self, dx: f64, dy: f64) { /* … */ }
    #[neon(task)]
    pub fn heavy_computation(self) -> f64 { /* … */ }
}
```

generates:

```typescript
export declare class Point {
    constructor(x: number, y: number);
    static readonly maxCoordinate: number;
    distance(other: Point): number;
    translate(dx: number, dy: number): void;
    heavyComputation(): Promise<number>;
}
```

Notable rules: `#[neon(context)]` / `#[neon(this)]` parameters are internal plumbing
and hidden; the receiver (`&self` / `&mut self` / `self`) is invisible in TS; `async
fn` and `#[neon(task)]` methods return `Promise<T>`; a fallible constructor
(`Result<Self, E>`) drops the error (it becomes a throw); reference parameters
(`&OtherClass`) are typed as the class; const properties become `static readonly`.
Class members follow the same position rules as functions (methods marked
`#[neon(json)]` use JSON shapes).

A class instance crosses the boundary through the class's own bare conversions, so
the class macro implements `TsFromJs` and `TsIntoJs` for the class type, both
returning the class name. Classes therefore appear by name when used as parameter or
return types in other exports (not as opaque boxes).

### Generation and output

`neon::typescript` exposes:

```rust
/// Render a complete `.d.ts` string.
pub fn generate() -> String;
```

(A structured-AST API, `generate_ast()`, is deferred — see
[Alternatives considered §1](#alternatives-considered).)

The output is a `.d.ts` intended to sit beside the `.node` binary. Top-level items
are named `export declare` statements (with `export interface` / `export type` for
collected declarations):

```typescript
// Auto-generated by Neon. Do not edit.

export declare const __neon_tag: unique symbol;
export interface BoxedArcSchema { readonly [__neon_tag]: 'Arc<Schema>' }
export type SearchResult = { /* … */ };

export declare function newSchema(schema: SchemaDescriptor): BoxedArcSchema;
export declare function commit(index: BoxedArcIndex): Promise<void>;

export declare class Point { /* … */ }
```

**Auto-attach.** When the `typescript` feature is on, Neon attaches the generated
declarations to the addon's module exports under `Symbol.for("neon:types")` during
module init, so a small Node script can extract them without a Rust-side shim:

```js
const addon = require("./index.node");
require("fs").writeFileSync("index.d.ts", addon[Symbol.for("neon:types")]);
```

Auto-attach runs before user-defined exports, so it works even with a custom
`#[neon::main]`. Module scoping (wrapping the output in `declare module "…" { … }`
for addons loaded through a shim) is out of scope for this iteration; a consumer who
needs it can wrap the flat output themselves.

### Escape-hatch attributes

The codegen handles common cases; a few per-item overrides cover the ragged edges:

- `#[neon(ts_skip)]` — exclude an item from the `.d.ts`. Works on functions,
  classes, methods, static properties, and the `fn new` constructor. Skipping a
  class omits its `declare class`, but exports that reference it still name it, so
  skip only classes no other export mentions.
- `#[neon(ts_name = "...")]` — rename in TS output without changing the Rust type
  or JS export name.
- `#[neon::export(ts_returns = "...")]` (also on methods) — override the inferred
  return type with a literal TS string (e.g. for a type Neon can't infer).
- `#[neon(ts_type = "...")]` on an individual function/method parameter — override
  that parameter's inferred type.
- `#[neon::export(class, ts_no_constructor)]` — emit a `declare class` with no
  `constructor`, for classes only constructed via factory methods elsewhere.

A deliberate `unknown` is written with these overrides (`ts_returns = "unknown"`),
which keeps it distinguishable from a type Neon simply couldn't describe.

### Feature-gating

TypeScript generation is behind a `typescript` feature on `neon`:

```toml
neon = { version = "...", features = ["typescript"] }
```

When off (the default), no metadata is emitted, no extra dependencies are pulled,
and there is zero impact on existing users (verified across the napi feature matrix).

## Type providers

User data types (the structs and enums carried across the boundary by `Json<T>`)
get their JSON shape from a third-party generator, reached through an adapter.

### The bridge derive

Rust's orphan rule forbids a blanket `impl<T: ts_rs::TS> TsJson for T` in an adapter
crate (and it would collide with the built-in impls). But the *user owns their type*,
so the adapter ships a trivial derive that generates the bridge impl in the user's
crate:

```rust
// neon-ts-rs generates, for the user's MyType:
impl neon_typescript::TsJson for MyType {
    fn ts_json() -> Cow<'static, str> { neon_ts_rs::ts_json::<Self>() }
    fn ts_json_collect(decls: &mut BTreeMap<String, String>) {
        neon_ts_rs::ts_json_collect::<Self>(decls);
    }
}
```

`neon_ts_rs::ts_json` / `ts_json_collect` delegate to `<Self as ts_rs::TS>`,
assembling a flat declaration set via ts-rs's `TypeVisitor`. This bridge derive is
*trivial*: it reads none of the type's fields or serde attributes — it emits a fixed
delegation. All serde understanding stays in ts-rs. The generated impl names the trait
through a hidden re-export in the adapter (`::neon_ts_rs::__private::TsJson`) and
otherwise uses only fully-qualified `::std` / `::ts_rs` paths, so a consumer depends
on just `neon` + `neon-ts-rs` (+ `ts-rs`) — never on `neon-typescript` directly, which
stays a pure implementation detail. (This is the serde-derive convention: the derive
references its own crate's re-exports, not the underlying contract crate.)

### What the user writes

```rust
// ts-rs path: one extra derive, nothing at the export boundary.
#[derive(Serialize, Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
#[serde(rename_all = "camelCase")]
struct SearchResult { doc_id: u32, /* … */ }

#[neon::export(json)]
fn search(q: String) -> SearchResult { /* … */ }   // Json extraction unchanged
```

- **ts-rs user** — derive `ts_rs::TS` + `neon_ts_rs::TsJson`; the types are also
  visible to any ts-rs tooling.
- **specta user** — derive `specta::Type` + `neon_specta::TsJson`; the types plug
  into the specta/Tauri ecosystem, with the (less stable) specta dependency
  quarantined in its adapter crate.
- **neither** — hand-write `impl TsJson` (see
  [Where type information comes from](#where-type-information-comes-from)).

Different types in one signature may use different adapters: each derive produces its
own `TsJson` impl, and Neon consumes only `TsJson`. (A single type can't use two
bridge derives, since that would be two impls of `TsJson` for one type.)

Nothing above is needed as long as the payload has a `TsJson` impl (a user type via
the bridge, a std type). The one case that needs more is a **foreign type used
directly as a JSON payload** — e.g. returning `Json<IndexMap<String, Field>>` rather
than wrapping it in a named struct. There, import `TsJsonExt` in that module — it lets
ts-rs describe foreign types in your JSON payloads, so they're typed instead of
falling back to `unknown`:

```rust
// Let ts-rs describe foreign types (e.g. IndexMap) in JSON payloads.
use neon_ts_rs::TsJsonExt as _;   // once per module that exports such types

#[neon::export]
fn fields(&self) -> Json<IndexMap<String, FieldDescriptor>> { /* … */ }
```

The `IndexMap` then renders through ts-rs (enable ts-rs's own `indexmap-impl`
feature) at your pinned ts-rs version, and the types reachable through it stay in
the output.

**Its reach is exactly the generator's type coverage.** `TsJsonExt` only covers a
foreign type that ts-rs itself implements; a foreign type ts-rs doesn't know still
types as `unknown`. In practice, reach for a generator-supported type: e.g. ts-rs 12
ships `indexmap-impl` but has no `ordermap` impl, so `IndexMap` works while
`ordermap::OrderMap` (insertion-ordered identically, and a thin layer over `IndexMap`)
does not. See [Graceful fallback](#graceful-fallback-unknown) for the mechanism, and
[Alternatives considered §4–5](#alternatives-considered) for why this beats
Neon-owned per-crate impls or a per-boundary wrapper.

### Adapter responsibilities

The generators are faithful to the *Rust type*; the JSON boundary is faithful to
*what serde_json serializes*. These differ in essentially one fixed place, so the
adapter carries a tiny bit of reconciliation:

- Configure large ints to `number` (ts-rs defaults `u64`/`i64` to `bigint`, but
  serde_json emits JSON numbers). Fixable via ts-rs `Config::with_large_int`.

That is nearly the whole list. The adapter deliberately does **not** add `Option`
leniency (optionality is user-expressible via serde — see
[Alternatives considered §2](#alternatives-considered)) and does **not** normalize
output styling (bridged types render in their provider's idiom —
[§3](#alternatives-considered)). This is a *fixed* surface: it does not grow when
serde adds an attribute.

### Serde fidelity is the generator's concern

Because a user's types are described by ts-rs / specta, matching serde's actual
serialization is those crates' job, not Neon's. In particular, **optionality is
expressed with standard serde attributes.** ts-rs marks a field TypeScript-optional
(`f?`) exactly when it can be absent in both directions — i.e. with
`#[serde(skip_serializing_if = "Option::is_none", default)]`, which yields
`f?: T | null` (accepts a value, `null`, `undefined`, or omission). A plain
`Option<T>` renders as `T | null` (present, maybe null). Neon owns no optionality
policy.

## Crate & repository layout

- **`neon-typescript`** lives in the main neon monorepo — part of neon's stable
  contract, evolving with neon.
- The **adapter crates** live in a **separate repository**, versioned
  **independently**, each tracking its upstream's major (`neon-ts-rs 12.x` for ts-rs
  12; a new major when ts-rs 13 breaks the `TS` trait). This keeps neon core from
  moving for an upstream bump, isolates upstream churn (especially specta's
  release-candidate cadence) from neon's repo and CI, and keeps ts-rs/specta out of
  neon's build graph. It matches `tauri-specta`, which is its own repo distinct from
  both specta and tauri.
- **Hard rule:** adapters version and release on their own line — never lockstepped
  to neon, regardless of repo layout.

**For this PR:** an initial `neon-ts-rs` ships *in-tree, temporarily*, so the
dogfooding PR ([dherman/tantivy#3](https://github.com/dherman/tantivy/pull/3)) can
consume it via a **git dependency** for full end-to-end validation. It is extracted
to its own repository and published before release; the git dependency then becomes
a normal versioned dependency. `neon-specta` is not part of this PR and starts in
the separate repo.

## Stability

The **Rust API** — the `TsJson`, `TsFromJs`, and `TsIntoJs` traits, `generate()`,
and the macro attributes — follows the normal semver guarantees of the crate that
defines it (as do `neon-ts-rs`'s `TsJson` derive and `TsJsonExt`, on that crate's own
version line). The metadata types in `neon::macro_internal` are
**not** part of the API and may change in any release.

**Lenient by default is a commitment.** A type with no known TypeScript type is
declared `unknown`, and that will stay the default: making missing types a compile
error by default could break builds that never opted into TypeScript (see
[Alternatives considered §11](#alternatives-considered)).

The **generated output format** — the exact text of the `.d.ts` — is **not yet
covered by semver** and may change while the feature settles. In particular, box
brand names are derived from `std::any::type_name`, whose format Rust doesn't
guarantee, and two same-named types from different modules currently share a brand.
If you commit generated declarations to source control, expect to regenerate them
across Neon upgrades until the format is declared stable.

Async exports (`#[neon::export(task)]`, `async fn`) render as `Promise<T>`; a unit
return normalizes to `Promise<void>`.

## Follow-up work

- **Generation-time reporting of `unknown`s.** Record each position that fell back
  to `unknown` (the export, the position, and the Rust type) and report them when the
  `.d.ts` is generated, with file/line and targeted advice (e.g. "add
  `use neon_ts_rs::TsJsonExt as _;`"); a `--strict` mode would make them fatal in CI.
  This needs no breaking changes, because the metadata types are internal and the
  fallback is Neon-owned. See [Alternatives considered §11](#alternatives-considered).
- **Extract `neon-ts-rs`** to its own repository and publish it before release.

---

## Design rationale

### Why Neon doesn't own a serde-aware derive

A Neon-owned derive that understands serde attributes (`rename_all`,
`tag`/`content`/`untagged`, `flatten`, `skip`, `default`, `transparent`, generics) is
the single largest and most volatile part of this feature. Keeping it correct is an
**open-ended maintenance treadmill** that tracks the evolution of serde and Rust —
and it is tangential to Neon's mission. A mature, widely-used crate (ts-rs) already
owns that surface. By owning only minimal string traits and sourcing user-type
information from such a crate through a version-isolated adapter, Neon sheds the
treadmill while still giving users the same result with one extra derive.

### Prior art and ecosystem research

Three crates generate TypeScript from Rust:

- [**specta**](https://github.com/specta-rs/specta) — a language-agnostic type
  introspection system with an intermediate representation, integrated with the
  Tauri/rspc ecosystem. It has the ecosystem, but **no dependable release**: there
  is no stable 2.0 (the ecosystem runs on exact-pinned `2.0.0-rc.*` release
  candidates that break between RCs; adoption is fragmented across RCs), and stable
  1.x is ~3 years old and effectively legacy. Not something to put in a foundational
  crate's public API today.
- [**ts-rs**](https://github.com/Aleph-Alpha/ts-rs) — a focused, TypeScript-only
  generator. The opposite profile: **healthy and semver-normal** (~13M downloads,
  real shipping majors, a major roughly every 4–9 months, multi-contributor), so it
  can be `^`-pinned — but a *standalone generator* with essentially **no consumer
  ecosystem** to interoperate with.
- [**typeshare**](https://github.com/1Password/typeshare) — source-level parsing via
  a CLI; multi-language, but its static analysis can't resolve types through macros
  or trait impls.

So there is no external crate that offers *both* a healthy project and a valuable
ecosystem. The design accommodates this by supporting **multiple adapters**: ts-rs
as the healthy default, specta for users who want its ecosystem (with its instability
quarantined in the adapter crate), and hand-written impls for anyone using neither.

The autoref-specialization probe is an established stable-Rust idiom: `anyhow` and
`eyre` use the same technique in their macros to dispatch on whether an argument
implements `std::error::Error`.

A spike (real ts-rs 12) validated the adapter shape end-to-end (~40 lines of
adapter) and compared ts-rs's output to serde_json's actual serialization:

| Feature | serde_json runtime | ts-rs type | Verdict |
|---|---|---|---|
| `rename_all = "camelCase"` | `docId` | `docId` | match |
| `#[serde(skip)]` | field omitted | field omitted | match |
| internally-tagged enum | `{"kind":"circle",…}` | `{ "kind": "circle", … }` | match |
| `Vec<String>` | `["a"]` | `Array<string>` | match (cosmetic) |
| `u64` | `42` (JSON number) | `bigint` (default) | mismatch — configurable |
| `Option<u32>` | key present, `null` | `number \| null` | accepted (leniency via serde) |

### Alternatives considered

**1. Structured AST — string-only traits, and `generate_ast()` deferred.**
The feature could also emit a structured, TSESTree-shaped AST (for programmatic
transforms). *Decision:* the traits are string-only, and `generate_ast()` plus the
string→AST parser that would back it are **deferred**; this iteration ships the
`.d.ts` string only. *Rejected:* putting an AST method in the traits — it would drag
the AST node types and parser into `neon-typescript`, inflating the crate we most
want minimal, and bridged types are parser-sourced regardless (so built-ins being
specially native buys inconsistency, not fidelity). *Why defer at all:* the
dogfooding consumes only the string (`Symbol.for("neon:types")`) and never touches
the AST, so shipping an unused, semver-affecting API (and its parser) from day one is
a cost with no current payoff. If a concrete consumer materializes, the AST can be
added later, produced in `neon` by parsing the strings.

**2. `Option` leniency in JSON — user-expressible via serde, not adapter policy.**
serde's default `Option<T>` serializes `None` to a present `null` (`T | null`), while
deserialization leniently accepts absent/`null`/`undefined` → `None`. *Decision:* the
adapter owns no leniency policy; a user expresses an optional field with
`#[serde(skip_serializing_if = "Option::is_none", default)]`, which ts-rs renders as
`f?: T | null` — fully lenient and accurate in both directions. The built-in JSON
`Option<T>` matches this at `T | null` for consistency. *Rejected:* post-processing
every `Option` to be lenient (bakes a uniform policy that's loose for return types,
reintroduces an output-format transform in the adapter, and can't even distinguish
`Option` from other nullables once ts-rs has stringified it). *Consequence:* JSON
leniency is opt-in — the honest default for a plain `Option<T>` is `T | null`. (A
*bare* `Option<T>` is a different conversion; see §7.)

**3. Bridged output styling — accept each provider's idiom.** ts-rs renders
semantically-identical TypeScript in a different style than Neon's built-ins
(`Array<string>` vs `string[]`, quoted discriminant keys). *Decision:* do not
normalize; bridged types render in their provider's style, mixing with Neon's own
style for built-in types. *Rejected:* normalizing (fragile string-rewriting that
reintroduces the "understand and rewrite the output format" ownership we're shedding)
and adopting the provider's style wholesale (impossible — Neon's own types stay
Neon-styled). The cost is cosmetic inconsistency across a file, consistent within any
one type.

**4. Foreign JSON payloads — an adapter rung, not Neon-owned batteries or a
per-boundary wrapper.** A foreign, non-std type used directly as a JSON payload (e.g.
`Json<IndexMap<String, Field>>`) has no `TsJson` impl: the orphan rule bars both the
adapter and the user from writing one, and the bridge derive reaches only user-owned
types. Left alone it falls back — and because collection recurses only through
*typed* components, every type reachable *only* through it silently vanishes from the
output, turning graceful degradation into a downstream `tsc` break located far from
its cause. *Decision:* close it with the adapter-provided `TsJsonExt` rung (see
[Graceful fallback](#graceful-fallback-unknown)), which resolves any `T: ts_rs::TS`
through the generator at the user's own version. *Rejected — feature-gated built-in
impls in `neon-typescript`* (`typescript-indexmap`, `typescript-chrono`, …): the
trait's crate is the only orphan-legal home for such a generic impl, but putting it
there re-couples the foreign crate's *version* to `neon-typescript`'s declared range
and silently fails to apply across a semver-major bump (the impl is then for a
different type) — exactly the per-crate maintenance surface the adapter design sheds,
reintroduced one crate at a time. *Rejected — an explicit per-boundary wrapper* (a
generic `TsRs<T>` the user wraps around foreign types in each signature): always
correct and version-safe, but it leaks into every affected signature (plus body
wrap/unwrap), forces the user to track which types need wrapping, and recurs with
every new foreign boundary. A hand-written `impl TsJson` on a local newtype remains
the last-resort escape for types the generator can't describe at all.

**5. Bringing the rung into scope — a per-module import named `TsJsonExt`.** For the
rung to dispatch, its trait must be in lexical scope at each export site (autoref
specialization resolves on an unqualified call, which only sees imported traits), and
it must be defined in the adapter (orphan-legality). Rust scope is per-module, so the
cost is one anonymous import per exporting module: `use neon_ts_rs::TsJsonExt as _;`
— the same `use … as _` idiom Neon's macros already emit internally. *Rejected — a
Cargo feature that auto-wires the rung* (`neon = { features = ["ts-rs"] }`, macro
emits the import for you): zero source overhead, but it forces `neon` to take an
optional dependency on a *specific* adapter, inverting the dependency arrow the crate
split establishes and — decisively — not scaling to third-party adapters `neon` ships
no knowledge of. (Proc-macros also can't carry a crate-level "which adapter" choice
across invocations, so a configure-once-in-code form isn't available.) *Rejected — a
per-item attribute* (`#[neon_ts_rs::export]` or an extra marker per export):
per-boundary overhead, no better than the wrapper it replaces. *Rejected —
overloading one name* so a single `use neon_ts_rs::TsJson;` pulls in both the bridge
derive (macro namespace) and the rung trait (type namespace), à la
`use serde::Serialize;`: maximally terse, but the rung trait would be a *different*
trait from the contract trait `neon_typescript::TsJson`, so a bound naming it would
silently mean the wrong thing — serde's overload works because its trait and derive
are two faces of one concept; ours would be two concepts sharing a name. `TsJsonExt`
reads as "this adapter extends `TsJson` coverage to additional (foreign) types."

**6. Collapsing `neon-typescript` into `neon`.** Considered dropping the standalone
contract crate and defining `TsJson` in `neon` directly. *Rejected:* it relieves no
orphan-rule constraint (a foreign-type impl must live in the trait's crate either way,
and the adapter rung still needs a shared probe to hang on — which is exactly why
`TsJsonProbe` lives in `neon-typescript`), and it *worsens* versioning — adapters
would pin a `neon` version instead of a tiny, semver-stable contract crate, recoupling
adapter releases to neon's. The split is what lets an adapter depend only on the
stable trait plus its upstream.

**7. One trait for both conversions — rejected in favor of one trait per
conversion.** An earlier version had a single `TypeScript` trait giving each Rust type
one TypeScript type, and the macro consulted it at every position. But Neon's two
conversions disagree: the built-in `Option` and `Vec` impls described the JSON shape,
so a bare `Option<u32>` return was declared `number | null` while the runtime returns
`undefined` (code that checks `x !== null` compiles and then crashes), and a bare
`Vec<f64>` was declared `number[]` when the runtime wants a `Float64Array`. Meanwhile
the smart-pointer impls described the bare shape, so the same types inside `Json<T>`
were declared opaque boxes. *Decision:* one trait per conversion, chosen by position
(see [Which conversion runs at a position](#which-conversion-runs-at-a-position)).
This changes no runtime behavior; it makes each declaration describe the conversion
that actually runs. *Rejected — one trait with separate argument/return methods:*
blocked by the orphan rule, since the bare shape of `Vec<T>` depends on `T: Binary`,
which `neon-typescript` can't name, while `neon` can't implement `neon-typescript`'s
trait for `Vec`; it would also put Neon-specific conversion knowledge into the trait
adapters implement. *Rejected — keep one trait but only trust it at bare positions
for a whitelist of types whose two shapes agree:* cheap and safe, but bare `Vec` and
`Option` become `unknown`, and it does nothing for the JSON side.

**8. Naming — `TsJson`, `TsFromJs`, `TsIntoJs`.** Each trait is named after the Neon
conversion it describes (`Json<T>`, `TryFromJs`, `TryIntoJs`), so a reader who knows
Neon's conversions can guess what each covers. *Rejected — keeping the name
`TypeScript`:* it suggests one TypeScript type per Rust type, which is the exact
misconception behind §7. *Rejected — `TsSerde`:* the shape is specifically
serde_json's (e.g. `u64` → `number`, map keys become strings), not serde's
format-neutral data model. *Rejected — spelled-out (`TypeScriptJson`, …) or
`…Shape` names:* longer, or not obviously about TypeScript.

**9. Not splitting `TsJson` by direction.** serde can accept more than it emits
(`#[serde(default)]`, aliases, missing `Option` fields), so in principle a JSON
argument and a JSON return have different types. *Decision:* one JSON shape. Those
differences come from the user's serde attributes, which only the generator sees, and
the generators produce a single type; splitting the trait would just make adapters
fill two methods with the same string. Using the serialized shape for an input is the
safe direction (it may reject some valid input, but never accepts invalid input). If a
generator ever offers distinct input and output types, an input method defaulting to
the output one could be added without breaking anyone.

**10. Fallback type — `unknown`, not `any`.** *Decision:* a type with no known
TypeScript type is declared `unknown`. An `unknown` return must be narrowed before
use, while `any` silently disables type checking for everything it flows into. (For
arguments the two accept the same values, which is part of why strictness, §11,
remains interesting.)

**11. Strictness — lenient by default; reporting at generation time, later.** Even
with `unknown`, a missing type is silent: a forgotten `TsJsonExt` import goes
unnoticed, types reachable only through a fallback vanish from the output, and an
`unknown` argument accepts anything. *Rejected — a per-item `ts_strict` attribute:*
an earlier version had one; it had gaps (syntactic fallbacks bypassed it) and asks for
the same per-site vigilance that causes the problem. *Rejected — strict by default at
compile time,* with leniency opted into per module by a `use` (an import can only add
method-resolution candidates, so it can opt into leniency but never into strictness,
which is why the default would have to flip): Cargo unifies features across a build,
so if one crate enables `neon/typescript`, another crate built alongside it would
start emitting metadata and fail to compile without ever having asked for TypeScript —
and its authors couldn't fix it without writing newtypes. The errors would also be
hard to read (an E0599 in macro-generated code that
`#[diagnostic::on_unimplemented]` can't improve). *Decision:* lenient by default,
permanently, and a follow-up that reports `unknown`s **at generation time** with
messages Neon controls (see [Follow-up work](#follow-up-work)), where whoever
generates the `.d.ts` decides whether they're fatal.

## Implementation plan

1. The contract crate (`neon-typescript`: `TsJson`, its built-ins, and the JSON
   probe) and the in-tree ts-rs adapter (`neon-ts-rs`: the `TsJson` bridge derive,
   collection helpers, and `TsJsonExt`).
2. Neon's bare shapes (`TsFromJs` / `TsIntoJs` next to each conversion, box brands,
   the bare probes), `generate()`, and auto-attach.
3. The export and class macros: per-position trait selection, `Result` peeling,
   `unknown` fallbacks, and the escape-hatch attributes; metadata types internal to
   `neon::macro_internal`.
4. End-to-end tests, including a `tsc --noEmit --strict` check over the full generated
   output and runtime round-trips confirming each declaration matches the runtime.
5. Validate against the dogfooding PR
   ([dherman/tantivy#3](https://github.com/dherman/tantivy/pull/3)).
6. Before release, extract `neon-ts-rs` to its own repository and publish it; the
   dogfooding git dependency becomes a normal versioned dependency.
