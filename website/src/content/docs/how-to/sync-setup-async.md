---
title: Synchronous setup before async work
description: Use #[neon::export(async)] to do synchronous setup on the main thread, then return a Future for the async portion.
---

:::caution[Status: skeleton]
This page is a placeholder. Content forthcoming.
:::

This guide shows how to use `#[neon::export(async)]` on a function that runs synchronously to do setup on the JavaScript main thread (where it has access to `Cx`) and then returns an `impl Future` for the asynchronous portion. This pattern is useful when you need to read JS values before kicking off background work.

```rust
# use neon::prelude::*;
# use neon::types::extract::{with, Error, TryIntoJs};
# async fn fetch_user(_id: u64) -> Result<String, Error> { Ok(String::new()) }
#[neon::export]
async fn load_user(id: f64) -> impl for<'cx> TryIntoJs<'cx> {
    println!("Hello from the JavaScript main thread!");
    let user = fetch_user(id as u64).await;
    with(move |cx| user.try_into_js(cx))
}
```
