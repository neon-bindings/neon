# Version 0.1.10 (2016-05-11)

* Added `JsError` API with support for throwing [all](https://github.com/rustbridge/neon/issues/65) [standard](https://github.com/rustbridge/neon/issues/66) [error](https://github.com/rustbridge/neon/issues/67) [types](https://github.com/rustbridge/neon/issues/74)
* [Test harness and CI integration](https://github.com/rustbridge/neon/issues/80)!! :tada: :tada: :tada:
* API to [call JS functions from Rust](https://github.com/rustbridge/neon/issues/60)
* API to [new JS functions from Rust](https://github.com/rustbridge/neon/issues/61)
* Added [generalized `as_slice` and `as_mut_slice` methods](https://github.com/rustbridge/neon/issues/64) to `CSlice` API.
* Fixed a [soundness issue](https://github.com/rustbridge/neon/issues/64) with Locks.

## Incompatible Changes

* The `JsTypeError` type is gone, and replaced by the more general `JsError` type.
* `neon::js::error::JsTypeError::throw(msg)` is now `neon::js::error::JsError::throw(neon::js::error::kind::TypeError, msg)`
