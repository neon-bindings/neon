---
title: Run blocking work on the libuv pool
description: Offload synchronous, CPU- or IO-bound code to Node's libuv worker pool with #[neon::export(task)].
---

:::caution[Status: skeleton]
This page is a placeholder. Content forthcoming.
:::

This guide shows how to mark a synchronous Rust function with `#[neon::export(task)]` so it runs on Node's libuv worker pool instead of blocking the JavaScript main thread. The exported function returns a `Promise` that resolves with the function's result.
