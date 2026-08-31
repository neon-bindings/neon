---
title: Expose Rust types as JavaScript classes
description: Use #[neon::class] to expose a Rust struct as a JavaScript class with constructors and methods.
status: todo
---

This guide shows how to use `#[neon::class]` to expose a Rust struct as a JavaScript class, including constructors, methods, and shared state. It covers the pieces of the macro you usually need and how the resulting class behaves on the JavaScript side.
