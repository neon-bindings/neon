---
title: Export async functions
description: Write async fn exports backed by a global executor like tokio.
status: todo
---

This guide shows how to export an `async fn` from a Neon addon so it returns a JavaScript `Promise`. It covers registering a global async executor (such as tokio) at addon load and the constraints on what can be `.await`ed inside an exported function.
