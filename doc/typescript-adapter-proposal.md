# Proposal: minimal `TypeScript` trait + third-party adapters

**Status: proposal under discussion (not yet implemented).** This note proposes
an alternative to the derive-macro approach currently in this PR. It is here so
the maintainers can react to it inline before any code moves.

## The problem this addresses

The current PR ships a `#[derive(neon::TypeScript)]` macro that understands serde
attributes (`rename`, `rename_all`, `tag`, `content`, `untagged`, `flatten`,
`skip`, `default`, `transparent`) and the four enum tagging modes. That derive is
the single largest and most volatile part of the feature: keeping it correct is
an **open-ended maintenance treadmill** that tracks the evolution of serde and
Rust. It is also tangential to Neon's mission. We would prefer Neon not own it.

The obvious alternative — depend on an existing Rust→TypeScript crate — was
researched (see the Prior Art section of `typescript.md` and the notes below):

- **Specta** has the ecosystem (Tauri/rspc) but no dependable release: no stable
  2.0, exact-pinned churning release candidates, adoption fragmented across RCs.
  Not something to put in a foundational crate's public API today.
- **ts-rs** is the opposite: a healthy, semver-normal, widely-used crate (~13M
  downloads, real majors) — but a *standalone generator* with no consumer
  ecosystem to interoperate with.

So neither is a clean "just depend on it." But we can get the best of both by
**owning only a tiny stable contract and sourcing the type information from
whichever third-party crate the user already uses.**

## Proposed architecture

### Crate layout

```
neon-typescript          the stable contract: the `TypeScript` trait plus
  (no third-party deps)   built-in impls for std/core types. Rarely changes.
     ▲            ▲
     │            │
   neon        neon-ts-rs / neon-specta     the adapters
```

- **`neon-typescript`** — the trait and its impls for std/core types
  (primitives, `String`, `Vec`, `Option`, `HashMap`, tuples, …). No dependency
  on serde, ts-rs, specta, or `neon`. This is the contract everything binds to,
  so it is deliberately minimal and semver-stable.
- **`neon`** — depends on `neon-typescript`. Adds impls for its own boundary
  types (`Handle<Js*>`, `Boxed`, `Json<T>`), the `generate()` / `generate_ast()`
  machinery, the auto-attach, and the `#[neon::export]` / class metadata macros.
  Re-exports the trait as `neon::typescript::TypeScript`.
- **`neon-ts-rs`** — a small adapter. Provides a *bridge derive* and a runtime
  collection helper. Depends on `neon-typescript` and references `ts-rs` in the
  code it generates — **not on `neon`**. Its version tracks ts-rs: `neon-ts-rs`
  for ts-rs 12, a new major when ts-rs 13 breaks the `TS` trait. Neon core never
  moves for a ts-rs bump.
- **`neon-specta`** — the same shape against `specta::Type`, shipped later. Its
  worse version story is fully quarantined inside this crate.

The load-bearing property: the volatile dependency is only ever referenced by an
adapter crate whose entire job is to track it. Neon's own crates never depend on
ts-rs or specta.

### The minimal trait

```rust
pub trait TypeScript {
    /// The TypeScript type *expression*: "number", "string[]", "SearchResult", …
    fn ts_type() -> Cow<'static, str>;

    /// Add this type's own declaration (if any) plus its transitive
    /// declarations to `decls`, keyed by name for dedup.
    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        if let Some(d) = Self::ts_decl() {
            decls.entry(Self::ts_type().into_owned()).or_insert_with(|| d.into_owned());
        }
    }

    fn ts_decl() -> Option<Cow<'static, str>> { None }
}
```

No derive, no serde parsing, no attribute logic — and no AST — lives here. The
structured AST (where wanted) is produced in `neon` by parsing these strings, so
the contract crate stays string-only (see Alternatives Considered §1). That is
the point: this crate is small enough to be trivially stable.

### The adapter is a bridge derive, not a boundary wrapper

Rust's orphan rule forbids `impl<T: ts_rs::TS> TypeScript for T` in an adapter
crate (and it would collide with the built-in impls anyway). But the *user owns
their type*, so a derive can generate the bridge impl in **their** crate:

```rust
// neon-ts-rs provides a trivial derive that generates, for the user's MyType:
impl neon_typescript::TypeScript for MyType {
    fn ts_type() -> Cow<'static, str> { neon_ts_rs::ts_type::<Self>() }
    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        neon_ts_rs::ts_collect::<Self>(decls);
    }
}
```

`neon_ts_rs::ts_type` / `ts_collect` delegate to `<Self as ts_rs::TS>` (the
runtime helper is ~40 lines — a `TypeVisitor` that assembles a flat declaration
set, validated in the spike below). This bridge derive is *trivial*: it reads
none of the type's fields or serde attributes; it emits a fixed delegation. All
serde understanding stays in ts-rs. Because the generated impl references only
`neon-typescript` + `ts-rs`, the adapter crate never depends on `neon`, and
extraction stays the user's ordinary `Json<T>`.

### What the user writes

```rust
// ts-rs path (the healthy option): one extra derive, nothing at the boundary.
#[derive(Serialize, Deserialize, ts_rs::TS, neon_ts_rs::TypeScript)]
#[serde(rename_all = "camelCase")]
struct SearchResult { doc_id: u32, /* … */ }

#[neon::export(json)]
fn search(q: String) -> SearchResult { /* … */ }   // Json extraction unchanged
```

- **ts-rs user:** derive `ts_rs::TS` + `neon_ts_rs::TypeScript`; types are also
  visible to any ts-rs tooling.
- **specta user:** derive `specta::Type` + `neon_specta::TypeScript`; types plug
  into the specta/Tauri ecosystem — the interop we want, dependency quarantined.
- **neither:** hand-write `impl neon::typescript::TypeScript for MyType` (just
  `ts_type` + `ts_decl`). We are intentionally out of the derive business; a user
  with no type-gen crate hand-writes the impl.

### Adapter responsibilities (the small, bounded surface Neon keeps)

ts-rs is faithful to the *Rust type*; Neon's boundary is faithful to *what
serde_json actually serializes*. Those differ in essentially one fixed place, so
the adapter carries a tiny bit of reconciliation:

- Configure large ints to `number` (ts-rs defaults `u64`/`i64` to `bigint`, but
  serde_json emits JSON numbers). Confirmed fixable via `Config::with_large_int`.

That is nearly the whole list. Two things the adapter deliberately does **not**
do (see Alternatives Considered §2 and §3):

- It does **not** add `Option` leniency. Optionality is expressed by the user with
  standard serde attributes (`skip_serializing_if` + `default`), which ts-rs
  already honors; the adapter passes it through unchanged.
- It does **not** normalize output styling; bridged types render in their
  provider's idiom.

This is a *fixed* surface — it does not grow when serde adds an attribute. The
treadmill is gone.

## What changes in the current PR

- **Remove:** `crates/neon-macros/src/typescript/*` (structs / enums / attrs /
  rename — the derive + serde parsing), ~1,500 lines and the churniest code.
- **Keep:** the trait + built-in impls (moved into `neon-typescript`), the
  boundary impls, `generate()` + the string→AST parser, auto-attach, and the
  `#[neon::export]` / class metadata macros. The structured AST is generated in
  `neon` by parsing strings (Alternatives Considered §1). Whether `generate_ast()`
  ships in v1 at all is worth revisiting separately, since the dogfooding consumes
  only the string form (see §1).
- **Add:** the `neon-typescript` and `neon-ts-rs` crates.

The PR shrinks and loses its riskiest code, but the type-provider half is
genuinely reworked — this is a real redirection, not a tweak.

## Spike validation

A throwaway spike (real ts-rs 12 from crates.io) implemented the minimal trait,
the `neon-ts-rs` collection helper, and a bridged type, then compared ts-rs's
output against `serde_json`'s actual serialization:

| Feature | serde_json runtime | ts-rs type | Verdict |
|---|---|---|---|
| `rename_all = "camelCase"` | `docId` | `docId` | match |
| `#[serde(skip)]` | field omitted | field omitted | match |
| internally-tagged enum | `{"kind":"circle",…}` | `{ "kind": "circle", … }` | match |
| `Vec<String>` | `["a"]` | `Array<string>` | match (cosmetic) |
| `u64` | `42` (JSON number) | `bigint` (default) | **mismatch — configurable** |
| `Option<u32>` | key present, `null` | `number \| null` | accepted; leniency is user opt-in via serde (§2) |

The design works end-to-end (~40 lines of adapter), the flat-declaration
assembly works via ts-rs's `TypeVisitor`, and the one sharp fidelity gotcha
(large ints) is fixable via ts-rs `Config`.

## Alternatives Considered

Three design questions came up while working through this proposal. Each is
resolved below, with the alternatives we weighed and why we chose what we chose.

### 1. Where the structured AST lives — string-only trait, parse in `neon`

**Decision.** The trait is string-only (`ts_type` + `ts_collect`). The structured
AST (where wanted) is produced in `neon` by running the string→AST parser over
those strings. The AST node types and parser stay out of `neon-typescript`.

**Why the question arises.** The feature produces two outputs: a `.d.ts` *string*
(`generate()`) and a *structured AST* (`generate_ast()` — TSESTree-shaped, for
programmatic transforms). Bridged types (ts-rs/specta) naturally produce only
*strings*, so the AST for any bridged type can only come from *parsing its
string* anyway.

**Alternative rejected — put `ts_type_ast()` in the trait**, with a parse-based
default that built-ins override for native structured nodes. This would drag the
AST node types **and the parser** into `neon-typescript`, inflating the crate we
most want to keep minimal and tying its semver to the AST representation. And
since bridged types are parser-sourced regardless, making built-ins specially
native buys inconsistency ("built-ins native, bridged parsed"), not much
fidelity — the parser already handles the built-in shapes (primitives, arrays,
unions, records, tuples, literals) correctly.

**Consequence.** AST behavior is uniform (everything parsed by one well-tested
parser) and the contract crate stays tiny. The tradeoff: the AST is only ever as
good as the parser — any expression it cannot structure becomes a `Raw` node.

**Related scoping note (not part of this decision).** The
[dogfooding PR](https://github.com/dherman/tantivy/pull/3) consumes only the
string (`Symbol.for("neon:types")`) and never touches `generate_ast()` /
`Symbol.for("neon:types-ast")`; its `extract-types.cjs` hand-rolls the
`declare module "./load.cjs" { … }` wrap and indentation that the AST and
`generate_with({ module })` were meant to obviate. So the AST is currently unused
in practice, which argues for **deferring `generate_ast()` from v1** and instead
wiring the *module-scoped* string into the auto-attach (the transform the
dogfooding actually needs). That is a scoping call separate from the trait-shape
decision above, recorded here for follow-up.

### 2. `Option` leniency — user-expressible via serde, adapter passes through

**Decision.** The adapter owns no `Option`/leniency policy. Optionality is
expressed by the user with standard serde attributes and passed through
faithfully by whatever generator produced it.

**Why this works (spike evidence).** ts-rs already reflects serde's optionality
in the type, and it does so *accurately* — a field is marked TypeScript-optional
(`?`) exactly when it can be absent in **both** directions:

| Field attributes | serde output when `None` | ts-rs type |
|---|---|---|
| plain `Option<T>` | `"f": null` (present) | `f: T \| null` |
| `skip_serializing_if = "Option::is_none"` | key omitted | `f: T \| null` (still required) |
| `skip_serializing_if` **+** `default` | key omitted | `f?: T \| null` |
| `default` only | `"f": null` (present) | `f: T \| null` |

The idiomatic serde spelling of a fully optional field —
`#[serde(skip_serializing_if = "Option::is_none", default)]` — yields
`f?: T | null`, which accepts a value, `null`, `undefined`, *or* omission (the
`?` already covers `undefined`, so no `| undefined` is needed) and is accurate in
both directions. The user controls leniency; the adapter does nothing.

**Alternatives rejected.**
- *Post-process in the adapter* to make every `Option` lenient (add `| undefined`,
  or add `?` via a "nullable ⇒ optional" heuristic). This bakes a uniform policy
  that is loose for return types, reintroduces an output-format transform in the
  adapter, and — because ts-rs has already collapsed `Option` to a string —
  cannot even distinguish an `Option` from any other nullable.
- *Separate input/output types* (the precise fix for serde's serialize/deserialize
  asymmetry). Correct in principle but a large feature; out of scope for now.

**Consequence.** Leniency is opt-in, not automatic: a plain `Option<T>` renders as
`T | null` (required key — conservative but honest; the runtime still accepts
absent/null → `None`). Notably this is *more* correct than the in-PR derive, which
marks a field optional on `default` alone even though such a field is still
present-as-`null` in the output. Neon (and the adapters) stay entirely out of the
leniency-policy business — squarely aligned with the minimal-trait direction.

### 3. Bridged output styling — accept each provider's idiom

**Decision.** Do not normalize. Bridged types render in their provider's style
(`Array<string>`, quoted discriminant keys, its own map formatting), mixing with
Neon's own style for its boundary types.

**Why.** The alternatives are worse:
- *Normalize* bridged output to Neon's style — fragile string-rewriting that
  reintroduces exactly the "understand and rewrite the output format" ownership we
  are trying to shed (a small treadmill of its own).
- *Adopt the provider's style wholesale* — not possible, since Neon's own boundary
  types (`Handle`, `Boxed`, class output) are Neon-styled and stay that way.

Accepting the mixed idiom is the only option that keeps Neon out of output-format
logic. The cost is cosmetic inconsistency across a generated file (consistent
within any one type).

**AST interaction.** Because bridged types are string-sourced (§1), their AST
nodes come from parsing the provider's strings — `Array<string>` parses to a
`TSTypeReference` rather than a `TSArrayType`, and anything the parser cannot
structure becomes `Raw`. So the AST for bridged types is slightly less structured
than for built-ins — consistent with §1's uniform, best-effort stance.

## Recommendation / next step

If this shape looks right, the natural next step is a small two-crate skeleton
(`neon-typescript` trait + `neon-ts-rs` bridge derive) wired to one real
`#[neon::export]` in the test addon, to confirm the derive hygiene and the
`Json<T>` + bridge interaction end-to-end before reworking the PR.
