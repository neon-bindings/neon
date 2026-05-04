---
title: Concurrency with the libuv pool
description: Convert a CPU-bound function to run on Node's libuv worker pool with #[neon::export(task)].
---

:::caution[Status: skeleton]
This page is a placeholder. Content forthcoming.
:::

This tutorial takes a synchronous CPU-bound Rust function and moves it off the JavaScript main thread by exporting it as `#[neon::export(task)]`. The result runs on Node's libuv worker pool and resolves a `Promise` on completion, keeping the event loop responsive while the work runs.
