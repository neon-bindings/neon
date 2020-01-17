# [WIP] N-API Migration Guide

## Context

Many methods that previously did not require context (e.g., `JsString::size`) now require a context. In many cases, this means adding an additional argument or using a convenience method on the `Context` trait.

### Impacted methods

* `JsString`
    - `size`
    - `value`
* `PropertyKey`
    - `get_from`
    - `set_from`
