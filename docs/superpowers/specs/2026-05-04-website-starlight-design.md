# Neon docs site — Starlight rebuild

Date: 2026-05-04
Status: Approved (pending user review of this written spec)

## Goal

Replace the existing Neon docs site at <https://neon-rs.dev> (currently
[`neon-bindings/website`](https://github.com/neon-bindings/website), a Docusaurus
project) with a brand-new Starlight site that lives in this monorepo at
`website/`. The new site exemplifies modern Neon (1.2+: `#[neon::export]`,
`#[neon::class]`, async exports), follows
[Diátaxis](https://diataxis.fr/) for information architecture, and verifies
every Rust code sample with `cargo test` doctests so the site can never silently
drift from the API it documents.

## Non-goals

- Porting any content from the existing site. Content is written fresh.
- Internationalization (i18n / Crowdin). Not in v1.
- A blog. Not in v1.
- Migration guides for older Neon versions on the new site. Existing
  `doc/MIGRATION_GUIDE_*.md` files stay where they are.
- How-to guides for Electron- or Bun-specific projects in v1.
- Testing or showcasing community-built Neon modules.

## Audience & framing

The landing page is **product-first**, not audience-segmented. A visitor sees
what Neon is and what code looks like, then self-routes into the appropriate
Diátaxis quadrant. There is no "I'm coming from Rust" / "I'm coming from
Node" split.

Inside the docs, content assumes a reader comfortable with at least one of Rust
or Node and willing to learn the other. We do not teach Rust or Node from
scratch.

## Information architecture (Diátaxis)

Top-level sidebar groups, in order:

### Getting started

- Install (toolchain, `npm init neon@latest`).
- Quickstart (a few minutes; produces a runnable hello-world without explaining
  why).

### Tutorials (learning-oriented, linear)

1. **Your first Neon module.** Install toolchain → `npm init neon` → export a
   function with `#[neon::export]` → call it from JS → run it.
2. **Concurrency with the libuv thread pool.** Convert a CPU-bound function to
   `#[neon::export(task)]`, await the resulting Promise from JS.
3. **Async functions with tokio.** Register a global executor, write
   `async fn`, return a Promise.

### How-to guides (task-oriented, recipes)

- Pass and return common types (numbers, strings, arrays, objects, buffers).
- Use `serde` with `Json<T>` (and the `json` shorthand attribute).
- Define a class with `#[neon::class]`.
- Run blocking work on the libuv pool (`#[neon::export(task)]`).
- Run an `async fn` and return a Promise.
- Run synchronous setup on the JS main thread before async work
  (`#[neon::export(async)]` returning `impl Future`).
- Run code on the JS main thread *after* async work (`extract::with`).
- Cancel async work with `AbortController` (mirrors
  [`neon-bindings/examples` PR #104](https://github.com/neon-bindings/examples/pull/104)).
- Stream data between Rust and JS.
- Throw and catch JS errors from Rust (including `extract::Error` + `?`).
- Rename exports / customize the JS-facing name.
- Access `Cx` / `FunctionContext` from an exported function.
- Implement `this`-style methods (the `this` extractor / attribute).
- Publish a prebuilt binary to npm.

### Reference (information-oriented)

- API reference (rustdoc, mounted via `starlight-rustdoc` at `/api/`).
- Supported platforms (OS matrix, Node-API versions, MSRV).
- CLI reference for `create-neon` and `cargo-cp-artifact`.

### Explanation (understanding-oriented)

- What Neon is, and how it relates to Node-API.
- Threading and lifecycle (`Channel`, `Root`, `Deferred`).
- How `#[neon::export]` works under the hood.
- Error-handling philosophy (Rust `Result` ↔ JS exceptions).
- When to reach for Neon (and when not to).

### Top-level pages outside Diátaxis

- Home (custom landing).
- Changelog.
- Contributing (with a Slack invite link).

## Architecture

### Repository layout

```
website/
├── Cargo.toml          # rustc-side: makes `website` a workspace member
├── build.rs            # walks src/content/docs/ and emits doctests to OUT_DIR
├── package.json        # Astro/Starlight site
├── astro.config.mjs    # Starlight config + remark plugin + starlight-rustdoc
├── strip-hidden-rust-lines.mjs  # remark plugin (hidden-line stripping)
├── netlify.toml        # build command, publish dir, Rust toolchain pin
├── public/             # static assets (logo, favicon)
└── src/
    ├── content/
    │   ├── config.ts   # Starlight content collections schema
    │   └── docs/       # all markdown — single source of truth for site & doctests
    │       ├── index.mdx
    │       ├── getting-started/
    │       ├── tutorials/
    │       ├── how-to/
    │       ├── reference/
    │       └── explanation/
    └── components/     # custom landing-page components
```

The `website/` directory is **simultaneously** an Astro project (consumed by
Node tooling) and a Cargo crate (consumed by `cargo test` for doctests). The
two views overlap on `src/content/docs/`, which is the source of truth for both.

### Workspace registration

The root `Cargo.toml` adds `"website"` to `members`. The root `package.json`
already uses npm workspaces; we add `"website"` to its `workspaces` array.

### Doctest harness

`website/build.rs` runs at `cargo build` / `cargo test` time and:

1. Walks `website/src/content/docs/` recursively, collecting every `.md` and
   `.mdx` file.
2. For each file, emits a `pub mod __doctest_<sanitized_path> { #![doc = include_str!("…")] }`
   stub into a generated file at `$OUT_DIR/doctests.rs`.
3. Emits `cargo:rerun-if-changed` for the docs root and for every collected
   markdown file (so cargo re-runs the build script when files are added,
   removed, or modified).

`website/src/lib.rs` (or `src/main.rs`) is one line:
`include!(concat!(env!("OUT_DIR"), "/doctests.rs"));`

Generated artifacts live exclusively under `$OUT_DIR` (which cargo places under
`target/`). Nothing generated is committed to the repo.

`website/Cargo.toml` declares a dependency on `neon` via relative path. The
exact feature set is deferred to implementation, but the baseline matches the
existing `cargo neon-test` alias in `.cargo/config.toml`
(`napi-experimental,external-buffers,serde,tokio`) since samples cover
`Json<T>`, async/tokio, and other features that require those flags. The
implementation plan should pin the feature set explicitly and document why.

#### Interaction with `cargo neon-test`

Adding `website` to the root workspace means `cargo neon-test --all` (and the
existing CI matrix) will pick it up automatically and run its doctests on every
supported platform. This is intentional — Neon API changes that break samples
should fail the existing CI matrix, not just the website workflow. The
website-specific workflow (described below) still runs `cargo test -p website`
explicitly, but treats it as a fast pre-flight on a single platform; the
matrix is the authoritative cross-platform check.

#### Sample conventions

- **Every Rust sample is wrapped in `#[neon::export]`** (or another
  `#[neon::export(...)]` flavor — `task`, `async`, `class`, etc.). This means
  rustdoc compiles each sample but does not invoke the function from its
  synthesized `fn main`, giving us compile-checking without needing a Node
  runtime.
- **Hidden lines** (`#`-prefixed) are used freely for `use` statements, helper
  bindings, and other setup that should compile but not appear on the page.
- **The only fence we recognize is plain `` ```rust ``.** No `rust,no_run`,
  `rust,ignore`, etc. If a sample needs to compile but appear different on the
  rendered page, authors do that with hidden lines, not info-string flags. If
  this constraint becomes a problem we revisit it; the simplicity is worth it
  for v1.
- **Pure-Rust runnable samples** (rare) can include an explicit `fn main() { … }`
  inside the fence; rustdoc will then run it.

### Hidden-line remark plugin

`website/strip-hidden-rust-lines.mjs` is a small remark plugin imported by
`astro.config.mjs`. It mirrors rustdoc's hidden-line convention exactly:

- Walks `code` nodes whose `lang === 'rust'`.
- Removes lines whose first non-whitespace character is `#` followed by a
  space, *or* a `#` at end-of-line.
- Replaces `##` at the start of a line with a literal `#` in the rendered
  output.
- Leaves all other code blocks (and rendered Markdown) untouched.

This plugin runs only on rendered output. It does not touch the on-disk markdown
that the doctest harness consumes; rustdoc applies its own (identical) hidden-
line rules when compiling.

No separate test for the plugin in v1.

### rustdoc integration

- `starlight-rustdoc` is added to Astro/Starlight integrations and mounts
  rustdoc HTML at `/api/`.
- `package.json` defines `"prebuild": "cargo doc -p neon --no-deps"` so the
  rustdoc HTML exists before Astro builds. `"build"` is the Astro build.
  Netlify and CI both run `npm run build`, which transitively runs prebuild.
- The site always reflects the in-tree `neon` crate. Historical versions are
  served by docs.rs; we don't replicate that here.
- Local dev (`npm run dev`) does *not* rebuild rustdoc on Rust source changes.
  Contributors run `cargo doc -p neon --no-deps` once locally, or whenever
  they need updated API docs.

### Visual design

- **Doc pages** use Starlight's default template and theme, with the brand
  accent set to Neon's existing green and the existing lightning-bolt logo
  (`doc/neon.png`) used for the header logo and favicon.
- **Landing page** uses Starlight's `splash` template with custom layout. See
  the next section for the structure.
- **Search** uses Starlight's built-in Pagefind. No third-party service.
- **Sidebar** mirrors Diátaxis quadrants in the order listed under
  "Information architecture" above. The Reference group's "API reference" is
  a single external-style link to `/api/neon/`, not an expandable tree.

### Landing page layout

Single page (`src/content/docs/index.mdx` plus components in `src/components/`),
top-to-bottom:

1. **Hero**, two-column.
   - Left: logo + wordmark, tagline **"Write Node addons in Rust"**, sub-line
     **"Safe, fast, parallel."**, two CTAs ("Get started" → first-module
     tutorial; "API reference" → `/api/neon/`).
   - Right: side-by-side JS + Rust code comparison showing the same small
     async function in both languages, using `#[neon::export]` on the Rust
     side. Stacked vertically on narrow viewports.
2. **Three-tile grid.** Headers preserved from the existing site:
   - **Simple tooling.** No build scripts. No finicky system dependencies.
     Just Node and Rust.
   - **Guaranteed safety.** If a Neon module compiles, it is guaranteed by
     the Rust compiler to be memory-safe.
   - **Easy parallelism.** Safely run multiple threads — without data races.
3. **A code block** showing the `#[neon::export(async)]` synchronous-setup-
   then-async pattern. No heading, no caption — the code stands alone.
4. **Footer.** Standard Starlight footer with GitHub, Slack, license links.
   No "who's using Neon" section in v1.

## CI & deployment

### Workflow

A new `.github/workflows/website.yml` runs on:

- `push` to `main`
- `pull_request` against `main`

…with path filters limiting it to changes under `website/**`, `crates/neon/**`,
or `crates/neon-macros/**` (since either crate's API can break samples or
rustdoc).

Jobs:

1. **doctests**: `cargo test -p website`. Verifies every Rust sample compiles
   against the in-tree `neon` crate.
2. **build**: `npm ci && npm run build` inside `website/`. Verifies the site
   builds end-to-end (including `cargo doc`). Uploads the `dist/` artifact
   for inspection if needed.

The `doctests` job becomes a required check on PRs that match the path filters.
This is intentional: if a Neon API change breaks a documented sample, the PR
is blocked until samples are updated.

### Netlify

- `netlify.toml` sets `base = "website"`, `command = "npm run build"`,
  `publish = "website/dist"`.
- Rust toolchain is provisioned via Netlify's `RUST_VERSION` env var (or a
  `rustup` invocation in a build hook if the env var route doesn't pin
  precisely enough).
- Deploy previews per PR are enabled by Netlify's GitHub integration. The PR
  comment with the preview URL is automatic; nothing to wire up beyond
  Netlify project settings.

## Open questions for implementation

These weren't decided during brainstorming but don't block writing the spec.
The plan should resolve them:

- Exact `neon` feature set the doctest crate should enable.
- Whether `starlight-rustdoc` needs config beyond defaults (mount path, sidebar
  integration).
- Pagefind index size with rustdoc included; if it's too large, we may need
  to scope Pagefind to non-rustdoc routes.
- Exact sample for the hero JS/Rust comparison and the section-3 code block.
  These should be drafted during implementation, not pre-frozen here.

## Out of scope (revisit post-launch)

- Blog and release announcements.
- Migration guide(s) on the new site.
- Electron- and Bun-specific how-tos.
- Versioned API docs (in-tree only for v1; docs.rs covers history).
- i18n.
- Community-built modules showcase.
- Custom search beyond Pagefind.
