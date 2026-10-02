---
title: Cancel async work with AbortController
description: Bridge a JavaScript AbortController to a tokio CancellationToken so async Neon functions can be cancelled.
status: todo
---

This guide shows how to accept a JavaScript `AbortController` (or its `AbortSignal`) and adapt it to a tokio `CancellationToken` inside a Neon async function, so that aborting on the JS side propagates cancellation into the running Rust future. The pattern mirrors the example in [neon-bindings/examples PR #104](https://github.com/neon-bindings/examples/pull/104).
