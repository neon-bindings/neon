---
title: Return to the main thread after async work
description: Use extract::with to hop back to the JavaScript main thread after awaiting work on another executor.
---

:::caution[Status: skeleton]
This page is a placeholder. Content forthcoming.
:::

This guide shows how to use `extract::with` to schedule a closure back on the JavaScript main thread after `.await`ing work on a background executor. That closure receives a `Cx`, so you can build the final JavaScript value to resolve the `Promise` with.
