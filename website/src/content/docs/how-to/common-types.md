---
title: Pass common types between Rust and JavaScript
description: Send numbers, strings, arrays, objects, and buffers across the Neon boundary.
---

:::caution[Status: skeleton]
This page is a placeholder. Content forthcoming.
:::

This guide shows the everyday conversions: accepting and returning numbers, strings, arrays, objects, and buffers in functions exported with `#[neon::export]`. It focuses on the common types that cover most JavaScript-facing APIs, with pointers to the more specialized guides for `serde`, classes, and streaming.
