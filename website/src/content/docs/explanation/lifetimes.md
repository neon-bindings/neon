---
title: Handle lifetimes
description: Why Neon handles carry a 'cx lifetime, and what it protects you from.
---

:::caution[Status: skeleton]
This page is a placeholder. Content forthcoming.
:::

This page explains the `'cx` lifetime that appears on Neon handles: what it represents, why Neon ties handle validity to the lifetime of a context, and what kinds of bugs that design rules out at compile time.
