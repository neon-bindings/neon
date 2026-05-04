---
title: How #[neon::export] works
description: A look under the hood at the code the export macro generates and the runtime pieces it relies on.
---

:::caution[Status: skeleton]
This page is a placeholder. Content forthcoming.
:::

This page explains what `#[neon::export]` actually does: how it registers the function with the module init, the wrapper code it generates around your Rust function, and how the `task` and `async` variants compose with the rest of Neon's runtime. It is aimed at readers who want to understand the macro rather than just use it.
