---
title: Use serde with Json<T>
description: Move structured data across the Neon boundary using Json<T> and the json shorthand attribute.
status: todo
---

This guide covers Neon's `Json<T>` wrapper and the `json` shorthand on `#[neon::export]`, which let you accept and return any `serde`-compatible Rust type as a JavaScript value. It is the easiest way to ferry structured data between Rust and JS without writing manual conversions.
