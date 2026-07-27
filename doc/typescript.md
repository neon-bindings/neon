# TypeScript Type Declaration Generation for Neon

## Status: Implemented

This document began as a design proposal; the feature is now implemented behind
the `typescript` feature flag. Sections describing rationale and alternatives are
retained for context, but the API and output format described below reflect what
ships.

## Motivation

Neon modules export Rust functions and classes to JavaScript via the `#[neon::export]`
macro. Today, if a module author wants TypeScript consumers to have proper type
declarations, they must hand-write a `.d.ts` file (or a TypeScript wrapper module) that
redeclares every export with its correct type signature. This is tedious, error-prone,
and a real barrier to providing a good TypeScript experience from a Neon module.

As a motivating example, the [tantivy-node](https://github.com/dherman/tantivy)
project demonstrates the typical pattern: a hand-written
[TypeScript wrapper](https://github.com/dherman/tantivy/blob/main/src/index.cts)
that redeclares every function exported by the
[Rust addon](https://github.com/dherman/tantivy/blob/main/crates/tantivy-node/src/lib.rs),
then builds an idiomatic class-based API on top. The raw addon declarations
(`declare module "./load.cjs" { ... }`) are entirely mechanical -- they mirror the
Rust function signatures with types translated to TypeScript -- yet they must be
maintained by hand.

The goal of this initiative is to **automatically generate `.d.ts` type declarations**
for the exports of a Neon module.

## Scope

A Neon module typically has two layers of TypeScript API:

1. **Raw addon declarations** -- the flat functions and boxed types that `#[neon::export]`
   produces directly (e.g., `function newSchema(schema: SchemaDescriptor): BoxedSchema`)
2. **Idiomatic wrapper API** -- hand-authored classes and types that wrap the raw addon
   into a pleasant, object-oriented surface (e.g., a `Schema` class that holds a
   `BoxedSchema` internally)

Layer (2) involves design decisions that are inherently authorial. **This initiative
focuses on layer (1)**: generating correct, complete type declarations for the raw
exports of a Neon addon.

This is the highest-value target because:
- It's entirely mechanical -- every exported function has a deterministic TS signature
- It eliminates the most tedious and error-prone part of the workflow
- It gives wrapper authors a correct, typed foundation to build on

## Design Principles

- **Generated code should never be hand-modified**, so it can be safely regenerated
  at any time.
- **Minimal assumptions about build architecture.** The safest mechanism is a runtime
  reflection operation that users can invoke from their own build scripts, a standalone
  binary, or any other build tool of their choosing.
- **Minimal annotation burden.** Some amount of annotation (e.g., a `#[derive(...)]`
  attribute) is acceptable, but boilerplate should be low enough that auto-generation
  is clearly worth it compared to hand-writing type wrappers.

## Architecture

### Overview

The system has three layers:

1. **A `TypeScript` trait** that maps Rust types to their TypeScript representations.
   Neon provides built-in implementations for primitive types and common wrappers.
   Users derive it for their own types (particularly those used with `Json<T>`).

2. **Enhanced `#[neon::export]` macro** that collects type metadata for each export
   by emitting code that calls `TypeScript` trait methods at runtime.

3. **A runtime generation function** that iterates all registered exports, resolves
   their types, and produces a `.d.ts` string.

### The `TypeScript` Trait

```rust
/// A Rust type that has a known TypeScript representation.
pub trait TypeScript {
    /// The TypeScript expression for this type (e.g., `"number"`, `"string[]"`).
    fn ts_type() -> Cow<'static, str>;

    /// Optional top-level type declaration(s) needed to support this type
    /// (e.g., an interface or enum definition). Returns `None` for primitive
    /// types that need no declaration.
    fn ts_decl() -> Option<Cow<'static, str>> {
        None
    }

    /// Collect this type and all of its transitive dependencies into a
    /// declaration set. The default implementation adds `Self::ts_decl()`
    /// if present; types with fields/variants override this to also collect
    /// their children.
    fn collect_declarations(decls: &mut BTreeMap<Cow<'static, str>, Cow<'static, str>>) {
        if let Some(decl) = Self::ts_decl() {
            decls.insert(Self::ts_type(), decl);
        }
    }
}
```

### Built-in Implementations

Neon provides `TypeScript` impls for types that already have `TryFromJs`/`TryIntoJs`
implementations:

| Rust type | `ts_type()` | Notes |
|---|---|---|
| `f64`, `i32`, `u32`, `u8` | `"number"` | All JS-safe numeric types |
| `String`, `&str` | `"string"` | |
| `bool` | `"boolean"` | |
| `()` | `"undefined"` | |
| `Option<T>` | `"T \| undefined \| null"` | Delegates to `T::ts_type()` |
| `Vec<T>` | `"T[]"` | Delegates to `T::ts_type()` |
| `Result<T, E>` | `T` | Error case becomes a JS throw |
| `Json<T>` | Delegates to `T::ts_type()` | `T` must impl `TypeScript` |
| `Arc<T>`, `Ref<T>`, `RefCell<T>` | Opaque boxed type | See "Boxed Types" below |
| `Handle<'cx, V>` | Depends on `V` | JS value types |

### Boxed Types

When a Neon export accepts or returns a smart pointer like `Arc<MyStruct>`, it appears
on the JavaScript side as an opaque boxed value. The generated TypeScript uses a branded
interface to prevent accidental interchange between different boxed types:

```typescript
declare const __neon_tag: unique symbol;
interface BoxedMyStruct { readonly [__neon_tag]: 'MyStruct' }
```

This ensures that `BoxedMyStruct` and `BoxedOtherStruct` are not assignable to each
other, even though both are opaque.

### Derive Macro for User Types

Types used with `Json<T>` need a TypeScript representation that matches their serde
serialization. Users opt in with a derive macro:

```rust
#[derive(Serialize, Deserialize, neon::TypeScript)]
#[serde(tag = "type", rename_all = "camelCase")]
enum FieldDescriptor {
    Text { flags: Option<Vec<TextOption>> },
    String { flags: Option<Vec<TextOption>> },
    F64 { flags: Option<Vec<NumericOption>> },
}
```

The derive macro reads serde attributes to produce correct TypeScript. See
"Serde Compatibility" below for the full set of supported attributes.

### Type Metadata Collection

The `#[neon::export]` macro is enhanced (behind a `typescript` feature flag) to
generate a type metadata entry alongside each export's existing wrapper and
registration code. These entries are collected via `linkme::distributed_slice`,
matching the existing pattern used for `EXPORTS`:

```rust
#[linkme::distributed_slice(TYPE_METADATA)]
static __META_new_schema: ExportMeta = ExportMeta {
    name: "newSchema",
    // Functions that call TypeScript trait methods at runtime
    // to resolve concrete type strings
    params: &[ParamMeta { name: "schema", ts_type: || Json::<OrderMap<String, FieldDescriptor>>::ts_type() }],
    ret: || RefCell::<Schema>::ts_type(),
    is_async: false,
};
```

By using closures that call trait methods rather than storing static strings, the
metadata can resolve types at runtime, which lets us leverage Rust's trait resolution
to handle generics, `Json<T>` delegation, and all other type mappings.

### Graceful Fallback for Missing TypeScript Impls

A key design goal is that enabling the `typescript` feature should never force users
to implement `TypeScript` on every type used in their exports. A user who enables
`typescript` to generate declarations for some of their API should not be blocked by
exports using opaque types or third-party types that lack `TypeScript` impls. Those
exports should simply appear as `any` in the generated declarations.

This is accomplished via an **autoref specialization** pattern in the macro-generated
metadata closures. Instead of directly calling `<T as TypeScript>::ts_type()` (which
would require `T: TypeScript` and fail to compile otherwise), the macros use a probe
type that resolves the TypeScript representation when available and falls back to
`"any"` when it is not:

```rust
// In neon::macro_internal — always available, not feature-gated.

/// Compile-time probe for whether a type implements `TypeScript`.
pub struct TsProbe<T>(pub PhantomData<T>);

// Higher priority: inherent method, available only when T: TypeScript.
impl<T: TypeScript> TsProbe<T> {
    pub fn ts_type_of(&self) -> Cow<'static, str> { T::ts_type() }
    pub fn ts_collect_of(&self, decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

// Lower priority: trait method via autoref, available for ALL T.
pub trait TsFallback {
    fn ts_type_of(&self) -> Cow<'static, str>;
    fn ts_collect_of(&self, decls: &mut BTreeMap<String, String>);
}
impl<T> TsFallback for &TsProbe<T> {
    fn ts_type_of(&self) -> Cow<'static, str> { "any".into() }
    fn ts_collect_of(&self, _: &mut BTreeMap<String, String>) {}
}
```

The macro then generates metadata closures like:

```rust
ts_type: || {
    let __probe = TsProbe::<MyType>(PhantomData);
    (&__probe).ts_type_of()
}
```

Rust's method resolution tries the inherent method first (exact match on
`&TsProbe<T>`). If `T: TypeScript`, the inherent impl matches and returns the real
type. If `T` does not implement `TypeScript`, the inherent impl is unavailable, so
method resolution falls through to the trait impl on `&&TsProbe<T>` (via autoref),
which returns `"any"`.

This pattern ensures:
- Types with `TypeScript` impls get accurate type information
- Types without `TypeScript` impls silently produce `"any"`
- No compilation errors from missing trait implementations
- No viral `TypeScript` bound requirements on user types

### Generation API

The `neon::typescript` module exposes four entry points:

```rust
/// Render a complete `.d.ts` file as a string.
pub fn generate() -> String;

/// Like `generate`, but with rendering options (e.g. module wrapping).
pub fn generate_with(options: GenerateOptions) -> String;

/// Produce a structured, TSESTree-shaped AST (serde-serializable).
pub fn generate_ast() -> Vec<Decl>;

/// Like `generate_ast`, but with the same options as `generate_with`.
pub fn generate_ast_with(options: GenerateOptions) -> Vec<Decl>;
```

Each iterates `TYPE_METADATA`, resolves types via the `TypeScript` trait,
collects transitive declarations, and renders the result.

Users invoke these from a build script or standalone binary:

```rust
fn main() {
    let dts = neon::typescript::generate();
    std::fs::write("index.d.ts", dts).unwrap();
}
```

In practice most users don't need to call these directly: when the `typescript`
feature is enabled, the generated declarations are auto-attached to the addon's
module exports (see below), so a small Node script can extract them without any
Rust-side glue.

### Auto-Attached Module Symbols

When the `typescript` feature is enabled, Neon automatically attaches the generated
declarations to the addon's module exports under two well-known symbols:

- `Symbol.for("neon:types")` -- the rendered `.d.ts` string (suitable for writing
  directly to a `.d.ts` file).
- `Symbol.for("neon:types-ast")` -- a JSON-serialized structured AST (suitable for
  programmatic transformations).

This eliminates the need for users to write a `#[neon::export]` shim just to extract
the declarations from JavaScript.

```js
// extract-types.cjs
const addon = require("./index.node");
const fs = require("fs");

// Plain .d.ts text
fs.writeFileSync("index.d.ts", addon[Symbol.for("neon:types")]);

// Or structured AST for transformation pipelines
const ast = JSON.parse(addon[Symbol.for("neon:types-ast")]);
// ast is a Vec<Decl> -- iterate, transform, render however you like
```

The auto-attach happens during module initialization, before user-defined exports
are wired up, so it's available even if the user provides their own `#[neon::main]`.

### Structured AST API

In addition to [`generate`], Neon exposes [`generate_ast`] which returns a
`Vec<Decl>` of structured TypeScript AST nodes. The node shapes mirror
[TSESTree](https://typescript-eslint.io/packages/typescript-estree/), so JSON output
is compatible with TypeScript-ESLint tools.

```rust
let ast: Vec<Decl> = neon::typescript::generate_ast();
let json = serde_json::to_string_pretty(&ast)?;
```

The structured output enables programmatic transformations -- wrap declarations in
a module, prefix names, filter or rewrite types -- without resorting to regex
post-processing of the rendered string.

The [`TypeScript`] trait has an optional `ts_type_ast()` method that returns a
structured `TsType` node. The default implementation parses the string output of
`ts_type()`; types that want stable structured output should override this method
directly. Anything the parser cannot structure falls through to `TsType::Raw`.

### Per-Item Override Attributes

The codegen handles common cases automatically, but a few escape hatches are
available for cases where the inferred output isn't what you want:

- `#[neon(ts_skip)]` -- exclude an item from the generated `.d.ts`. Works on
  functions, classes, methods, static properties, and the `fn new` constructor.
  The class's `TypeScript` impl is still emitted, so other types can still
  reference it by name.
- `#[neon(ts_name = "...")]` -- rename an item in the TS output without
  affecting the Rust type or the JS export name. Useful when the desired TS
  identifier differs from the Rust identifier.
- `#[neon::export(ts_returns = "...")]` (also on methods) -- override the
  inferred return type with a literal TS type string. Useful for types that
  Neon can't infer.
- `#[neon(ts_type = "...")]` on individual function/method parameters -- override
  the inferred parameter type. Already supported on struct fields; this
  extends it to function/method parameters.
- `#[neon::export(ts_strict)]` (on functions, classes, or methods) -- opt out
  of silent `"any"` fallback. Any referenced type that does not implement
  `TypeScript` becomes a compile error. Useful for catching the case where a
  type is serialized across the FFI boundary (via `Json<T>`, etc.) but its
  inferred TS type is `any`, so consumers never see those fields. Class-level
  `ts_strict` applies to all methods and the constructor.
- `#[neon::export(class, ts_no_constructor)]` -- emit a `declare class` without
  a `constructor` member. Useful for classes that are only ever instantiated
  via factory methods on other classes (e.g. `Searcher::term_query` returning
  a `Query`), where exposing a JS-callable constructor would be misleading.

### Module Scoping

By default the rendered `.d.ts` emits top-level `export declare ...`
statements. To wrap the output in `declare module "X" { ... }` instead, use
[`generate_with`]:

```rust
let dts = neon::typescript::generate_with(neon::typescript::GenerateOptions {
    module: Some("./load.cjs".into()),
});
```

This is useful when the addon is loaded via an indirection (e.g. an
`@neon-rs/load`-style shim) and the type declarations should be attached to
that import path rather than the top-level package entry.

## Serde Compatibility

The derive macro must produce TypeScript that matches what serde actually serializes
to JSON, since `Json<T>` is the bridge between Rust types and JavaScript values.

### Supported Serde Attributes

**Enum representations:**

| Serde attribute | JSON shape | TypeScript output |
|---|---|---|
| *(default: externally tagged)* | `{"Variant": data}` | `{ Variant: Data }` |
| `#[serde(tag = "t")]` | `{"t": "Variant", ...fields}` | Discriminated union on `t` |
| `#[serde(tag = "t", content = "c")]` | `{"t": "Variant", "c": data}` | Adjacent-tagged union |
| `#[serde(untagged)]` | `data` (no tag) | Plain union |

**Rename transforms:**

`#[serde(rename_all = "...")]` is supported at the struct, enum, and variant level
with all standard serde conventions: `camelCase`, `snake_case`,
`SCREAMING_SNAKE_CASE`, `kebab-case`, `PascalCase`, `lowercase`, `UPPERCASE`.

**Field/variant attributes:**

| Attribute | Effect on TypeScript |
|---|---|
| `#[serde(rename = "name")]` | Uses the renamed field/variant name |
| `#[serde(skip)]` | Omits the field/variant entirely |
| `#[serde(skip_serializing)]` | Omits the field (it won't appear in output) |
| `#[serde(default)]` | Makes the field optional (`field?: Type`) |
| `#[serde(flatten)]` | Produces intersection type (`& OtherType`) |
| `#[serde(transparent)]` | Struct becomes its single field's type |

### Escape Hatch

For cases the derive macro cannot handle (custom serializers via `#[serde(with)]`,
third-party types, etc.), an explicit override is available:

```rust
#[derive(Serialize, Deserialize, neon::TypeScript)]
struct MyStruct {
    name: String,
    #[neon(ts_type = "string")]
    timestamp: DateTime<Utc>,  // chrono serializes as string
}
```

This can also be applied at the type level:

```rust
#[derive(Serialize, Deserialize, neon::TypeScript)]
#[neon(ts_type = "Record<string, FieldDescriptor>")]
struct SchemaMap(OrderMap<String, FieldDescriptor>);
```

## Generics

Rust types used with `Json<T>` may be generic:

```rust
#[derive(Serialize, Deserialize, neon::TypeScript)]
struct Page<T> {
    items: Vec<T>,
    total: u64,
}
```

The derive macro preserves type parameters in the generated TypeScript:

```typescript
interface Page<T> {
    items: T[];
    total: number;
}
```

When a concrete instantiation like `Json<Page<SearchResult>>` appears in an export
signature, the generated declaration references the generic interface with a concrete
argument: `Page<SearchResult>`.

The derive macro must:
- Identify type parameters from the struct/enum definition
- Preserve them as symbolic placeholders during recursive type resolution
- Handle nested generic usage (e.g., `Option<Vec<T>>` becomes `T[] | undefined | null`)

## Third-Party Type Support

When an export uses a type from an external crate (e.g., `chrono::DateTime`,
`indexmap::IndexMap`, `uuid::Uuid`), our derive macro has no opportunity to run on
that type. There are three mechanisms to handle this:

1. **Built-in impls behind feature flags.** Neon can ship `TypeScript` implementations
   for popular crates: `neon = { features = ["typescript-chrono", "typescript-uuid", "typescript-ordermap"] }`.
   Each impl is a few lines mapping to the obvious TS type.

2. **Manual trait implementation.** Users can implement `TypeScript` for any type in
   their own crate (subject to orphan rules -- typically via a newtype wrapper).

3. **Field-level override.** The `#[neon(ts_type = "...")]` escape hatch works for
   any field regardless of its Rust type.

The initial release should ship with impls for `serde_json::Value` (mapped to `any`)
and standard collection types. Additional crate support can be added based on demand.

## Feature Flag Design

TypeScript generation is behind a `typescript` feature flag on the `neon` crate:

```toml
[dependencies]
neon = { version = "...", features = ["typescript"] }
```

When enabled:
- `#[neon::export]` emits type metadata entries alongside existing export registrations
- The `neon::TypeScript` derive macro becomes available
- The `neon::typescript::generate()` function is available
- Binary size increases slightly due to metadata

When disabled (default): no additional code is generated, no additional dependencies
are pulled in, and there is zero impact on existing users.

## Prior Art and Ecosystem Considerations

Several Rust crates provide TypeScript generation from Rust types:

- [**specta**](https://github.com/specta-rs/specta) provides a language-agnostic type
  introspection system with TypeScript as one of many target languages. Its architecture
  uses an intermediate data type representation and supports advanced features like
  branded types and function signatures. It's designed to integrate with RPC frameworks
  like rspc and Tauri.

- [**ts-rs**](https://github.com/Aleph-Alpha/ts-rs) takes a more focused approach,
  generating TypeScript strings directly from derive macros without an intermediate
  representation. It's TypeScript-only by design, which keeps the API surface small.

- [**typeshare**](https://github.com/1Password/typeshare) uses source-level parsing
  via a CLI tool rather than proc macros, which avoids compilation but limits its
  ability to resolve types through macros or trait implementations. It targets multiple
  languages for cross-platform FFI.

All three are well-engineered projects. We are choosing to implement Neon's own
`TypeScript` trait and derive macro for several reasons:

- **Neon has domain-specific needs** that general-purpose crates don't address: boxed
  types via smart pointers, async task functions, class exports, and the `Json<T>`
  extraction boundary. These would require significant adapter logic on top of any
  external crate.
- **Minimal dependency footprint.** Neon is a foundational library, and adding a large
  transitive dependency tree has a real cost for all downstream users. A focused,
  Neon-specific implementation keeps the dependency surface small.
- **Stability requirements.** As a widely-used crate, Neon needs its TypeScript
  generation to be stable and predictable. Depending on another project's release
  cycle adds coordination overhead.

That said, the `TypeScript` trait is designed so that **compatibility layers with
existing crates could be added in the future** via blanket implementations behind
feature flags (e.g., `impl<T: specta::Type> TypeScript for T`). This keeps the door
open without committing to a dependency today.

## Class Exports

The `#[neon::class]` macro knows the full class structure at compile time: constructor
signature, methods with their receiver types and attributes, and const properties. This
is enough to generate complete TypeScript `class` declarations.

### What the Macro Knows

For each class, the macro has access to:

- **Class name** (and optional JS name override via `#[neon::export(class(name = "..."))]`)
- **Constructor**: parameter names and types, `json` attribute, fallibility (`Result<Self, E>`)
- **Methods**: receiver type (`&self`, `&mut self`, `self`), parameter names and types,
  return type, and attributes (`name`, `json`, `async`, `task`, `context`, `this`)
- **Const properties**: name, type, `json` attribute, optional JS name override

### Generated TypeScript

A Rust class like:

```rust
#[neon::export(class)]
struct Point { x: f64, y: f64 }

#[neon::class]
impl Point {
    const ORIGIN_X: f64 = 0.0;

    #[neon(name = "maxCoordinate")]
    const MAX_COORD: f64 = 1000.0;

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: &Self) -> f64 { ... }

    pub fn translate(&mut self, dx: f64, dy: f64) { ... }

    #[neon(json)]
    pub fn to_object(&self) -> PointData { ... }

    #[neon(task)]
    pub fn heavy_computation(self) -> f64 { ... }
}
```

Would generate:

```typescript
declare class Point {
    constructor(x: number, y: number);

    static readonly ORIGIN_X: number;
    static readonly maxCoordinate: number;

    distance(other: Point): number;
    translate(dx: number, dy: number): void;
    toObject(): PointData;
    heavyComputation(): Promise<number>;
}
```

### Design Details

**Hidden parameters.** Parameters marked with `#[neon(context)]` or `#[neon(this)]`
are internal Neon plumbing and do not appear in the TypeScript signature. The macro
already knows which parameters these are.

**Receiver type is invisible.** Whether a method takes `&self`, `&mut self`, or `self`
is a Rust implementation detail. All three appear the same in TypeScript -- as an
instance method with no special receiver syntax.

**Async and task methods.** Methods declared as `async fn` or annotated with
`#[neon(task)]` return a `Promise<T>` in TypeScript, where `T` is the resolved
return type.

**JSON methods.** When a method has `#[neon(json)]`, its parameters and return type
pass through serde serialization. The TypeScript types for these are resolved via the
`TypeScript` trait on the underlying types, exactly as with `Json<T>` in function
exports.

**Fallible constructors.** A constructor returning `Result<Self, E>` produces a
TypeScript constructor with just the parameters -- errors become thrown exceptions,
not part of the type signature.

**Reference parameters.** When a method takes `&OtherClass` or `&mut OtherClass`,
the TypeScript parameter type is just `OtherClass`. The reference/mutability is a
Rust-side concern.

**Const properties.** These become `static readonly` properties on the TypeScript
class. Their types are resolved the same way as function return types. When
`#[neon(json)]` is present, the type is determined by the `TypeScript` trait impl
on the value type.

### Classes as Types in Other Signatures

When a class type appears as a parameter or return type in a standalone
`#[neon::export]` function, it should be typed as the class name (not as an opaque
boxed type). For example:

```rust
#[neon::export]
fn distance_between(a: &Point, b: &Point) -> f64 { ... }
```

Should generate:

```typescript
declare function distanceBetween(a: Point, b: Point): number;
```

The `#[neon::class]` macro generates a `TypeScript` impl for the class where
`ts_type()` returns the class name and `ts_decl()` returns the full
`declare class { ... }` block. This means classes integrate naturally with the
trait-based type resolution used by function exports.

Non-class boxed types (plain `Arc<T>` where `T` is not a Neon class) still use the
branded interface pattern described in "Boxed Types" above.

### Metadata Collection for Classes

Class metadata follows the same `linkme::distributed_slice` pattern as function
exports, but with a richer structure:

```rust
#[linkme::distributed_slice(TYPE_METADATA)]
static __META_Point: ExportMeta = ExportMeta::Class(ClassMeta {
    name: "Point",
    constructor: Some(ConstructorMeta {
        params: &[
            ParamMeta { name: "x", ts_type: || f64::ts_type() },
            ParamMeta { name: "y", ts_type: || f64::ts_type() },
        ],
    }),
    methods: &[
        MethodMeta {
            name: "distance",
            params: &[ParamMeta { name: "other", ts_type: || Point::ts_type() }],
            ret: || f64::ts_type(),
            is_async: false,
        },
        // ...
    ],
    static_properties: &[
        PropertyMeta { name: "ORIGIN_X", ts_type: || f64::ts_type() },
        PropertyMeta { name: "maxCoordinate", ts_type: || f64::ts_type() },
    ],
});
```

## Output Format

The default output is a **`.d.ts` declaration file** intended to sit beside the
`.node` binary module. Top-level items are emitted as named `export declare`
statements (and `export interface` / `export type` for collected types):

```typescript
// Auto-generated by Neon. Do not edit.

export declare const __neon_tag: unique symbol;
export interface BoxedSchema { readonly [__neon_tag]: 'Schema' }

export interface SearchResult { ... }

export declare function newSchema(schema: SchemaDescriptor): BoxedSchema;
export declare function commit(index: BoxedIndex): Promise<void>;
// ...

export declare class Point {
    constructor(x: number, y: number);
    distance(other: Point): number;
    // ...
}
```

To attach these declarations to a specific module path instead of emitting
top-level exports (e.g. when the addon is loaded via a `load.cjs` shim), pass
`GenerateOptions { module: Some("./load.cjs".into()) }` to
[`generate_with`](#generation-api); the body is then wrapped in
`declare module "./load.cjs" { ... }`.

```rust
pub struct GenerateOptions {
    /// Wrap output in `declare module "<name>" { ... }`.
    pub module: Option<String>,
}
```

## Stability

The **Rust API** (the `TypeScript` trait, `generate()` / `generate_with()` /
`generate_ast()` / `generate_ast_with()`, the metadata types, and the macro
attributes) follows the crate's normal semver guarantees.

The **generated output format** — the exact text of the `.d.ts` and the shape of
the AST — is **not yet covered by semver** and may change between releases while
the feature settles in real-world use. Two changes are already anticipated:

- Promoting the transitional `Decl::Raw` interface/type-alias nodes in the AST to
  structured `TSInterfaceDeclaration` / `TSTypeAliasDeclaration` variants.
- The synthesized names for *anonymous or composed* boxed types (e.g.
  `Arc<Option<String>>` currently rendering as `Boxedstringundefinednull`). Names
  for *named* boxed types (`Arc<Database>` → `BoxedDatabase`) are stable; only the
  best-effort names for composed types are subject to change.

If you commit generated declarations to source control, expect to regenerate them
across Neon upgrades until the format is declared stable.

## Resolved Questions

- **`Option<T>` semantics.** `Option<T>` maps to `T | undefined | null`, matching
  Neon's extraction behavior (both JS `undefined` and `null` deserialize to `None`).

- **Async functions.** Exports marked `#[neon::export(task)]` (and `async fn`)
  render as `Promise<T>`. A unit return normalizes to `Promise<void>`.

## Implementation Stages

The implementation can be broken into incremental stages, each delivering usable
value:

### Stage 1: Functions with Primitive Types

- `TypeScript` trait definition with built-in impls for primitives
- Metadata collection in `#[neon::export]` behind feature flag
- `generate_typescript()` producing a `.d.ts` file
- Supports: `f64`, `i32`, `u32`, `u8`, `bool`, `String`, `()`, `Option<T>`, `Vec<T>`,
  `Result<T, E>`, opaque boxed types via `Arc<T>`/`RefCell<T>`
- Async/task functions emit `Promise<T>` return types

### Stage 2: Serde Types via Derive Macro

- `#[derive(neon::TypeScript)]` macro with serde attribute parsing
- MVP serde support: `rename_all`, `rename`, `skip`, `tag` (internally tagged enums),
  `transparent`
- `#[neon(ts_type = "...")]` escape hatch for unsupported cases
- `Json<T>` delegation to the `TypeScript` trait

### Stage 3: Class Exports

- Metadata collection in `#[neon::class]` for constructors, methods, and properties
- `TypeScript` impl generation for classes
- Classes usable as types in function signatures

### Stage 4: Extended Serde and Ecosystem Support

- Remaining enum representations (adjacently tagged, untagged, externally tagged)
- `flatten`, `default`, `skip_serializing`/`skip_deserializing`
- Feature-flagged `TypeScript` impls for popular crates (chrono, uuid, etc.)
- Generic type parameter preservation in derive macro
