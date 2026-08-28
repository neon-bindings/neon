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

No derive, no serde parsing, no attribute logic lives here. That is the point.

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

The adapter is not a transparent pass-through. ts-rs is faithful to the *Rust
type*; Neon's boundary is faithful to *what serde_json actually serializes*.
Those differ in a few fixed places, so each adapter carries a little
reconciliation logic (a few dozen lines and a couple of policy constants):

- Configure large ints to `number` (ts-rs defaults `u64`/`i64` to `bigint`, but
  serde_json emits JSON numbers). Confirmed fixable via `Config::with_large_int`.
- Decide the `Option` representation (see Open Question 2).
- Optional cosmetic normalization (see Open Question 4).

This is a *fixed* surface — it does not grow when serde adds an attribute. The
treadmill is gone.

## What changes in the current PR

- **Remove:** `crates/neon-macros/src/typescript/*` (structs / enums / attrs /
  rename — the derive + serde parsing), ~1,500 lines and the churniest code.
- **Keep:** the trait + built-in impls (moved into `neon-typescript`), the
  boundary impls, `generate()` / `generate_ast()` / parser, auto-attach, and the
  `#[neon::export]` / class metadata macros.
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
| `Option<u32>` | key present, `null` | `number \| null` | differs from Neon's `T \| undefined \| null` |

The design works end-to-end (~40 lines of adapter), the flat-declaration
assembly works via ts-rs's `TypeVisitor`, and the one sharp fidelity gotcha
(large ints) is fixable via ts-rs `Config`.

## Open questions

### 1. Where does the structured AST live?

**Background.** The feature produces two outputs: a `.d.ts` *string* (via
`generate()`) and a *structured AST* (via `generate_ast()` — TSESTree-shaped,
serde-serializable, for programmatic transforms). The AST is possible because
each type can expose a structured node (`ts_type_ast()`) in addition to its
string (`ts_type()`). Neon's built-in impls produce native structured nodes; a
string→AST parser exists as a fallback for anything that only produces strings.

**Why the adapter forces a decision.** Bridged types (ts-rs/specta) naturally
produce only *strings* — ts-rs hands you `"Array<string>"` and declaration
strings, not Neon AST nodes. (specta has its own IR we could map, but that is
real per-adapter work, and ts-rs gives strings regardless.) So the structured
AST for any bridged type can only come from *parsing its string*. Given that,
where should the AST machinery live?

- **(a) Keep `ts_type_ast()` in the trait**, with a default that parses
  `ts_type()`. Built-ins override it for exact structured output; bridged types
  use the parse default. *Cost:* the AST node types **and the parser** must live
  in `neon-typescript` (the contract crate), because the trait method returns AST
  nodes and defaults to parsing. That inflates the crate we most want to keep
  tiny and stable, and ties its semver to the AST representation.
- **(b) Keep the trait string-only** (`ts_type` + `ts_collect`) and generate the
  AST entirely inside `neon` by parsing the strings. *Cost:* every type's AST is
  parser-derived — even built-ins lose their "native" structured nodes and go
  through the parser like everything else. In exchange, `neon-typescript` stays
  minimal, and AST behavior is *uniform* (everything parsed) rather than
  "built-ins native, bridged parsed."

**Lean: (b).** Since bridged types are parser-sourced no matter what, having
built-ins be specially native buys inconsistency, not much fidelity — the parser
already handles the built-in shapes (primitives, arrays, unions, records, tuples,
literals) correctly. (b) keeps the contract crate small, which is the whole
motivation, and makes AST fidelity a single well-tested parser rather than two
code paths. The tradeoff to weigh: (b) means the AST is only ever as good as the
parser, so any type expression the parser cannot structure becomes a `Raw` node.

### 2. What should `Option<T>` mean, and how do we keep it consistent?

**Background — what serde actually does.**
- Default `Option<T>` (no `skip_serializing_if`): `Some(x)` serializes to
  `"field": x`; `None` serializes to `"field": null`. **The key is always
  present.** So the observed JSON shape is `field: T | null`.
- With `#[serde(skip_serializing_if = "Option::is_none")]`: `None` omits the key
  entirely, so the shape is `field?: T` (key may be absent).

**Where the two crates differ.**
- **Neon's current impl:** `Option<T>` → `T | undefined | null`. That union was
  chosen for *deserialization leniency* (both JS `undefined` and `null`
  deserialize to `None`). But for a value the addon *returns*, `undefined`
  implies the key can be absent — which is only true under `skip_serializing_if`.
  So for the common (default) case, `T | undefined | null` slightly overstates
  optionality.
- **ts-rs:** `Option<T>` → `T | null`, matching the default serialization (key
  present, value possibly null), and handles `skip_serializing_if` separately by
  making the field optional (`field?`).

**Why it matters.** Three things: (1) adopting ts-rs bridging changes `Option`
output from `T | undefined | null` to `T | null` — a visible behavior change;
(2) which is "correct" is direction-sensitive — for a *returned* value ts-rs's
`T | null` is more accurate, while for a *parameter* the caller passes in,
`undefined` leniency is often desirable; and most importantly (3) if Neon's
built-in impls keep `T | undefined | null` while ts-rs-bridged types emit
`T | null`, a single generated `.d.ts` will represent `Option` *inconsistently*
depending on whether a type came from a built-in or a bridge. That inconsistency
is the real problem to resolve. Options: align Neon's built-ins to ts-rs's
`T | null`; post-process/configure the adapter to emit `T | undefined | null`; or
declare it explicit adapter policy and document the difference.

### 4. Do we normalize bridged output styling, or accept each provider's idiom?

**Background.** The spike showed ts-rs renders semantically-identical TypeScript
in a different *style* than Neon's hand-written impls: `Array<string>` vs
`string[]`, quoted discriminant keys (`"kind"`), its own formatting for maps,
etc. These are cosmetic — the types mean the same thing — but they mean a single
generated `.d.ts` would mix styles: a function returning `Vec<String>` (a Neon
built-in) shows `string[]`, while a field inside a bridged struct shows
`Array<string>`.

**The choice.**
- **(a) Normalize** bridged output to Neon's style — i.e. post-process the
  strings ts-rs produces. *Cost:* fragile string munging, and it re-introduces
  exactly the kind of "understand and rewrite the output format" logic we are
  trying to stop owning — a small treadmill of its own.
- **(b) Accept the mixed style** — each type renders in its provider's idiom
  (consistent within a type, varying across the file). *Cost:* stylistic
  inconsistency in the output, which some consumers may find untidy.
- **(c) Adopt the provider's style wholesale** — not fully possible, since Neon's
  own boundary types (`Handle`, `Boxed`, class output) are Neon-styled and stay
  that way.

**AST interaction.** Because bridged types are string-sourced, their AST nodes
come from parsing ts-rs's strings — so `Array<string>` parses to a
`TSTypeReference` to `Array<T>` rather than a `TSArrayType`, and any construct
the parser cannot structure becomes a `Raw` node. So the styling choice also
affects how uniform the *AST* is across built-in vs bridged types.

**Lean: (b), accept the mixed idiom.** Normalizing fights the entire premise
(don't own output-format logic). Document that bridged types render in their
provider's idiom.

## Recommendation / next step

If this shape looks right, the natural next step is a small two-crate skeleton
(`neon-typescript` trait + `neon-ts-rs` bridge derive) wired to one real
`#[neon::export]` in the test addon, to confirm the derive hygiene and the
`Json<T>` + bridge interaction end-to-end before reworking the PR.
