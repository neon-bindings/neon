---
title: Threading and value lifecycle
description: How Channel, Root, and Deferred coordinate JavaScript values across threads.
---

:::caution[Status: skeleton]
This page is a placeholder. Content forthcoming.
:::

This page explains how Neon manages the lifecycle of JavaScript values across threads: `Channel` for scheduling work back on the main thread, `Root` for keeping a JS value alive across thread boundaries, and `Deferred` for resolving a `Promise` from a background thread. It is aimed at readers who want to understand the model that the higher-level export macros sit on top of.
