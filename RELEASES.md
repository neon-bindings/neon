# Version ✨0.2✨ 🎉

See the [Neon 0.2 Migration Guide](https://github.com/neon-bindings/neon/wiki/Neon-0.2-Migration-Guide) for documentation on migrating your projects from the Neon 0.1.x series to Neon 0.2, and please [let us know](https://github.com/neon-bindings/neon#get-involved) if you need help!

* Release automation (#318)
* New `ArrayBuffer` views API -- see [RFC 5](https://github.com/neon-bindings/rfcs/blob/master/text/0005-array-buffer-views.md) (#306)
* VM 2.0 -- see [RFC 14](https://github.com/neon-bindings/rfcs/blob/master/text/0014-vm-2.0.md) (#306)
* New `JsString` constructor -- see [RFC 21](https://github.com/neon-bindings/rfcs/blob/master/text/0021-string-constructor.md) (#322)
* Eliminated `JsInteger`, `JsVariant`, `callee()` -- see [RFC 22](https://github.com/neon-bindings/rfcs/blob/master/text/0022-zero-dot-two.md) (#323)
* Renamed `Key` to `PropertyKey` and its method names -- see [RFC 22](https://github.com/neon-bindings/rfcs/blob/master/text/0022-zero-dot-two.md) (#323)
* Module reorganization -- see [RFC 20](https://github.com/neon-bindings/rfcs/blob/master/text/0020-module-reorg.md) (#324)
* New `JsError` API -- see [RFC 23](https://github.com/neon-bindings/rfcs/blob/master/text/0023-error-subtyping.md) (#325)
* Eliminated `ToJsString` API -- see [RFC 22](https://github.com/neon-bindings/rfcs/blob/master/text/0022-zero-dot-two.md) (#326)
* Eliminated `NEON_NODE_ABI` env var -- see [RFC 22](https://github.com/neon-bindings/rfcs/blob/master/text/0022-zero-dot-two.md) (#327)
* Default to release builds -- see [RFC 22](https://github.com/neon-bindings/rfcs/blob/master/text/0022-zero-dot-two.md) (#328)
* Made `Buffer` construction safe by default (#329, #331)
* Made `Throw` not implement `std::error::Error` to avoid accidental suppression, thanks to [@kjvalencik](https://github.com/kjvalencik) (#334)
* Fixed a bug causing unnecessary rebuilds, thanks to [@kjvalencik](https://github.com/kjvalencik) (#343)
* Fixed a soundness bug in the `Task` API, thanks to [@kjvalencik](https://github.com/kjvalencik) (#335)

# Version 0.1.23

* Optimization in `Scope` structures, thanks to [@maciejhirsz](https://github.com/maciejhirsz) (#282)
* Fixed a memory leak in the `Task` API, thanks to [@kjvalencik](https://github.com/kjvalencik) (#291)
* Add support for Node 10, thanks to [@mhsjlw](https://github.com/mhsjlw) and [@nomadtechie](https://github.com/nomadtechie) (#314)

# Version 0.1.22

* Reinstate `JsInteger` (although it's deprecated) for now, to be removed in 0.2. (#279)

# Version 0.1.21

* Fix a bug that was causing annoying unnecessary rebuilds ([#242](https://github.com/neon-bindings/neon/issues/242)).
* New [API for getting the global object](https://api.neon-bindings.com/neon/scope/trait.scope#method.global) ([#249](https://github.com/neon-bindings/neon/issues/249)).

# Version 0.1.20

* Background task API ([#214](https://github.com/neon-bindings/neon/pull/214)).
* Fixes to Windows builds ([#221](https://github.com/neon-bindings/neon/pull/221), [#227](https://github.com/neon-bindings/neon/pull/227)), thanks to [@hone](https://github.com/hone)'s tenacious troubleshooting.

# Version 0.1.19

* TypeScript upgrade fixes ([neon-bindings/neon-cli#62](https://github.com/neon-bindings/neon-cli/pull/62), [neon-bindings/neon-cli#65](https://github.com/neon-bindings/neon-cli/pull/65)).

# Version 0.1.18

* CLI bugfix ([neon-bindings/neon-cli#59](https://github.com/neon-bindings/neon-cli/pull/59)).
* JsArrayBuffer ([#210](https://github.com/neon-bindings/neon/pull/210)).

# Version 0.1.17

* CLI bugfix ([neon-bindings/neon-cli#57](https://github.com/neon-bindings/neon-cli/pull/57)).

# Version 0.1.16

* CLI bugfix ([neon-bindings/neon-cli#56](https://github.com/neon-bindings/neon-cli/pull/56)).

# Version 0.1.15 (2017-05-21)

* Better Electron support in CLI's build process.
* Better support for Electron via the artifacts file ([neon-bindings/neon-cli#52](https://github.com/neon-bindings/neon-cli/pull/52)).

# Version 0.1.14 (2017-04-02)

* Ensure failing tests break the build ([#191](https://github.com/neon-bindings/neon/pull/191))
* Catch Rust panics and convert them to JS exceptions ([#192](https://github.com/neon-bindings/neon/pull/192))
* Implement `Error` for `Throw` ([#201](https://github.com/neon-bindings/neon/pull/191))
* Clean up the CLI and allow `neon build` to optionally take module names ([neon-bindings/neon-cli#48](https://github.com/neon-bindings/neon-cli/pull/48)).

# Version 0.1.13 (2017-02-17)

* More robust build scripts for neon-runtime, fixing Homebrew node installations (see [#189](https://github.com/neon-bindings/neon/pull/189))

# Version 0.1.12 (2017-02-16)

* [Optimized rooting protocol](https://github.com/neon-bindings/neon/commit/cef41584d9978eda2d59866a077cfe7c7d3fa46e)
* [Eliminate rustc warnings](https://github.com/neon-bindings/neon/pull/107)
* Lots of internal API docs
* Windows support! :tada:
* [Renamed `neon-sys` to `neon-runtime`](https://github.com/neon-bindings/neon/issues/169)
* Depend on `neon-build` as a build dependency (see [neon-bindings/neon-cli#46](https://github.com/neon-bindings/neon-cli/issues/46)).

# Version 0.1.11 (2016-08-08)

* [Exposed `This` trait](https://github.com/neon-bindings/neon/issues/101) to allow user-level abstractions involving `FunctionCall`
* Bump version to match Neon so they can be kept in sync from now on.
* Generate a `build.rs` to make Windows work (see [neon-bindings/neon-cli#42](https://github.com/neon-bindings/neon-cli/pull/42) and [neon-bindings/neon-cli#44](https://github.com/neon-bindings/neon-cli/issues/44)).

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
