---
title: Publish your addon to npm
description: Take a working Neon addon and ship it to npm so anyone can `npm install` it on Linux, macOS, or Windows without needing a Rust toolchain.
status: todo
---

This tutorial walks through publishing a working Neon addon to npm as a package that installs prebuilt binaries on every supported platform. It covers cross-platform CI builds, packaging per-platform binaries as `optionalDependencies`, the runtime selector that picks the right binary at install time, and the npm publish workflow.
