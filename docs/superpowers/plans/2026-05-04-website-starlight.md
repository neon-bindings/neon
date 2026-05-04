# Neon docs site (Starlight) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace neon-rs.dev with a brand-new Starlight site at `website/` in this monorepo, with Rust-doctest-verified samples, mounted rustdoc, Diátaxis content structure, and Netlify deploy previews.

**Architecture:** `website/` is simultaneously a Cargo workspace member (so `cargo test -p website` doctests every Rust sample in `src/content/docs/**/*.md{,x}`) and an Astro/Starlight project (so `npm run build` produces the static site, with `cargo doc` HTML copied into `public/api/` during `prebuild`). A small remark plugin hides `#`-prefixed lines from rendered Rust fences. CI runs both jobs on a path-filtered workflow; Netlify deploys end-to-end on every PR.

**Tech Stack:** Rust 1.65+, Cargo workspaces, Astro, Starlight, Pagefind (built-in), npm workspaces, Netlify, GitHub Actions.

**Source spec:** `docs/superpowers/specs/2026-05-04-website-starlight-design.md`

---

## Conventions

- Each task is committed at the end. Commits are small and focused.
- All paths in this plan are relative to the repo root (`/Users/kj/git/github/neon-bindings/neon`).
- "Verify" steps run a concrete command and check expected output. Don't skip them.
- Markdown files in this plan deliberately avoid multi-line bullet continuations and reference-style links — the project's auto-formatter has been observed mangling those patterns. If editing markdown, prefer single-line bullets.

---

## Phase 1: Workspace plumbing

The goal of this phase is to land an empty-but-buildable `website/` crate that's a member of both the Cargo and npm workspaces, with no content yet. After this phase, `cargo test -p website` succeeds (with zero tests) and `cargo neon-test` continues to pass.

### Task 1.1: Create the `website/` crate skeleton

**Files:**

- Create: `website/Cargo.toml`
- Create: `website/src/lib.rs`
- Create: `website/build.rs`
- Create: `website/.gitignore`
- Modify: `Cargo.toml` (root) to add `"website"` to `members`

- [ ] **Step 1: Create `website/Cargo.toml`**

```toml
[package]
name = "website"
version = "0.0.0"
edition = "2021"
publish = false
description = "Doctest harness for the Neon docs site (https://neon-rs.dev)."

[lib]
path = "src/lib.rs"

[dependencies]
neon = { path = "../crates/neon", default-features = false, features = [
    "napi-experimental",
    "external-buffers",
    "serde",
    "tokio",
] }

[build-dependencies]
walkdir = "2"
```

The `neon` feature set matches the `cargo neon-test` alias in `.cargo/config.toml` (minus `sys`, which is for docs-rs metadata only): `napi-experimental` for the latest Node-API surface, `external-buffers` for buffer samples, `serde` for `Json<T>` samples, `tokio` (which transitively pulls in `futures` + `tokio/rt-multi-thread`) for async samples.

- [ ] **Step 2: Create `website/src/lib.rs`** with the single line that pulls in the build-script-generated doctests.

```rust
include!(concat!(env!("OUT_DIR"), "/doctests.rs"));
```

- [ ] **Step 3: Create a stub `website/build.rs`** that does nothing yet but emits an empty doctests file so `lib.rs` compiles.

```rust
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let out_path = PathBuf::from(out_dir).join("doctests.rs");
    fs::write(&out_path, "// generated\n").expect("write doctests.rs");
}
```

- [ ] **Step 4: Create `website/.gitignore`** to keep Astro/Node artifacts out of the repo. We add Rust artifacts here too even though the workspace target dir is at the root, in case anyone runs cargo commands from inside `website/`.

```gitignore
# Astro / Node
node_modules/
dist/
.astro/
.netlify/

# Rust
target/

# Editor / OS
.DS_Store
```

- [ ] **Step 5: Add `website` to the root Cargo workspace.** Modify `Cargo.toml` (root) so the `members` array includes `"website"`. The current file is:

```toml
[workspace]
resolver = "2"
members = [
    "crates/*",
    "test/*",
    "bench",
]

[profile.release]
lto = true
```

Change `members` to:

```toml
members = [
    "crates/*",
    "test/*",
    "bench",
    "website",
]
```

- [ ] **Step 6: Verify the crate compiles.**

Run: `cargo build -p website`
Expected: succeeds; produces `target/debug/libwebsite.rlib` (or similar). No warnings about empty `doctests.rs`.

- [ ] **Step 7: Verify `cargo test -p website` passes with zero tests.**

Run: `cargo test -p website`
Expected: `test result: ok. 0 passed; 0 failed; ...`.

- [ ] **Step 8: Verify the workspace-wide test alias still works.**

Run: `cargo neon-test --no-run`
Expected: succeeds. (We use `--no-run` to skip the actual JS-runtime-dependent tests; we just want to confirm the workspace still builds.)

- [ ] **Step 9: Commit.**

```bash
git add website/ Cargo.toml
git commit -m "feat(website): add empty website crate to workspace

Stub crate that will host the doctest harness for the new Starlight
docs site. Cargo workspace member only; npm workspace registration
and build script logic come in later commits."
```

### Task 1.2: Register `website/` as an npm workspace

**Files:**

- Create: `website/package.json` (minimal stub)
- Modify: `package.json` (root) to add `"website"` to `workspaces`

- [ ] **Step 1: Create a minimal `website/package.json`.**

```json
{
  "name": "@neon-rs/website",
  "version": "0.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "build": "echo 'build not implemented yet' && exit 1"
  }
}
```

The stub `build` script exits non-zero so a future task that forgets to replace it will fail loudly rather than silently succeed.

- [ ] **Step 2: Add `website` to the root `package.json` workspaces array.**

The current root `package.json` has:

```json
"workspaces": [
    "pkgs/*",
    "test/*",
    "bench"
]
```

Change to:

```json
"workspaces": [
    "pkgs/*",
    "test/*",
    "bench",
    "website"
]
```

- [ ] **Step 3: Verify `npm install` picks up the new workspace.**

Run: `npm install`
Expected: succeeds. `node_modules/@neon-rs/website` exists as a symlink into `website/`.

Run: `ls -la node_modules/@neon-rs/website`
Expected: symlink pointing to `../../website`.

- [ ] **Step 4: Commit.**

```bash
git add website/package.json package.json package-lock.json
git commit -m "feat(website): register website as npm workspace member"
```

---

## Phase 2: Doctest harness

After this phase, dropping a markdown file with a ` ```rust ` fence into `website/src/content/docs/` automatically becomes a doctest that's compiled by `cargo test -p website`.

### Task 2.1: Implement the markdown-walking build script

**Files:**

- Modify: `website/build.rs`
- Create: `website/src/content/docs/.gitkeep` (so the directory exists at git checkout time)
- Create: `website/src/content/docs/smoke.md` (a tiny smoke-test markdown file with one passing doctest)

- [ ] **Step 1: Replace `website/build.rs` with the real implementation.** This walks `src/content/docs/`, sanitizes each path into a Rust-safe module name, and emits one `pub mod` per file containing `#![doc = include_str!("…")]`.

```rust
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

const DOCS_ROOT: &str = "src/content/docs";

fn main() {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_dir = PathBuf::from(manifest_dir);
    let docs_root = manifest_dir.join(DOCS_ROOT);

    println!("cargo:rerun-if-changed={}", docs_root.display());

    let mut entries = Vec::new();
    if docs_root.is_dir() {
        for entry in WalkDir::new(&docs_root).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = match path.extension().and_then(|e| e.to_str()) {
                Some(e) => e,
                None => continue,
            };
            if ext != "md" && ext != "mdx" {
                continue;
            }
            println!("cargo:rerun-if-changed={}", path.display());
            entries.push(path.to_path_buf());
        }
    }

    entries.sort();

    let mut out = String::from("// generated by build.rs\n");
    for path in &entries {
        let rel = path
            .strip_prefix(&docs_root)
            .expect("path under docs_root");
        let module_name = sanitize_module_name(rel);
        // Use absolute path in include_str! so it works regardless of cwd.
        let include_path = path.to_string_lossy().replace('\\', "/");
        out.push_str(&format!(
            "#[doc = include_str!(\"{}\")]\npub mod {} {{}}\n",
            include_path, module_name
        ));
    }

    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let out_path = PathBuf::from(out_dir).join("doctests.rs");
    fs::write(&out_path, out).expect("write doctests.rs");
}

fn sanitize_module_name(rel: &Path) -> String {
    let mut s = String::from("doc");
    for component in rel.with_extension("").components() {
        if let std::path::Component::Normal(part) = component {
            s.push('_');
            for ch in part.to_string_lossy().chars() {
                if ch.is_ascii_alphanumeric() {
                    s.push(ch.to_ascii_lowercase());
                } else {
                    s.push('_');
                }
            }
        }
    }
    s
}
```

A few notes for the implementer:

- We use `walkdir` (already in `[build-dependencies]`) rather than `std::fs::read_dir` recursion so symlinks and platform quirks are handled correctly.
- `include_str!` resolves relative to the *generated file*, which lives in `OUT_DIR`. That's somewhere weird like `target/debug/build/website-xxx/out/`, so we use absolute paths to avoid any chance of path-resolution ambiguity.
- `sanitize_module_name` is intentionally simple: it lowercases, replaces non-alphanumeric with `_`, and prefixes `doc` (no trailing underscore) so the loop's leading separator yields a clean `doc_<first>` snake_case module without a double-underscore prefix.

- [ ] **Step 2: Create `website/src/content/docs/.gitkeep`** so the directory exists in git even before any markdown is added. Empty file.

- [ ] **Step 3: Write a smoke-test markdown file** to verify the harness end-to-end. Create `website/src/content/docs/smoke.md`:

````markdown
# Smoke test

This file exists to verify the doctest harness works. It will be deleted in a later task once we have real content.

```rust
# use neon::prelude::*;
#[neon::export]
fn hello() -> &'static str {
    "Hello from a doctest"
}
```
````

The `# use neon::prelude::*;` line is a hidden line — rustdoc strips it from the rendered output but compiles it. The remark plugin we add in Phase 4 will do the same for the rendered website.

- [ ] **Step 4: Verify the build script generates the expected output.**

Run: `cargo build -p website`

Run: `cat target/debug/build/website-*/out/doctests.rs`
Expected: contains a line like `#[doc = include_str!("/Users/.../website/src/content/docs/smoke.md")]\npub mod doc_smoke {}`.

- [ ] **Step 5: Verify the doctest actually runs and passes.**

Run: `cargo test -p website --doc`
Expected: `test result: ok. 1 passed; 0 failed; ...`. The `1` is the smoke test.

- [ ] **Step 6: Verify the build script reruns when the markdown changes.**

Run: `touch website/src/content/docs/smoke.md && cargo build -p website -v 2>&1 | grep -E "Compiling website|Fresh website" | head -2`

Expected: shows `Compiling website` (not `Fresh website`), proving the build script ran again. (If you see `Fresh`, the rerun-if-changed logic is broken.)

- [ ] **Step 7: Verify deliberate breakage fails the doctest.** This is a cheap correctness check that the harness actually catches errors, not just compiles trivially.

Edit `website/src/content/docs/smoke.md` and change `"Hello from a doctest"` to a syntax error like `"Hello`.

Run: `cargo test -p website --doc`
Expected: FAIL with a Rust compile error.

Restore the file: change `"Hello` back to `"Hello from a doctest"`.

Run: `cargo test -p website --doc`
Expected: PASS again.

- [ ] **Step 8: Commit.**

```bash
git add website/build.rs website/src/content/docs/
git commit -m "feat(website): add markdown-walking doctest harness

build.rs walks src/content/docs/ at compile time and emits one
include_str!-d module per .md/.mdx file into OUT_DIR. cargo test -p
website now compiles every Rust fenced code block in the docs tree
as a rustdoc doctest. Includes a smoke-test markdown file to verify
the harness end-to-end; that file will be removed once real content
lands."
```

---

## Phase 3: Astro / Starlight scaffold

After this phase, `npm run dev` inside `website/` starts a working Starlight dev server with placeholder content; `npm run build` produces a static site in `website/dist/`. No custom landing page yet, no rustdoc, no remark plugin.

### Task 3.1: Install Starlight and its dependencies

**Files:**

- Modify: `website/package.json` to add real dependencies and scripts
- Modify: `package-lock.json` (root) — automatically updated by npm install

- [ ] **Step 1: From the repo root, install Starlight in the website workspace.**

Run:

```bash
npm install --workspace=@neon-rs/website astro @astrojs/starlight
npm install --workspace=@neon-rs/website --save-dev @astrojs/check typescript
```

Expected: succeeds. `website/package.json` now has dependencies. `node_modules/` at the root contains Astro and Starlight.

- [ ] **Step 2: Replace the stubbed scripts in `website/package.json`** so it has a real Astro setup. The exact `dependencies` and `devDependencies` blocks will be filled in by npm; your job is to set the `scripts`, `type`, and `name` correctly. After this step the file should look like:

```json
{
  "name": "@neon-rs/website",
  "version": "0.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "astro dev",
    "start": "astro dev",
    "build": "astro build",
    "preview": "astro preview",
    "astro": "astro",
    "check": "astro check"
  },
  "dependencies": {
    "@astrojs/starlight": "...",
    "astro": "..."
  },
  "devDependencies": {
    "@astrojs/check": "...",
    "typescript": "..."
  }
}
```

(The `...` versions are whatever `npm install` pinned in step 1.)

- [ ] **Step 3: Verify the dev script even attempts to run.** It will fail because there's no `astro.config.mjs` yet, but it should fail with an Astro-specific error rather than "command not found".

Run: `npm run dev --workspace=@neon-rs/website -- --help`
Expected: prints Astro CLI help text. Confirms astro binary is reachable.

- [ ] **Step 4: Commit.**

```bash
git add website/package.json package.json package-lock.json
git commit -m "feat(website): install Astro + Starlight"
```

### Task 3.2: Create the minimal Starlight config

**Files:**

- Create: `website/astro.config.mjs`
- Create: `website/tsconfig.json`
- Create: `website/src/content.config.ts`
- Create: `website/src/content/docs/index.mdx` (placeholder, replaces `smoke.md`)
- Delete: `website/src/content/docs/smoke.md`

- [ ] **Step 1: Create `website/astro.config.mjs`** with the minimum config needed for Starlight to start.

```javascript
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

export default defineConfig({
  site: "https://neon-rs.dev",
  integrations: [
    starlight({
      title: "Neon",
      description: "Write Node addons in Rust.",
      logo: {
        src: "./public/logo.png",
        replacesTitle: false,
      },
      social: [
        { icon: "github", label: "GitHub", href: "https://github.com/neon-bindings/neon" },
        { icon: "slack", label: "Slack", href: "https://rust-bindings.slack.com" },
      ],
      sidebar: [
        { label: "Getting started", autogenerate: { directory: "getting-started" } },
        { label: "Tutorials", autogenerate: { directory: "tutorials" } },
        { label: "How-to guides", autogenerate: { directory: "how-to" } },
        {
          label: "Reference",
          items: [
            { label: "API reference", link: "/api/neon/", attrs: { target: "_blank" } },
            { label: "Supported platforms", link: "/reference/supported-platforms/" },
            { label: "CLI reference", link: "/reference/cli/" },
          ],
        },
        { label: "Explanation", autogenerate: { directory: "explanation" } },
      ],
    }),
  ],
});
```

The `Reference > API reference` link uses `attrs: { target: "_blank" }` because rustdoc has its own UI; opening it in a new tab keeps the Starlight nav available.

- [ ] **Step 2: Create `website/tsconfig.json`** — Starlight's recommended config.

```json
{
  "extends": "astro/tsconfigs/strict",
  "include": [".astro/types.d.ts", "**/*"],
  "exclude": ["dist", "target", "node_modules"]
}
```

- [ ] **Step 3: Copy the existing Neon logo into the website's public assets.**

```bash
mkdir -p website/public
cp doc/neon.png website/public/logo.png
cp doc/neon.png website/public/favicon.png
```

- [ ] **Step 4: Create `website/src/content.config.ts`** — Starlight requires this file so it knows about the docs collection.

```typescript
import { defineCollection } from "astro:content";
import { docsLoader } from "@astrojs/starlight/loaders";
import { docsSchema } from "@astrojs/starlight/schema";

export const collections = {
  docs: defineCollection({ loader: docsLoader(), schema: docsSchema() }),
};
```

- [ ] **Step 5: Replace the smoke-test markdown with a placeholder index.** Delete `website/src/content/docs/smoke.md`. Create `website/src/content/docs/index.mdx`:

```mdx
---
title: Neon
description: Write Node addons in Rust.
template: splash
---

# Neon

Write Node addons in Rust. Safe, fast, parallel.

This is a placeholder landing page. The real one is built in a later task.
```

- [ ] **Step 6: Create directory placeholders** so Starlight's `autogenerate` doesn't fail. Create empty index files in each Diátaxis directory:

```bash
mkdir -p website/src/content/docs/getting-started \
         website/src/content/docs/tutorials \
         website/src/content/docs/how-to \
         website/src/content/docs/reference \
         website/src/content/docs/explanation
```

Create `website/src/content/docs/getting-started/index.md`:

```markdown
---
title: Getting started
---

Placeholder. Real content lands in Phase 5.
```

Repeat for `tutorials/index.md`, `how-to/index.md`, `reference/index.md`, `explanation/index.md`, swapping the title each time.

- [ ] **Step 7: Verify the dev server starts.**

Run: `npm run dev --workspace=@neon-rs/website`
Expected: prints `Local: http://localhost:4321/` (or similar). No errors. (Stop the server with Ctrl-C.)

- [ ] **Step 8: Verify the production build succeeds.**

Run: `npm run build --workspace=@neon-rs/website`
Expected: succeeds; produces `website/dist/index.html` and per-section `index.html` files.

Run: `ls website/dist/`
Expected: `index.html`, plus `getting-started/`, `tutorials/`, `how-to/`, `reference/`, `explanation/`, plus Astro asset folders.

- [ ] **Step 9: Verify the doctest harness still works** — the Phase-2 smoke test was deleted, so we expect zero passing doctests, but the build itself must still succeed.

Run: `cargo test -p website --doc`
Expected: `test result: ok. 0 passed; 0 failed; ...`.

- [ ] **Step 10: Commit.**

```bash
git add website/ package-lock.json
git commit -m "feat(website): scaffold Starlight site with Diátaxis sidebar

Adds the minimum Starlight configuration with placeholder pages in
each Diátaxis directory, the Neon lightning-bolt logo as the header
mark, and a sidebar that mirrors the spec's information architecture."
```

---

## Phase 4: Hidden-line remark plugin

After this phase, the rendered HTML hides `# `-prefixed lines from Rust fences (matching rustdoc's behavior), so authors can include `use` statements and other setup without cluttering the page.

### Task 4.1: Implement the remark plugin

**Files:**

- Create: `website/strip-hidden-rust-lines.mjs`
- Modify: `website/astro.config.mjs` to register the plugin

- [ ] **Step 1: Create `website/strip-hidden-rust-lines.mjs`.**

```javascript
/**
 * Remark plugin: hide rustdoc-style hidden lines from rendered Rust code blocks.
 *
 * Mirrors rustdoc's convention exactly:
 *   - Lines whose first non-whitespace character is `#` followed by a space
 *     (or `#` at end-of-line) are removed from the rendered output.
 *   - Lines starting with `##` are unescaped to a literal `#`.
 *   - Other lines pass through unchanged.
 *
 * Applies only to fences whose language is exactly `rust`.
 */
export function remarkStripHiddenRustLines() {
  return (tree) => {
    visit(tree, "code", (node) => {
      if (node.lang !== "rust") return;
      const lines = node.value.split("\n");
      const out = [];
      for (const line of lines) {
        const trimmed = line.trimStart();
        if (trimmed === "#" || trimmed.startsWith("# ")) {
          continue;
        }
        if (trimmed.startsWith("##")) {
          // Unescape ## → # at the same leading-whitespace position.
          const leading = line.slice(0, line.length - trimmed.length);
          out.push(leading + trimmed.slice(1));
          continue;
        }
        out.push(line);
      }
      node.value = out.join("\n");
    });
  };
}

// Inline visit() to avoid adding a dependency for ~10 lines.
function visit(node, type, fn) {
  if (node.type === type) fn(node);
  if (Array.isArray(node.children)) {
    for (const child of node.children) visit(child, type, fn);
  }
}
```

The plugin is dependency-free on purpose. `unist-util-visit` is the conventional way to walk an mdast tree, but for a 10-line traversal it's overkill.

- [ ] **Step 2: Register the plugin in `website/astro.config.mjs`.** Add the import at the top and the `markdown.remarkPlugins` field to `defineConfig`.

```javascript
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import { remarkStripHiddenRustLines } from "./strip-hidden-rust-lines.mjs";

export default defineConfig({
  site: "https://neon-rs.dev",
  markdown: {
    remarkPlugins: [remarkStripHiddenRustLines],
  },
  integrations: [
    starlight({
      // ... existing config unchanged ...
    }),
  ],
});
```

- [ ] **Step 3: Add a hidden-line example to the placeholder index** so we can verify the plugin works visually. Edit `website/src/content/docs/index.mdx`:

````mdx
---
title: Neon
description: Write Node addons in Rust.
template: splash
---

# Neon

Write Node addons in Rust. Safe, fast, parallel.

```rust
# use neon::prelude::*;
#[neon::export]
fn hello() -> &'static str {
    "Hello from Neon"
}
```
````

- [ ] **Step 4: Verify the rendered page hides the `# use` line.**

Run: `npm run build --workspace=@neon-rs/website`
Run: `grep -c "use neon::prelude" website/dist/index.html`
Expected: `0` (the hidden line is stripped).

Run: `grep -c "Hello from Neon" website/dist/index.html`
Expected: `1` (or more — the visible content is preserved).

- [ ] **Step 5: Verify the doctest still includes the hidden line.**

Run: `cargo test -p website --doc -- --nocapture`
Expected: `test result: ok. 1 passed; ...`. The doctest works because rustdoc strips the `#` lines on its own; the file on disk has the `# use` line, which rustdoc compiles in but hides from rendered docs.

- [ ] **Step 6: Add a `##` escape-test fence** to verify the unescape branch. Add another fence to `index.mdx`:

````mdx
```rust
# use neon::prelude::*;
## not actually hidden
#[neon::export]
fn one() -> i32 { 1 }
```
````

- [ ] **Step 7: Verify the `##` line renders as a literal `#`.**

Run: `npm run build --workspace=@neon-rs/website`
Run: `grep "# not actually hidden" website/dist/index.html`
Expected: one match showing `# not actually hidden` (single `#`, not `##`).

- [ ] **Step 8: Verify the doctest passes** — note that `## not actually hidden` is a Rust comment-like line that, after rustdoc's own unescape, becomes `# not actually hidden`, which is invalid Rust. Wait — that's a bug. We need to reconsider the test case.

Actually rustdoc's behavior is: it strips lines starting with `# `, and unescapes `##` → `#` for display, but for *compilation* it removes the leading `#` entirely. So `## not actually hidden` becomes `# not actually hidden` which is also invalid Rust.

The correct demonstration is to use `##` only in cases where the resulting content is valid Rust. For example, `## #[allow(dead_code)]` becomes `# #[allow(dead_code)]` which is — also not valid Rust.

This means the `##` escape is rarely useful in practice and is mostly a curiosity from rustdoc's design. For our smoke test, just delete this second fence; we don't need to demonstrate `##` for v1.

Edit `website/src/content/docs/index.mdx` and remove the second fence we added in Step 6.

- [ ] **Step 9: Re-verify everything passes.**

Run: `npm run build --workspace=@neon-rs/website`
Expected: succeeds.

Run: `cargo test -p website --doc`
Expected: `test result: ok. 1 passed; ...`.

- [ ] **Step 10: Commit.**

```bash
git add website/strip-hidden-rust-lines.mjs website/astro.config.mjs website/src/content/docs/index.mdx
git commit -m "feat(website): add remark plugin to hide #-prefixed Rust lines

Mirrors rustdoc's hidden-line convention so authors can include use
statements and helper bindings in samples without cluttering the
rendered page. The plugin is dependency-free; mdast traversal is
inlined rather than pulling in unist-util-visit."
```

---

## Phase 5: Content scaffolding

After this phase, every page listed in the spec exists with frontmatter, a "skeleton" Starlight aside, and a one-paragraph topic summary. Real prose lands in subsequent tasks (which can be assigned to other contributors). The site has a complete sidebar and no broken links.

This phase intentionally does not write the long-form content for each page — that's editorial work that needs sustained attention from a domain expert, not a checklist task. What this phase produces is the *skeleton*, so contributors filling in pages have a clear assignment and the sidebar/links are testable.

### Task 5.1: Create page stubs for every spec-listed page

**Files:**

- Create: `website/src/content/docs/getting-started/install.md`
- Create: `website/src/content/docs/getting-started/quickstart.md`
- Create: `website/src/content/docs/tutorials/first-module.md`
- Create: `website/src/content/docs/tutorials/concurrency-libuv.md`
- Create: `website/src/content/docs/tutorials/async-tokio.md`
- Create: `website/src/content/docs/how-to/common-types.md`
- Create: `website/src/content/docs/how-to/serde-json.md`
- Create: `website/src/content/docs/how-to/classes.md`
- Create: `website/src/content/docs/how-to/blocking-libuv.md`
- Create: `website/src/content/docs/how-to/async-fn.md`
- Create: `website/src/content/docs/how-to/sync-setup-async.md`
- Create: `website/src/content/docs/how-to/main-thread-after-async.md`
- Create: `website/src/content/docs/how-to/abort-controller.md`
- Create: `website/src/content/docs/how-to/streaming.md`
- Create: `website/src/content/docs/how-to/errors.md`
- Create: `website/src/content/docs/how-to/rename-exports.md`
- Create: `website/src/content/docs/how-to/cx-access.md`
- Create: `website/src/content/docs/how-to/this-methods.md`
- Create: `website/src/content/docs/how-to/prebuilt-binaries.md`
- Create: `website/src/content/docs/reference/supported-platforms.md`
- Create: `website/src/content/docs/reference/cli.md`
- Create: `website/src/content/docs/explanation/what-is-neon.md`
- Create: `website/src/content/docs/explanation/threading-lifecycle.md`
- Create: `website/src/content/docs/explanation/export-internals.md`
- Create: `website/src/content/docs/explanation/error-handling.md`
- Create: `website/src/content/docs/explanation/type-hierarchy.md`
- Create: `website/src/content/docs/explanation/lifetimes.md`
- Create: `website/src/content/docs/explanation/when-to-use.md`
- Create: `website/src/content/docs/changelog.md`
- Create: `website/src/content/docs/contributing.md`
- Delete: each of the placeholder `index.md` files added in Task 3.2 (they're replaced by the real subsection content)

- [ ] **Step 1: Write a single-page template** that every stub will use. Each file has frontmatter with `title` and `description`, a heading, and a "Status: skeleton — content forthcoming" callout. Example for `tutorials/first-module.md`:

```markdown
---
title: Your first Neon module
description: Install the toolchain, scaffold a project with create-neon, export a function with #[neon::export], and call it from JavaScript.
---

:::caution[Status: skeleton]
This page is a placeholder. Content forthcoming.
:::

This tutorial walks through building your first Neon module from scratch. You will install the Rust toolchain, scaffold a project with `npm init neon@latest`, export a function with `#[neon::export]`, and call it from JavaScript.
```

The `:::caution[Status: skeleton]` block is a Starlight aside; it makes draft pages visually distinct so reviewers don't think they're real content.

- [ ] **Step 2: For every file in the file list above, create it with the template format from Step 1.** Tailor the `title`, `description`, and one-paragraph summary to the page's specific topic. Specifically:

- `getting-started/install.md` — toolchain prerequisites and `npm init neon@latest`.
- `getting-started/quickstart.md` — produce a runnable hello-world fast.
- `tutorials/first-module.md` — see Step 1.
- `tutorials/concurrency-libuv.md` — convert a CPU-bound function to `#[neon::export(task)]`.
- `tutorials/async-tokio.md` — register a global executor and write `async fn`.
- `how-to/common-types.md` — pass numbers, strings, arrays, objects, buffers.
- `how-to/serde-json.md` — `Json<T>` and the `json` shorthand attribute.
- `how-to/classes.md` — `#[neon::class]`.
- `how-to/blocking-libuv.md` — `#[neon::export(task)]`.
- `how-to/async-fn.md` — `async fn` exports and global executors.
- `how-to/sync-setup-async.md` — `#[neon::export(async)]` returning `impl Future`.
- `how-to/main-thread-after-async.md` — `extract::with`.
- `how-to/abort-controller.md` — adapting `AbortController` to a tokio `CancellationToken` (mirrors <https://github.com/neon-bindings/examples/pull/104>).
- `how-to/streaming.md` — stream data between Rust and JS.
- `how-to/errors.md` — throw and catch JS errors from Rust, including `extract::Error` + `?`.
- `how-to/rename-exports.md` — customize the JS-facing name.
- `how-to/cx-access.md` — access `Cx` / `FunctionContext` from an exported function.
- `how-to/this-methods.md` — implement `this`-style methods.
- `how-to/prebuilt-binaries.md` — publish prebuilts to npm.
- `reference/supported-platforms.md` — OS matrix, Node-API versions, MSRV.
- `reference/cli.md` — `create-neon` and `cargo-cp-artifact`.
- `explanation/what-is-neon.md` — Neon and its relationship to Node-API.
- `explanation/threading-lifecycle.md` — `Channel`, `Root`, `Deferred`.
- `explanation/export-internals.md` — how `#[neon::export]` works.
- `explanation/error-handling.md` — Rust `Result` ↔ JS exceptions.
- `explanation/type-hierarchy.md` — the Neon type hierarchy. The spec calls for a Mermaid diagram on this page. Mermaid is not built into Starlight; when the real content lands, the author should either install `astro-mermaid` from <https://github.com/joesaby/astro-mermaid> (listed in the Starlight community plugins registry) or render the diagram as a static SVG. Skeleton task only needs frontmatter + summary.
- `explanation/lifetimes.md` — handle lifetimes, `'cx`, why they exist.
- `explanation/when-to-use.md` — when Neon is and isn't worth it.
- `changelog.md` — for v1, this stub should link to <https://github.com/neon-bindings/neon/blob/main/RELEASES.md>. A future task may inline RELEASES.md via Starlight's `import` of MDX, but that's out of scope for the skeleton.
- `contributing.md` — must include the Slack invite link from the project README: <https://join.slack.com/t/rust-bindings/shared_invite/zt-1pl5s83xe-ZvXyrzL8vuUmijU~7yiEcg>. Also link to <https://github.com/neon-bindings/neon/blob/main/CONTRIBUTING.md> for the in-repo contributor guide.

- [ ] **Step 3: Delete the placeholder index files added in Task 3.2** — they were stand-ins for the section landing pages and are no longer needed now that real pages exist.

```bash
rm website/src/content/docs/getting-started/index.md
rm website/src/content/docs/tutorials/index.md
rm website/src/content/docs/how-to/index.md
rm website/src/content/docs/reference/index.md
rm website/src/content/docs/explanation/index.md
```

- [ ] **Step 4: Update `astro.config.mjs`'s sidebar** so every page is discoverable. The current `autogenerate: { directory: ... }` config will pick up everything alphabetically, which is fine for v1 but non-ideal for tutorials (which want explicit ordering). Add explicit ordering for the tutorials directory:

```javascript
sidebar: [
  { label: "Getting started", autogenerate: { directory: "getting-started" } },
  {
    label: "Tutorials",
    items: [
      { label: "Your first Neon module", link: "/tutorials/first-module/" },
      { label: "Concurrency with the libuv pool", link: "/tutorials/concurrency-libuv/" },
      { label: "Async functions with tokio", link: "/tutorials/async-tokio/" },
    ],
  },
  { label: "How-to guides", autogenerate: { directory: "how-to" } },
  {
    label: "Reference",
    items: [
      { label: "API reference", link: "/api/neon/", attrs: { target: "_blank" } },
      { label: "Supported platforms", link: "/reference/supported-platforms/" },
      { label: "CLI reference", link: "/reference/cli/" },
    ],
  },
  { label: "Explanation", autogenerate: { directory: "explanation" } },
  { label: "Changelog", link: "/changelog/" },
  { label: "Contributing", link: "/contributing/" },
],
```

- [ ] **Step 5: Verify the build still works and produces every page.**

Run: `npm run build --workspace=@neon-rs/website`
Expected: succeeds.

Run: `find website/dist -name index.html | wc -l`
Expected: ≥ 30 (one per stub plus the home page plus Astro generated pages).

- [ ] **Step 6: Verify there are no broken sidebar links.**

Run: `npm run build --workspace=@neon-rs/website 2>&1 | grep -i "warning\|broken"`
Expected: no output (or only Astro framework warnings unrelated to our links).

- [ ] **Step 7: Verify the doctests still pass** (none of the stub pages have Rust fences yet, so we expect 0 doctests).

Run: `cargo test -p website --doc`
Expected: `test result: ok. 0 passed; 0 failed; ...`.

- [ ] **Step 8: Commit.**

```bash
git add website/src/content/docs/ website/astro.config.mjs
git commit -m "feat(website): scaffold all spec-listed pages

Every page in the Diátaxis content map now exists as a stub with
frontmatter, a Starlight 'skeleton' aside, and a one-paragraph
description. Sidebar order is explicit for tutorials (which need
linear progression) and autogenerated for how-to/explanation.
Real prose lands in subsequent commits."
```

---

## Phase 6: Custom landing page

After this phase, `/` is a custom splash page with the hero, three-tile grid, and bare async-setup code sample described in the spec. The placeholder `index.mdx` from Phase 3 is replaced.

### Task 6.1: Build the landing page

**Files:**

- Create: `website/src/components/Hero.astro`
- Create: `website/src/components/CodeCompare.astro`
- Create: `website/src/components/FeatureTiles.astro`
- Modify: `website/src/content/docs/index.mdx`

- [ ] **Step 1: Create `website/src/components/CodeCompare.astro`** — the side-by-side JS+Rust panel for the hero. Use Starlight's `Code` component for syntax highlighting.

```astro
---
import { Code } from "@astrojs/starlight/components";

const js = `// JavaScript
const URL = "https://api.example.com/echo";

export async function echo(text) {
  const res = await fetch(URL, {
    method: "POST",
    body: text,
  });
  return await res.text();
}
`;

const rust = `// Rust + Neon
const URL: &str = "https://api.example.com/echo";

#[neon::export]
async fn echo(text: String) -> Result<String, Error> {
    let res = reqwest::Client::new().post(URL).body(text).send().await?;
    Ok(res.text().await?)
}
`;
---

<div class="code-compare">
  <div class="panel">
    <Code code={js} lang="js" />
  </div>
  <div class="panel">
    <Code code={rust} lang="rust" />
  </div>
</div>

<style>
  .code-compare {
    display: grid;
    gap: 1rem;
    grid-template-columns: 1fr;
  }
  @media (min-width: 640px) {
    .code-compare {
      grid-template-columns: 1fr 1fr;
    }
  }
  .panel {
    min-width: 0;
  }
</style>
```

The exact JS and Rust content is a placeholder; the spec calls out this sample as a deferred decision. Pick something that's recognizably async on both sides and uses `#[neon::export]` with a real-world flavor. The `reqwest` example mirrors the existing site and is likely fine.

- [ ] **Step 2: Create `website/src/components/Hero.astro`.**

```astro
---
import CodeCompare from "./CodeCompare.astro";
---

<section class="hero">
  <div class="hero-text">
    <img src="/logo.png" alt="Neon" class="logo" />
    <h1>Neon</h1>
    <p class="tagline">Write Node addons in Rust.</p>
    <p class="subline">Safe, fast, parallel.</p>
    <div class="ctas">
      <a class="cta primary" href="/tutorials/first-module/">Get started</a>
      <a class="cta secondary" href="/api/neon/">API reference</a>
    </div>
  </div>
  <div class="hero-code">
    <CodeCompare />
  </div>
</section>

<style>
  .hero {
    display: grid;
    gap: 2rem;
    align-items: center;
    padding: 2rem 0;
    grid-template-columns: 1fr;
  }
  @media (min-width: 960px) {
    .hero {
      grid-template-columns: 1fr 1.2fr;
    }
  }
  .logo {
    width: 4rem;
    height: auto;
  }
  h1 {
    font-size: clamp(2rem, 6vw, 3.5rem);
    margin: 0.5rem 0;
  }
  .tagline {
    font-size: clamp(1.25rem, 3vw, 1.75rem);
    margin: 0.25rem 0;
  }
  .subline {
    font-size: 1rem;
    color: var(--sl-color-gray-3);
    margin: 0 0 1.5rem 0;
  }
  .ctas {
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
  }
  .cta {
    display: inline-block;
    padding: 0.6rem 1.1rem;
    border-radius: 0.4rem;
    text-decoration: none;
    font-weight: 600;
  }
  .cta.primary {
    background: var(--sl-color-accent);
    color: var(--sl-color-accent-high);
  }
  .cta.secondary {
    background: transparent;
    color: var(--sl-color-text);
    border: 1px solid var(--sl-color-gray-5);
  }
</style>
```

- [ ] **Step 3: Create `website/src/components/FeatureTiles.astro`.**

```astro
<section class="tiles">
  <div class="tile">
    <h3>Simple tooling</h3>
    <p>No build scripts. No finicky system dependencies. Just Node and Rust.</p>
  </div>
  <div class="tile">
    <h3>Guaranteed safety</h3>
    <p>If a Neon module compiles, it is guaranteed by the Rust compiler to be memory-safe.</p>
  </div>
  <div class="tile">
    <h3>Easy parallelism</h3>
    <p>Safely run multiple threads — without data races.</p>
  </div>
</section>

<style>
  .tiles {
    display: grid;
    gap: 1.25rem;
    grid-template-columns: 1fr;
    padding: 2rem 0;
  }
  @media (min-width: 720px) {
    .tiles {
      grid-template-columns: repeat(3, 1fr);
    }
  }
  .tile h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.1rem;
  }
  .tile p {
    margin: 0;
    color: var(--sl-color-gray-2);
    line-height: 1.5;
  }
</style>
```

- [ ] **Step 4: Replace `website/src/content/docs/index.mdx`** with the real landing page. The page imports the components and includes a final code block (the section-3 "bare code" sample).

````mdx
---
title: Neon
description: Write Node addons in Rust.
template: splash
hero:
  tagline: ""
---

import Hero from "../../components/Hero.astro";
import FeatureTiles from "../../components/FeatureTiles.astro";

<Hero />

<FeatureTiles />

```rust
# use neon::prelude::*;
# use neon::types::extract::{with, Error, TryIntoJs};
# async fn fetch_user(_id: u64) -> Result<String, Error> { Ok(String::new()) }
#[neon::export]
async fn load_user(id: f64) -> impl for<'cx> TryIntoJs<'cx> {
    println!("Hello from the JavaScript main thread!");
    let user = fetch_user(id as u64).await;
    with(move |cx| user.try_into_js(cx))
}
```
````

The hidden lines provide the imports the doctest needs without cluttering the rendered page. The visible content is the `#[neon::export(async)]` synchronous-setup-then-async pattern from the spec — short, self-contained, and shows off something Neon-unique.

- [ ] **Step 5: Verify the build succeeds.**

Run: `npm run build --workspace=@neon-rs/website`
Expected: succeeds.

- [ ] **Step 6: Verify the home page renders the new components.**

Run: `grep -c "Write Node addons in Rust" website/dist/index.html`
Expected: at least 1.

Run: `grep -c "Simple tooling" website/dist/index.html`
Expected: 1.

- [ ] **Step 7: Verify the bare code sample compiles as a doctest.**

Run: `cargo test -p website --doc`
Expected: `test result: ok. 1 passed; ...`. (The exact number depends on whether other pages have Rust fences yet; it should be ≥ 1.)

If the doctest fails because of an API mismatch with `crates/neon` (the `extract::with` import path or signature may have moved between versions), fix the hidden-line imports in `index.mdx` to match what `crates/neon` actually exports. The point is that the sample compiles — the exact import path is implementation detail.

- [ ] **Step 8: Commit.**

```bash
git add website/src/components/ website/src/content/docs/index.mdx
git commit -m "feat(website): build custom splash landing page

Hero with the JS/Rust code comparison, three-tile feature grid with
the headers the maintainer wanted preserved, and a bare code block
demonstrating the sync-setup-then-async pattern. Doctest-checked
end-to-end."
```

---

## Phase 7: Rustdoc integration

After this phase, `npm run build` produces `dist/api/neon/index.html` (the rustdoc), and links from the Starlight sidebar to `/api/neon/` work both locally and on Netlify.

### Task 7.1: Wire `cargo doc` into the Astro build

**Files:**

- Modify: `website/package.json` to add the `prebuild` script
- Create: `website/scripts/copy-rustdoc.mjs` — small Node script that copies `target/doc/` into `public/api/`
- Modify: `website/.gitignore` to ignore `public/api/` (it's generated)

- [ ] **Step 1: Create `website/scripts/copy-rustdoc.mjs`.** We use a tiny Node script rather than shelling out to `cp -r` so the build works identically on Linux, macOS, and Windows (Netlify is Linux, but contributors are everywhere).

```javascript
import { cp, rm } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..", "..");
const src = resolve(repoRoot, "target", "doc");
const dst = resolve(here, "..", "public", "api");

console.log(`Copying rustdoc HTML from ${src} to ${dst}`);
await rm(dst, { recursive: true, force: true });
await cp(src, dst, { recursive: true });
console.log("Done.");
```

- [ ] **Step 2: Add the `prebuild` script to `website/package.json`.** It runs `cargo doc` from the repo root (so the output lands in the workspace `target/`) and then runs the copy script.

```json
"scripts": {
  "dev": "astro dev",
  "start": "astro dev",
  "prebuild": "cargo doc -p neon --no-deps --manifest-path ../Cargo.toml && node scripts/copy-rustdoc.mjs",
  "build": "astro build",
  "preview": "astro preview",
  "astro": "astro",
  "check": "astro check"
}
```

The `--manifest-path ../Cargo.toml` makes `cargo doc` use the workspace manifest even when run from `website/`. The `--no-deps` keeps the output small (rustdoc for `neon`'s direct deps would balloon the dist size and isn't useful to readers).

- [ ] **Step 3: Add the generated rustdoc to `.gitignore`.** Append to `website/.gitignore`:

```gitignore

# Generated rustdoc HTML (copied from target/doc by prebuild)
public/api/
```

- [ ] **Step 4: Verify the prebuild script runs end-to-end.**

Run: `npm run build --workspace=@neon-rs/website`
Expected: prints "Copying rustdoc HTML…" line, then completes the Astro build.

Run: `ls website/dist/api/neon/`
Expected: contains `index.html` and the rustdoc-generated subdirectories.

Run: `head -1 website/dist/api/neon/index.html`
Expected: starts with `<!DOCTYPE html>`.

- [ ] **Step 5: Verify the sidebar link resolves.**

Run: `npm run preview --workspace=@neon-rs/website` (start a local preview server)

In a separate terminal: `curl -sI http://localhost:4321/api/neon/ | head -1`
Expected: `HTTP/1.1 200 OK`.

Stop the preview server.

- [ ] **Step 6: Verify the doctest workflow is unchanged.**

Run: `cargo test -p website --doc`
Expected: passes with the same count as before Phase 7.

- [ ] **Step 7: Verify Pagefind did not index `/api/`.** Pagefind only indexes Astro-rendered pages (those that go through Starlight's `data-pagefind-body` markup); pure static assets in `public/` are skipped. We verify by inspecting the generated index.

Run: `find website/dist/pagefind -name '*.pf_*' | head -5`
Expected: at least some Pagefind index files exist.

Run: `grep -rl "neon::types" website/dist/pagefind/ 2>/dev/null | wc -l`
Expected: `0`. (If non-zero, Pagefind picked up rustdoc HTML and the index will be huge. In that case, scope Pagefind by adding an `data-pagefind-ignore` attribute on the API container or excluding the directory in `astro.config.mjs`'s Pagefind options. Defer the fix; just flag it.)

- [ ] **Step 8: Commit.**

```bash
git add website/package.json website/scripts/ website/.gitignore
git commit -m "feat(website): mount cargo doc HTML at /api/

prebuild runs cargo doc -p neon and copies the result into public/api/.
We use a small Node script for the copy step so the build works
identically on macOS, Linux, and Windows. The generated HTML is
gitignored; CI and Netlify always rebuild it from the in-tree source."
```

---

## Phase 8: CI and Netlify

After this phase, every PR that touches `website/`, `crates/neon/`, or `crates/neon-macros/` runs the doctest job and the build job in GitHub Actions, and Netlify produces a deploy preview URL.

### Task 8.1: Add the website CI workflow

**Files:**

- Create: `.github/workflows/website.yml`

- [ ] **Step 1: Create `.github/workflows/website.yml`.**

```yaml
name: Website

on:
  push:
    branches:
      - main
    paths:
      - "website/**"
      - "crates/neon/**"
      - "crates/neon-macros/**"
      - ".github/workflows/website.yml"
  pull_request:
    branches:
      - main
    paths:
      - "website/**"
      - "crates/neon/**"
      - "crates/neon-macros/**"
      - ".github/workflows/website.yml"

jobs:
  doctests:
    name: Doctests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Use Rust stable
        uses: dtolnay/rust-toolchain@stable

      - name: Rust cache
        uses: Swatinem/rust-cache@v2

      - name: Run website doctests
        run: cargo test -p website --doc

  build:
    name: Build site
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Use Rust stable
        uses: dtolnay/rust-toolchain@stable

      - name: Rust cache
        uses: Swatinem/rust-cache@v2

      - name: Use Node 20
        uses: actions/setup-node@v4
        with:
          node-version: 20.x
          cache: npm

      - name: Install dependencies
        run: npm ci --prefer-offline --no-audit --no-fund

      - name: Build website
        run: npm run build --workspace=@neon-rs/website

      - name: Upload built site
        uses: actions/upload-artifact@v4
        with:
          name: website-dist
          path: website/dist
          retention-days: 7
```

- [ ] **Step 2: Verify the YAML parses locally** (typo-check before pushing).

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/website.yml'))"`
Expected: no output (success).

- [ ] **Step 3: Commit.**

```bash
git add .github/workflows/website.yml
git commit -m "ci(website): add website workflow with doctests + build

Path-filtered to website/, crates/neon/, crates/neon-macros/, plus the
workflow file itself. Uploads the built dist as an artifact so reviewers
can poke at it without spinning up Netlify."
```

### Task 8.2: Add the Netlify configuration

**Files:**

- Create: `website/netlify.toml`

- [ ] **Step 1: Create `website/netlify.toml`.**

```toml
[build]
  base = "website"
  command = "npm run build"
  publish = "dist"

[build.environment]
  NODE_VERSION = "20"
  NPM_VERSION = "10"
  # Provision Rust on Netlify's Ubuntu image. RUST_VERSION is honored by
  # Netlify's build image when present.
  RUST_VERSION = "stable"

[[headers]]
  # Long cache for hashed Astro assets.
  for = "/_astro/*"
  [headers.values]
    Cache-Control = "public, max-age=31536000, immutable"
```

The `base = "website"` tells Netlify to run the build with `website/` as cwd; combined with `publish = "dist"`, the published directory is `website/dist`. The `NODE_VERSION` pin should match what we use in CI (Node 20).

If `RUST_VERSION = "stable"` doesn't actually trigger a rustup install on Netlify's current image (the env var honoring this varies), fall back to a build hook: add a `[[plugins]]` block running a small script that does `curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable && source $HOME/.cargo/env`. Verify on the first deploy preview which path actually works.

- [ ] **Step 2: Commit.**

```bash
git add website/netlify.toml
git commit -m "ci(website): add Netlify configuration

Builds from the website/ subdirectory with Node 20 and Rust stable.
Netlify's GitHub integration handles deploy previews on PRs without
additional configuration once the project is connected to this repo."
```

### Task 8.3: Connect the Netlify site (manual)

This task is not automatable from a PR. The repository owner must perform these steps in the Netlify dashboard once.

- [ ] **Step 1: In Netlify, create a new site connected to the `neon-bindings/neon` GitHub repository.** Branch: `main`. Build settings: leave blank (will pick up `website/netlify.toml`).

- [ ] **Step 2: In the new site's settings, enable "Deploy previews" for pull requests.** This is on by default but should be confirmed.

- [ ] **Step 3: Open a draft PR that touches `website/`** and confirm Netlify posts a preview URL comment. Verify the URL serves the new site, including `/api/neon/`.

- [ ] **Step 4: Once the new site is live and known good, plan the DNS cutover** from the old Docusaurus site to the new one. The `site` field in `astro.config.mjs` is already set to `https://neon-rs.dev`, so no code changes are needed for the cutover. Out of scope for this plan; flag for follow-up.

---

## Phase 9 (optional, follow-up): Branch protection

After all PRs above are merged, the repository owner should add the `Website / Doctests` job as a required status check on PRs that match the path filters. This is a GitHub repository setting, not a code change.

- [ ] **Step 1: In GitHub repo settings → Branches → main → Branch protection rule, add `Website / Doctests` to "Required status checks".**

- [ ] **Step 2: Verify the check is enforced** by opening a PR that deliberately breaks a sample (e.g., changes the `hello` doctest's expected return type) and confirming GitHub blocks merge.

---

## Self-review

Before declaring this plan complete, the implementer should verify:

1. Every page listed in the spec's "Information architecture" section exists as a stub after Phase 5. There are 30 stub pages; count them with `find website/src/content/docs -name '*.md' -o -name '*.mdx' | wc -l`.
2. `cargo test -p website --doc` passes.
3. `cargo neon-test --no-run` still succeeds (we haven't broken the matrix).
4. `npm run build --workspace=@neon-rs/website` produces `dist/index.html` and `dist/api/neon/index.html`.
5. The hero page contains the tagline "Write Node addons in Rust" and the sub-line "Safe, fast, parallel."
6. The Slack invite link in the spec (<https://join.slack.com/t/rust-bindings/shared_invite/zt-1pl5s83xe-ZvXyrzL8vuUmijU~7yiEcg>) appears on the contributing page.
7. No file in `website/src/content/docs/` has a Rust fence whose body fails to compile against `crates/neon`'s in-tree feature set.

If any of those fail, the offending phase is not complete.
