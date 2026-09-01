# TypeScript Declaration Generation for Neon

## Status

This is the agreed design, presented as a single coherent design rather than a
change-set. Type information for user data types comes from third-party generators
(via an adapter) rather than a Neon-owned derive; the earlier in-tree serde-aware
derive macro has been removed. The PR implements the adapter-based core (the minimal
trait, built-in impls, bridge derive, `generate()`, auto-attach). The one piece still
being wired up is the adapter-provided **boundary rung** (`TypeScriptExt`) that lets a
*foreign* type used at a `Json` boundary resolve through the generator instead of
degrading to `any` — see [Graceful fallback](#graceful-fallback-for-missing-impls).
The ts-rs adapter (`neon-ts-rs`) currently lives in-tree for end-to-end dogfooding and
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

Responsibility splits cleanly in two:

- **Neon owns the export→declaration machinery.** The `#[neon::export]` /
  `#[neon::class]` macros collect type metadata at compile time; at runtime,
  `generate()` walks that metadata and renders a `.d.ts` string. Neon supplies the
  TypeScript mapping for its own boundary types (`Handle<Js*>`, boxed smart
  pointers, `Json<T>`, classes) and for std/core types. This is stable, small, and
  squarely Neon's domain.

- **Type information for user data types (and other foreign types) comes from a
  third-party generator** (ts-rs or specta), reached through a thin *adapter*. Neon
  does **not** own a serde-aware derive macro. Understanding serde's serialization
  (renames, enum tagging, flatten, optionality, generics) is an open-ended
  maintenance treadmill that tracks the evolution of serde and Rust; a mature crate
  like ts-rs already owns it. The same adapter also lets foreign types (e.g. ordered
  maps, dates) resolve through the generator at a boundary, so Neon owns no per-crate
  type mappings. See [Design rationale](#design-rationale) for why.

The crate picture:

```
neon-typescript          the stable contract: the TypeScript trait + built-in
  (no third-party deps)   impls for std/core types. Rarely changes.
     ▲            ▲
     │            │
   neon        neon-ts-rs / neon-specta     the adapters
```

- **`neon-typescript`** — the trait and impls for std/core types. No dependency on
  serde, ts-rs, specta, or `neon`. The contract everything binds to; deliberately
  minimal and semver-stable.
- **`neon`** — depends on `neon-typescript`. Adds impls for its boundary types, the
  `generate()` machinery, auto-attach, and the export/class metadata macros.
- **`neon-ts-rs` / `neon-specta`** — adapters. Each depends on `neon-typescript`
  and references its upstream (ts-rs / specta) in generated code — **not** on
  `neon`. Versioned independently to track the upstream. See
  [Crate & repository layout](#crate--repository-layout).

## Architecture

### The `TypeScript` trait

The contract is deliberately minimal and string-based:

```rust
pub trait TypeScript {
    /// The TypeScript type *expression*: "number", "string[]", "SearchResult", …
    fn ts_type() -> Cow<'static, str>;

    /// An optional top-level declaration this type needs (an interface, type
    /// alias, or `declare class`). `None` for primitives.
    fn ts_decl() -> Option<Cow<'static, str>> { None }

    /// Add this type's own declaration plus its transitive declarations to
    /// `decls`, keyed by name for dedup. Composite types override this to also
    /// collect from their components (see "transitivity" below).
    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        if let Some(d) = Self::ts_decl() {
            decls.entry(Self::ts_type().into_owned()).or_insert_with(|| d.into_owned());
        }
    }
}
```

No derive, no serde parsing, no AST lives here — that keeps the crate trivially
stable. (A structured-AST output is a deferred, non-v1 concern; see
[Alternatives considered §1](#alternatives-considered).)

**Transitivity is decentralized.** There is no central tree-walk. Each composite
type's `ts_collect` adds its own declaration and then calls `ts_collect` on its
component types; the transitive walk emerges from that call chain. Leaf types use
the default (add self, stop); `Vec<T>` delegates to `T`; a struct adapter delegates
to each field type. `generate()` seeds the walk from each export's parameter and
return types, and the `BTreeMap` keyed by type name dedups declarations referenced
from multiple places.

### Where type information comes from

Four sources implement `TypeScript`, in order of how a type is resolved:

1. **Built-in impls (in `neon-typescript`)** for std/core types:

   | Rust type | `ts_type()` |
   |---|---|
   | `f64`, `i32`, `u32`, `u8`, … | `number` |
   | `String`, `&str` | `string` |
   | `bool` | `boolean` |
   | `()` | `undefined` (→ `void` in return position) |
   | `Option<T>` | `T \| null` |
   | `Vec<T>`, `HashSet<T>` | `T[]` |
   | `HashMap<K, V>`, `BTreeMap<K, V>` | `Record<string, V>` |
   | `(A, B)` | `[A, B]` |
   | `Result<T, E>` | `T` (the error becomes a JS throw) |

   Two of these are worth noting: `HashMap` maps to `Record<string, V>` regardless
   of `K`, because JSON object keys are always strings; and `Option<T>` maps to
   `T | null` (matching serde's default serialization, where `None` becomes a
   present `null`), consistent with what the generators emit — see
   [Alternatives considered §2](#alternatives-considered). The **branded-box impls
   for the smart pointers Neon auto-boxes** (`Arc<T>`, `Rc<T>`, `RefCell<T>`,
   `Ref<T>`, `RefMut<T>`; see [Boxed types](#boxed-types)) also live here — the
   orphan rule forbids implementing `TypeScript` for these foreign types anywhere
   but the trait's own crate.

2. **Boundary impls (in `neon`)** for Neon's own JS types: `Handle<'cx, JsBigInt>`
   → `bigint`, `Handle<'cx, JsString>` → `string`, etc.; the extractors; `Json<T>`
   delegating to `T`; the `Boxed<T>` extractor (reusing the branded-box convention
   from `neon-typescript`); and classes (see [Class exports](#class-exports)).
   Foreign types that are neither std nor Neon-local (e.g. `IndexMap`,
   `chrono::DateTime`, `serde_json::Value`) are *not* special-cased here. Instead,
   when the adapter's `TypeScriptExt` is in scope, they are described by the
   generator (see [Graceful fallback](#graceful-fallback-for-missing-impls) below
   for how), so Neon owns no per-crate impls. A type the generator doesn't cover
   falls through to `any` (which for `serde_json::Value` is exactly the intended
   result).

3. **User data types**, via an adapter: the user derives the upstream generator's
   trait plus a trivial *bridge derive* that implements `neon-typescript`'s
   `TypeScript` by delegating to it. See [Type providers](#type-providers).

4. **Hand-written impls** for the rare user who uses no generator crate: a small
   `impl TypeScript` (just `ts_type` + `ts_decl`). Neon is intentionally out of the
   derive business.

### Graceful fallback for missing impls

Enabling the `typescript` feature must never force a `TypeScript` impl on every
type in every export — a user typing part of their API should not be blocked by an
export that uses an opaque or un-annotated type. Such types simply appear as `any`.

> The "rung" vocabulary below is internal mechanism, for maintainers. Users never
> encounter it: the entire user-facing surface is one import,
> `use neon_ts_rs::TypeScriptExt as _;`, explained in plain terms under
> [Type providers](#type-providers).

This is achieved with an **autoref-specialization probe** in the macro-generated
metadata, so the macros never emit a hard `<T as TypeScript>::ts_type()` bound.
Resolution is a **three-rung ladder**, tried most-specific first:

```rust
// In neon-typescript — always available. (It lives here, not in `neon`, so that
// an adapter crate can hang its own rung on it; see below.)
pub struct TsProbe<T>(pub PhantomData<T>);

// Rung 1 (most specific): inherent method, available only when T: TypeScript.
impl<T: TypeScript> TsProbe<T> {
    pub fn ts_type_of(&self) -> Cow<'static, str> { T::ts_type() }
}

// Rung 3 (fallback): reached via autoref for ALL T.
pub trait TsFallback { fn ts_type_of(&self) -> Cow<'static, str>; }
impl<T> TsFallback for &TsProbe<T> {
    fn ts_type_of(&self) -> Cow<'static, str> { "any".into() }
}
```

**Rung 1** — the type has a `TypeScript` impl (a built-in, a boundary impl, a
class, or a user type via the bridge derive). Real type info. **Rung 3** — nothing
matches; the type becomes `"any"`, with no viral bounds and no compile error.

**Rung 2 (the adapter rung)** sits between them and is what lets a *foreign* type
used at a boundary resolve through the generator instead of degrading to `any`. It
is provided by the adapter, because the orphan rule bars anyone but the trait's own
crate from implementing `TypeScript` for a foreign type — and `neon-typescript` can't
name `ts_rs::TS`. So the adapter defines its own trait and implements it for the
*bare* probe:

```rust
// In neon-ts-rs. A new, adapter-local trait, so implementing it for a foreign
// type (TsProbe is defined in neon-typescript) is orphan-legal ("you can always
// implement your own trait").
pub trait TypeScriptExt { fn ts_type_of(&self) -> Cow<'static, str>; }
impl<T: ts_rs::TS> TypeScriptExt for TsProbe<T> {
    fn ts_type_of(&self) -> Cow<'static, str> { neon_ts_rs::ts_type::<T>() }
}
```

The `&`-levels are what order the rungs. The call site is unchanged from the
two-rung probe — `(&__probe).ts_type_of()` — so the receiver has type `&TsProbe<T>`.
Rung 2's method receiver is *also* `&TsProbe<T>` (`&self` on `impl … for TsProbe<T>`),
the same candidate as rung 1's inherent method; since inherent methods outrank trait
methods at the same candidate, rung 1 wins whenever it applies and rung 2 catches the
rest. Rung 3's receiver is one autoref deeper (`&&TsProbe<T>`, from `impl … for
&TsProbe<T>`), so it is reached only after both. The probe *call shape* is therefore
unchanged (`(&__probe).ts_type_of()`); the rung's wiring is just (a) relocating
`TsProbe`/`TsFallback` into `neon-typescript` so the adapter can name them, and
(b) probing the payload of a `Json` boundary, below.

**Probing through `Json`.** Data crosses the boundary as `Json<T>`, but rung 2 fires
on the *bare* foreign type: `Json<Foreign>: ts_rs::TS` is false and unfixable (the
orphan rule bars anyone from implementing ts-rs's trait for Neon's `Json`). So at a
`Json` boundary the macro instantiates the probe with the payload `T`, not the
`Json<T>` wrapper. `Json<T>` is transparent for TypeScript — its TS shape is exactly
`T`'s — so this leaves rung-1 output identical for every existing case while making a
foreign payload (e.g. `IndexMap<String, User>`) reachable by rung 2.

Now `Json<IndexMap<String, Field>>` at a boundary — a foreign type with no
`TypeScript` impl — resolves via rung 2 through ts-rs (at *the user's* ts-rs version),
rather than falling to `any`. This is the only place the gap can be closed: the bridge
derive reaches only user-owned types, and both the adapter and the user are
orphan-barred from implementing `TypeScript` on a foreign type directly. (Foreign
types *nested inside* a user type never hit the gap — the bridge delegates the whole
type to ts-rs, whose visitor already handles them. The rung is specifically for a
foreign type used as the *outermost* type at a boundary.)

**Opting the rung into scope.** Autoref-specialization dispatches on an *unqualified*
method call, and for a *trait* method that only resolves if the trait is `use`d in
scope. Rung 1 is an inherent method (found via the fully-qualified `TsProbe` path — no
import), and rung 3's `TsFallback` lives in a crate the macro knows, so the macro
brings it into scope itself (`use … as _;`). It cannot do that for rung 2's trait,
because it doesn't know which adapter the user chose. So the user brings it in, once
per module that has exports:

```rust
use neon_ts_rs::TypeScriptExt as _;   // "this adapter provides TS for extra types"
```

This is the ordinary anonymous-extension-trait idiom (`use std::io::Write as _;`) —
the name never appears anywhere but the import, and the methods are consumed only by
macro-generated code. If the import is absent (or the user uses no adapter),
resolution simply skips rung 2 and behaves exactly as the two-rung ladder did:
real-or-`any`. Its one hazard — forget the import and a boundary silently becomes
`any` again — is covered by `#[neon::export(ts_strict)]`, which emits the same probe
call but *omits* the rung-3 `TsFallback` import, so a type that resolves via rung 1 or
rung 2 compiles while one that would hit rung 3 is a compile error at the signature
(naming both the `TypeScript` and `ts_rs::TS` bounds it failed). See
[Escape-hatch attributes](#escape-hatch-attributes); pairing the import with
`ts_strict` is the recommended posture for JSON-heavy addons.

### Metadata collection

Behind the `typescript` feature, the `#[neon::export]` / `#[neon::class]` macros
emit a metadata entry per export into a `linkme::distributed_slice`, matching the
existing `EXPORTS` pattern:

```rust
#[linkme::distributed_slice(TYPE_METADATA)]
static __META_search: ExportMeta = ExportMeta::Function(FunctionMeta {
    name: "search",
    params: &[ParamMeta {
        name: "query",
        // closures, resolved at runtime through the probe
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

Emission is gated so builds without the feature pay nothing: `neon`'s `typescript`
feature enables `neon-macros/typescript`, and the macros gate metadata emission on
`cfg!(feature = "typescript")` (resolved when `neon-macros` compiles; Cargo unifies
the feature across the build). With the feature off, `#[neon::export]` emits no
metadata statics at all.

### Boxed types

When an export accepts or returns a smart pointer like `Arc<MyStruct>`, it appears
on the JS side as an opaque boxed value. The generated TypeScript uses a *branded
interface* so distinct boxed types are not interchangeable:

```typescript
export declare const __neon_tag: unique symbol;
export interface BoxedMyStruct { readonly [__neon_tag]: 'MyStruct' }
```

The brand value (`'MyStruct'`) carries the type's identity; the interface name is
synthesized (`Boxed` + a sanitized identifier). Boxed types are intended for
*named* types; composed types (e.g. `Arc<Option<String>>`) get a best-effort
synthesized name (see [Stability](#stability)).

### Class exports

`#[neon::export(class)]` / `#[neon::class]` know the full class structure at
compile time — constructor, methods (with receiver, attributes, return type), and
const properties — enough to emit a complete `declare class`. A class like:

```rust
#[neon::export(class)]
struct Point { x: f64, y: f64 }

#[neon::class]
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

The class macro emits a `TypeScript` impl where `ts_type()` is the class name and
`ts_decl()` is the `declare class` block, so classes appear as their name when used
as parameter/return types in other exports (not as opaque boxes).

### Generation and output

`neon::typescript` exposes:

```rust
/// Render a complete `.d.ts` string.
pub fn generate() -> String;

/// Like `generate`, but wraps the body in `declare module "X" { ... }`.
pub fn generate_with(options: GenerateOptions) -> String;

pub struct GenerateOptions {
    /// Wrap output in `declare module "<name>" { ... }`.
    pub module: Option<String>,
}
```

(A structured-AST API, `generate_ast()`, is deferred from v1 — see
[Alternatives considered §1](#alternatives-considered).)

The default output is a `.d.ts` intended to sit beside the `.node` binary. Top-level
items are named `export declare` statements (with `export interface` / `export type`
for collected declarations):

```typescript
// Auto-generated by Neon. Do not edit.

export declare const __neon_tag: unique symbol;
export interface BoxedSchema { readonly [__neon_tag]: 'Schema' }
export interface SearchResult { /* … */ }

export declare function newSchema(schema: SchemaDescriptor): BoxedSchema;
export declare function commit(index: BoxedIndex): Promise<void>;

export declare class Point { /* … */ }
```

Module scoping (`generate_with({ module: Some("./load.cjs") })`) instead wraps the
body in `declare module "./load.cjs" { … }` — useful when the addon is loaded via an
`@neon-rs/load`-style shim and the declarations should attach to that import path.

**Auto-attach.** When the `typescript` feature is on, Neon attaches the generated
declarations to the addon's module exports under `Symbol.for("neon:types")` during
module init, so a small Node script can extract them without a Rust-side shim:

```js
const addon = require("./index.node");
require("fs").writeFileSync("index.d.ts", addon[Symbol.for("neon:types")]);
```

Auto-attach only exposes the **flat** string. Module scoping needs the import path
(`"./load.cjs"`), which is the consumer's choice and unknown to Neon at build time,
so it can't be baked into the auto-attached value. A consumer who wants module-scoped
output either calls `generate_with({ module: … })` from a Rust-side build step, or
wraps the flat string themselves in `declare module "…" { … }`. (Auto-attach runs
before user-defined exports, so it works even with a custom `#[neon::main]`.)

> A future enhancement could auto-attach a *function*
> `addon[Symbol.for("neon:types-module")]("./load.cjs")` that performs the wrapping
> at extract time, giving module-scoped output without a Rust-side build step. Not
> in this iteration.

### Escape-hatch attributes

The codegen handles common cases; a few per-item overrides cover the ragged edges:

- `#[neon(ts_skip)]` — exclude an item from the `.d.ts`. Works on functions,
  classes, methods, static properties, and the `fn new` constructor. The class's
  `TypeScript` impl is still emitted, so other types can reference it by name.
- `#[neon(ts_name = "...")]` — rename in TS output without changing the Rust type
  or JS export name.
- `#[neon::export(ts_returns = "...")]` (also on methods) — override the inferred
  return type with a literal TS string (e.g. for a type Neon can't infer).
- `#[neon(ts_type = "...")]` on an individual function/method parameter — override
  that parameter's inferred type.
- `#[neon::export(ts_strict)]` (on functions, classes, or methods) — opt out of the
  silent `any` fallback: any referenced type lacking a `TypeScript` impl becomes a
  compile error. Catches the case where a type is serialized across the boundary but
  types as `any`, so consumers never see its fields. Class-level applies to the
  constructor and all methods.
- `#[neon::export(class, ts_no_constructor)]` — emit a `declare class` with no
  `constructor`, for classes only constructed via factory methods elsewhere.

### Feature-gating

TypeScript generation is behind a `typescript` feature on `neon`:

```toml
neon = { version = "...", features = ["typescript"] }
```

When off (the default), no metadata is emitted, no extra dependencies are pulled,
and there is zero impact on existing users (verified across the napi feature matrix).

## Type providers

User data types (the structs and enums carried across the boundary by `Json<T>`)
get their TypeScript from a third-party generator, reached through an adapter.

### The bridge derive

Rust's orphan rule forbids a blanket `impl<T: ts_rs::TS> TypeScript for T` in an
adapter crate (and it would collide with the built-in impls). But the *user owns
their type*, so the adapter ships a trivial derive that generates the bridge impl
in the user's crate:

```rust
// neon-ts-rs generates, for the user's MyType:
impl neon_typescript::TypeScript for MyType {
    fn ts_type() -> Cow<'static, str> { neon_ts_rs::ts_type::<Self>() }
    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        neon_ts_rs::ts_collect::<Self>(decls);
    }
}
```

`neon_ts_rs::ts_type` / `ts_collect` delegate to `<Self as ts_rs::TS>`, assembling
a flat declaration set via ts-rs's `TypeVisitor`. This bridge derive is *trivial*:
it reads none of the type's fields or serde attributes — it emits a fixed
delegation. All serde understanding stays in ts-rs. The generated impl names the
trait through a hidden re-export in the adapter (`::neon_ts_rs::__private::TypeScript`)
and otherwise uses only fully-qualified `::std` / `::ts_rs` paths, so a consumer
depends on just `neon` + `neon-ts-rs` (+ `ts-rs`) — never on `neon-typescript`
directly, which stays a pure implementation detail. (This is the serde-derive
convention: the derive references its own crate's re-exports, not the underlying
contract crate.) The adapter never depends on `neon`, and extraction stays the
user's ordinary `Json<T>`.

### What the user writes

```rust
// ts-rs path: one extra derive, nothing at the export boundary.
#[derive(Serialize, Deserialize, ts_rs::TS, neon_ts_rs::TypeScript)]
#[serde(rename_all = "camelCase")]
struct SearchResult { doc_id: u32, /* … */ }

#[neon::export(json)]
fn search(q: String) -> SearchResult { /* … */ }   // Json extraction unchanged
```

- **ts-rs user** — derive `ts_rs::TS` + `neon_ts_rs::TypeScript`; the types are also
  visible to any ts-rs tooling.
- **specta user** — derive `specta::Type` + `neon_specta::TypeScript`; the types
  plug into the specta/Tauri ecosystem, with the (less stable) specta dependency
  quarantined in its adapter crate.
- **neither** — hand-write `impl TypeScript` (see
  [Where type information comes from](#where-type-information-comes-from)).

Nothing above is needed at the *boundary* as long as the outermost type has a
`TypeScript` impl (a user type via the bridge, a std type, a class). The one case
that needs the adapter is a **foreign type used directly at a boundary** — e.g.
returning `Json<IndexMap<String, Field>>` rather than wrapping it in a named struct.
There, import `TypeScriptExt` in that module — it lets ts-rs describe foreign types
at your boundaries, so they are typed instead of falling back to `any`:

```rust
// Let ts-rs describe foreign types (e.g. IndexMap) used directly at a boundary.
use neon_ts_rs::TypeScriptExt as _;   // once per module that exports such types

#[neon::export]
fn fields(&self) -> Json<IndexMap<String, FieldDescriptor>> { /* … */ }
```

The `IndexMap` then renders through ts-rs (enable ts-rs's own `indexmap-impl`
feature) at your pinned ts-rs version, and the types reachable through it stay in
the output. Pair it with `#[neon::export(ts_strict)]` so a forgotten import surfaces
as a compile error rather than a silent `any`.

**Its reach is exactly the generator's type coverage.** `TypeScriptExt` only covers
a foreign type that ts-rs itself implements; a foreign type ts-rs doesn't know still
types as `any`. In practice, reach for a generator-supported type: e.g. ts-rs 12 ships
`indexmap-impl` but has no `ordermap` impl, so `IndexMap` works at a boundary while
`ordermap::OrderMap` (insertion-ordered identically, and a thin layer over `IndexMap`)
does not — swap to `IndexMap` rather than debugging a silent `any`. See
[Graceful fallback](#graceful-fallback-for-missing-impls) for the mechanism, and
[Alternatives considered §4–5](#alternatives-considered) for why this beats
Neon-owned per-crate impls or a per-boundary wrapper.

### Adapter responsibilities

The generators are faithful to the *Rust type*; Neon's boundary is faithful to
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

The **Rust API** (the `TypeScript` trait, `generate()` / `generate_with()`, the
metadata types, and the macro attributes) follows the crate's normal semver
guarantees.

The **generated output format** — the exact text of the `.d.ts` — is **not yet
covered by semver** and may change while the feature settles. Notably, the
synthesized names for *anonymous or composed* boxed types (e.g. `Arc<Option<String>>`)
are best-effort and subject to change; names for *named* boxed types
(`Arc<Database>` → `BoxedDatabase`) are stable. If you commit generated declarations
to source control, expect to regenerate them across Neon upgrades until the format
is declared stable.

Async exports (`#[neon::export(task)]`, `async fn`) render as `Promise<T>`; a unit
return normalizes to `Promise<void>`.

---

## Design rationale

### Why Neon doesn't own a serde-aware derive

A Neon-owned `#[derive(neon::TypeScript)]` that understands serde attributes
(`rename_all`, `tag`/`content`/`untagged`, `flatten`, `skip`, `default`,
`transparent`, generics) is the single largest and most volatile part of this
feature. Keeping it correct is an **open-ended maintenance treadmill** that tracks
the evolution of serde and Rust — and it is tangential to Neon's mission. A mature,
widely-used crate (ts-rs) already owns that surface. By owning only a minimal
string trait and sourcing user-type information from such a crate through a
version-isolated adapter, Neon sheds the treadmill while still giving users the same
result with one extra derive.

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

**1. Structured AST — string-only trait, and `generate_ast()` deferred from v1.**
The feature could also emit a structured, TSESTree-shaped AST (for programmatic
transforms). *Decision:* the trait is string-only, and `generate_ast()` plus the
string→AST parser that backs it are **deferred from v1**; v1 ships the `.d.ts`
string only. *Rejected:* putting a `ts_type_ast()` method in the trait — it would
drag the AST node types and parser into `neon-typescript`, inflating the crate we
most want minimal, and bridged types are parser-sourced regardless (so built-ins
being specially native buys inconsistency, not fidelity). *Why defer at all:* the
dogfooding consumes only the string (`Symbol.for("neon:types")`) and never touches
the AST — it hand-rolls a `declare module` wrap that the *module-scoped string*
already provides — so shipping an unused, semver-affecting API (and its parser) from
day one is a cost with no current payoff. If a concrete consumer materializes, the
AST can be added later, produced in `neon` by parsing the strings.

**2. `Option` leniency — user-expressible via serde, not adapter policy.** serde's
default `Option<T>` serializes `None` to a present `null` (`T | null`), while
deserialization leniently accepts absent/`null`/`undefined` → `None`. *Decision:*
the adapter owns no leniency policy; a user expresses an optional field with
`#[serde(skip_serializing_if = "Option::is_none", default)]`, which ts-rs renders as
`f?: T | null` — fully lenient and accurate in both directions. Built-in `Option<T>`
matches this at `T | null` for consistency. *Rejected:* post-processing every
`Option` to be lenient (bakes a uniform policy that's loose for return types,
reintroduces an output-format transform in the adapter, and can't even distinguish
`Option` from other nullables once ts-rs has stringified it) and separate
input/output types (precise but a large feature). *Consequence:* leniency is opt-in,
not automatic — the honest default for a plain `Option<T>` is `T | null`.

**3. Bridged output styling — accept each provider's idiom.** ts-rs renders
semantically-identical TypeScript in a different style than Neon's built-ins
(`Array<string>` vs `string[]`, quoted discriminant keys). *Decision:* do not
normalize; bridged types render in their provider's style, mixing with Neon's own
style for boundary types. *Rejected:* normalizing (fragile string-rewriting that
reintroduces the "understand and rewrite the output format" ownership we're shedding)
and adopting the provider's style wholesale (impossible — Neon's own boundary types
stay Neon-styled). The cost is cosmetic inconsistency across a file, consistent
within any one type.

**4. Foreign types at a boundary — an adapter rung, not Neon-owned batteries or a
per-boundary wrapper.** A foreign, non-std type used as the *outermost* type at a
`Json<T>` boundary (e.g. `Json<IndexMap<String, Field>>`) has no `TypeScript` impl:
the orphan rule bars both the adapter and the user from writing one for a foreign
type, and the bridge derive reaches only user-owned types. Left alone it falls to
`any` — and because collection recurses only through *typed* components, every type
reachable *only* through it silently vanishes from the output, turning graceful
degradation into a downstream `tsc` break located far from its cause. *Decision:*
close it with the adapter-provided `TypeScriptExt` rung (see
[Graceful fallback](#graceful-fallback-for-missing-impls)), which resolves any
`T: ts_rs::TS` through the generator at the user's own version. *Rejected — feature-
gated built-in impls in `neon-typescript`* (`typescript-indexmap`, `typescript-chrono`,
…, each mapping a foreign type to TS): the trait's crate is the only orphan-legal home
for such a generic impl, but putting it there re-couples the foreign crate's *version*
to `neon-typescript`'s declared range and silently fails to apply across a semver-major
bump (the impl is then for a different type) — exactly the per-crate maintenance surface
the adapter design sheds, reintroduced one crate at a time. The rung subsumes it for
anything the generator supports. *Rejected — an explicit per-boundary wrapper* (a
generic `TsRs<T>` / `JsonTs<T>` the user wraps around foreign types in each signature):
always correct and version-safe, but it leaks into every affected signature (plus
body wrap/unwrap), forces the user to track which types need wrapping, and recurs with
every new foreign boundary — the ongoing cost the feature exists to remove. A
hand-written `impl TypeScript` on a local newtype remains the last-resort escape for
types the generator can't describe at all.

**5. Bringing the rung into scope — a per-module import named `TypeScriptExt`.** For
the rung to dispatch, its trait must be in lexical scope at each `#[neon::export]`
site (autoref specialization resolves on an unqualified call, which only sees imported
traits), and it must be defined in the adapter (orphan-legality). Rust scope is
per-module, so the cost is one anonymous import per exporting module:
`use neon_ts_rs::TypeScriptExt as _;` — the same `use … as _` idiom Neon's macros
already emit internally for their own rungs. *Rejected — a Cargo feature that
auto-wires the rung* (`neon = { features = ["ts-rs"] }`, macro emits the import for
you): zero source overhead, but it forces `neon` to take an optional dependency on a
*specific* adapter, inverting the dependency arrow the crate split establishes and —
decisively — not scaling to third-party adapters `neon` ships no knowledge of.
(Proc-macros also can't carry a crate-level "which adapter" choice across invocations,
so a configure-once-in-code form isn't available.) *Rejected — a per-item attribute*
(`#[neon_ts_rs::export]` or an extra marker per export): per-boundary overhead, no
better than the wrapper it replaces. *Rejected — overloading the name `TypeScript`*
so one `use neon_ts_rs::TypeScript;` pulls in both the bridge derive (macro namespace)
and the rung trait (type namespace), à la `use serde::Serialize;`: maximally terse,
but `neon_ts_rs::TypeScript`-the-trait would be a *different* trait from the protocol
trait `neon_typescript::TypeScript`, so a bound `T: neon_ts_rs::TypeScript` would
silently mean the wrong thing — serde's overload works because its trait and derive
are two faces of one concept; ours would be two concepts sharing a name. The distinct
`TypeScriptExt` keeps a two-name import (collapsible to one `use` clause) at the cost
of one extra listed item. The name reads as "this adapter provides TypeScript
conversions for additional (foreign) types."

**6. Collapsing `neon-typescript` into `neon`.** Considered dropping the standalone
contract crate and defining the trait in `neon` directly. *Rejected:* it relieves no
orphan-rule constraint (a foreign-type impl must live in the trait's crate either way,
and the adapter rung still needs a shared `TsProbe` to hang on — which is exactly why
`TsProbe` lives in `neon-typescript`), and it *worsens* versioning — adapters would
pin a `neon` version instead of a tiny, semver-stable contract crate, recoupling
adapter releases to neon's. The split is what lets an adapter depend only on the stable
trait plus its upstream.

## Implementation plan

1. Build the two-crate skeleton — `neon-typescript` (trait + built-ins) and an
   initial in-tree `neon-ts-rs` (bridge derive + collection helper) — wired to one
   real `#[neon::export]` in the test addon, confirming the derive hygiene and the
   `Json<T>` + bridge interaction.
2. Point the dogfooding PR
   ([dherman/tantivy#3](https://github.com/dherman/tantivy/pull/3)) at `neon-ts-rs`
   via a git dependency for full end-to-end validation against a real library.
3. Rework this PR: drop the in-tree serde-aware derive, land `neon-typescript` and
   the retained Neon-side machinery (metadata collection, `generate()`, boundary and
   built-in impls, class exports, escape-hatch attributes, auto-attach, feature-gating).
4. Wire the boundary rung: relocate `TsProbe`/`TsFallback` into `neon-typescript`
   (re-exported from `neon::macro_internal`) so an adapter can name them, add the
   `TypeScriptExt` rung (`impl<T: ts_rs::TS> TypeScriptExt for TsProbe<T>`) to
   `neon-ts-rs`, and switch `ts_strict` to emit the probe without the `TsFallback`
   import. The probe *call shape* is unchanged, but the macro now instantiates it with
   the payload of a `Json` boundary (rung 2 fires on the bare foreign type, and
   `Json<Foreign>: ts_rs::TS` is unfixable). Validate that a bare foreign type at a
   boundary (e.g. an ordered/indexed map) resolves through ts-rs with the per-module
   `use neon_ts_rs::TypeScriptExt as _;` — including that the *value* type stays in the
   output — and that omitting the import under `ts_strict` is a compile error rather
   than a silent `any`.
5. Before release, extract `neon-ts-rs` to its own repository and publish it; the
   dogfooding git dependency becomes a normal versioned dependency.
