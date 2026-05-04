---
title: Access Cx from an exported function
description: Reach Cx and FunctionContext inside a function exported with #[neon::export].
---

:::caution[Status: skeleton]
This page is a placeholder. Content forthcoming.
:::

This guide shows how to get hold of `Cx` (or `FunctionContext`) inside a function exported with `#[neon::export]`, for the cases where the high-level extractor APIs aren't enough and you need direct access to the JavaScript context to build values, throw, or look things up on the global object.
