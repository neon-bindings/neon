---
title: Async functions with tokio
description: Register a global tokio executor and write async fn exports that resolve JavaScript Promises.
---

:::caution[Status: skeleton]
This page is a placeholder. Content forthcoming.
:::

This tutorial walks through wiring up a global tokio runtime in your Neon module and writing `async fn` exports that integrate naturally with Node's `Promise`-based concurrency model. You will see how to register the executor once at module load and then `.await` futures from inside an exported async function.
