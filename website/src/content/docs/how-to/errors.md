---
title: Throw and catch JavaScript errors from Rust
description: Throw exceptions from Rust, catch them, and use extract::Error with ? for ergonomic error handling.
---

:::caution[Status: skeleton]
This page is a placeholder. Content forthcoming.
:::

This guide shows how to raise a JavaScript exception from a Neon function, how to catch JS-thrown errors when calling back into JavaScript, and how to use `extract::Error` together with the `?` operator so Rust `Result` types map cleanly to thrown JS exceptions.
