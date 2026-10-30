---
title: Prerequisites
description: Set up Node.js, Rust, and your platform's build tools, then scaffold a new project with npm init neon@latest.
---

To build and run a Neon addon you need three things on your machine:
**Node.js**, the **Rust toolchain**, and your platform's standard
**build tools**. These are the same tools every Rust and Node.js
developer keeps installed; if you have either ecosystem set up already,
you're most of the way there.

Once they're in place, `npm init neon@latest my-project` scaffolds a
new project and you're ready to write code.

## 1. Node.js

Neon actively supports every [current and LTS release of
Node.js](https://github.com/nodejs/release#release-schedule). Any
recent version from your preferred installer (the
[official installer](https://nodejs.org/), [`nvm`](https://github.com/nvm-sh/nvm),
[`fnm`](https://github.com/Schniz/fnm), [Volta](https://volta.sh/), or your
package manager) is fine.

Older Node releases (down to Node 10) may also work, but you'll need to
target an older Node-API level — see the
[Node-API version matrix](https://nodejs.org/api/n-api.html#node-api-version-matrix)
and [Supported platforms](/reference/supported-platforms/).

```sh
node --version
```

If that prints a version, you're set.

## 2. Rust

Install Rust through [`rustup`](https://rustup.rs/), the official
toolchain installer. Neon is tested against the current stable and
nightly channels; scaffolded projects use the 2024 edition, which
requires **Rust 1.85 or newer**.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After it finishes, restart your shell (or `source ~/.cargo/env`) and
verify:

```sh
rustc --version
```

## 3. Platform build tools

Some Rust crates that Neon depends on (or that you'll bring in
yourself) link against system libraries, so you need your platform's
standard build toolchain available.

**macOS** — Install the Xcode Command Line Tools:

```sh
xcode-select --install
```

**Linux** — On Debian/Ubuntu, install `build-essential` (or its
equivalent on your distro):

```sh
sudo apt install build-essential
```

**Windows** — Install the **Visual Studio Build Tools** with the
"Desktop development with C++" workload. The
[`rustup` installer](https://rustup.rs/) prompts you for this when it
detects Windows; saying yes is the easiest path.

## 4. Scaffold a project

With the toolchains in place, create a new Neon project anywhere you
keep your code:

```sh
npm init neon@latest my-project
```

That runs the [`create-neon`](/reference/cli/) scaffolder, which asks a
few questions (project name, license, etc.) and then generates a fresh
project with everything wired together.

```text
my-project/
├── .gitignore
├── Cargo.toml          # Rust crate manifest
├── package.json        # npm package manifest
├── src/
│   └── lib.rs          # your Rust code
└── README.md
```

The default `src/lib.rs` exports a tiny `hello` function so you can
verify the build works end-to-end:

```rust
# assert_eq!(hello("world".into()), "hello world");
#[neon::export]
fn hello(name: String) -> String {
    format!("hello {name}")
}
```

`cd` into the project, install the npm dependencies, and build the
native addon:

```sh
cd my-project
npm install
npm run build
```

If that finishes without errors, your toolchain is healthy and Neon is
ready to go. Head to the [Quickstart](/getting-started/quickstart/) to
write your first function. If anything went wrong, double-check the
prerequisites above and look at
[Supported platforms](/reference/supported-platforms/) for known
constraints.
