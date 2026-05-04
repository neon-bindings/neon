# Smoke test

This file exists to verify the doctest harness works. It will be deleted in a later task once we have real content.

```rust
# use neon::prelude::*;
#[neon::export]
fn hello() -> &'static str {
    "Hello from a doctest"
}
```
