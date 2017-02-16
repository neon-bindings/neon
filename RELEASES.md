# Version 0.1.12 (2017-02-15)

* [Optimized rooting protocol](https://github.com/neon-bindings/neon/commit/cef41584d9978eda2d59866a077cfe7c7d3fa46e)
* [Eliminate rustc warnings](https://github.com/neon-bindings/neon/pull/107)
* Lots of internal API docs
* Windows support! :tada:
* [Renamed `neon-sys` to `neon-runtime`](https://github.com/neon-bindings/neon/issues/169)

# Version 0.1.11 (2016-08-08)

* [Exposed `This` trait](https://github.com/neon-bindings/neon/issues/101) to allow user-level abstractions involving `FunctionCall`

# Version 0.1.10 (2016-05-11)

* Added `JsError` API with support for throwing [all](https://github.com/neon-bindings/neon/issues/65) [standard](https://github.com/neon-bindings/neon/issues/66) [error](https://github.com/neon-bindings/neon/issues/67) [types](https://github.com/neon-bindings/neon/issues/74)
* [Test harness and CI integration](https://github.com/neon-bindings/neon/issues/80)!! :tada: :tada: :tada:
* API to [call JS functions from Rust](https://github.com/neon-bindings/neon/issues/60)
* API to [new JS functions from Rust](https://github.com/neon-bindings/neon/issues/61)
* Added [generalized `as_slice` and `as_mut_slice` methods](https://github.com/neon-bindings/neon/issues/64) to `CSlice` API.
* Fixed a [soundness issue](https://github.com/neon-bindings/neon/issues/64) with Locks.

## Incompatible Changes

* The `JsTypeError` type is gone, and replaced by the more general `JsError` type.
* `neon::js::error::JsTypeError::throw(msg)` is now `neon::js::error::JsError::throw(neon::js::error::kind::TypeError, msg)`
