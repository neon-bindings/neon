---
title: Rename exports
description: Customize the JavaScript-facing name of a function exported with #[neon::export].
---

:::caution[Status: skeleton]
This page is a placeholder. Content forthcoming.
:::

This guide shows how to give an exported Neon function a JavaScript name that differs from its Rust identifier — typically because Rust prefers `snake_case` while JavaScript APIs prefer `camelCase`. The rename is configured on the `#[neon::export]` attribute itself.
