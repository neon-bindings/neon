# Version 1.0.0-alpha.2

## Breaking Changes

### `neon::object::This`

https://github.com/neon-bindings/neon/pull/918

Trait [`neon::object::This`](https://docs.rs/neon/latest/neon/object/trait.This.html) has been removed. `This` was primarily added for use with the `declare_types!` macro to generate classes. The macro was removed and `This` is no longer needed. Additionally, the `This` argument on `JsFunction` was found to be _invalid_ because it asserted at compile time a type for `this` that could change at runtime. (Note that this was _not_ unsound because the type would be checked by Node-API and result in a `panic`.)

### `JsFunction::this`

https://github.com/neon-bindings/neon/pull/918

`JsFunction::this` was changed to perform a downcast and be _fallible_. This is in line with similar APIs (e.g., `Object::get`). Additionally, an infallible version, `JsValue::this_value` was added that does _not_ perform a downcast.

### Added Feature flag for external buffers

https://github.com/neon-bindings/neon/pull/937

Electron began using [pointer compression](https://www.electronjs.org/blog/v8-memory-cage) on JavaScript values that is incompatible with external buffers. As a preventative measure, `JsArrayBuffer::external` and `JsBuffer::external` have been placed behind a feature flag that warns of Electron incompatibility.

## Improvements

* Lifetimes were relaxed on `execute_scoped` to allow valid code to compile. (https://github.com/neon-bindings/neon/pull/919)
* Added a `from_slice` helper on `TypedArray` (https://github.com/neon-bindings/neon/pull/925)
* `JsTypedArray` construction and type aliases (https://github.com/neon-bindings/neon/pull/909)

## Bug Fixes

* Fixed a panic on VM shutdown when using `Channel` (https://github.com/neon-bindings/neon/pull/934)
* Type tags were added to `JsBox` to prevent undefined behavior when multiple native add-ons are used (https://github.com/neon-bindings/neon/pull/907)

## Docs

* Significantly improved documentation of `TypedArray` (https://github.com/neon-bindings/neon/pull/909)
* Removed unused values in `Channel` docs (https://github.com/neon-bindings/neon/pull/925)

### `cargo-cp-artifact`

`0.1.7` includes a fix to unlink `.node` files before copying to address common code signing errors on macOS (https://github.com/neon-bindings/neon/pull/921).

# Version 1.0.0-alpha.1

Pre-release of a major milestone for Neon. 1.0.

## Breaking Changes

### Major

* Removed the legacy backend; only Node-API is supported going forward (https://github.com/neon-bindings/neon/pull/881)
* Removed `neon::result::JsResultExt` in favor of more general `neon::result::ResultExt` (https://github.com/neon-bindings/neon/pull/904)

### Minor

* Length APIs (`argument`, `argument_ops`, `len`) use `usize` instead of `i32` (https://github.com/neon-bindings/neon/pull/889)
* Deprecate feature flags for accepted RFCs (https://github.com/neon-bindings/neon/pull/872)
* `neon::meta::version` returns `semver@1` version instead of `0.9` (https://github.com/neon-bindings/neon/pull/912)

## Features

* Add `Object.freeze` and `Object.seal` (https://github.com/neon-bindings/neon/pull/891)
* Futures RFC (https://github.com/neon-bindings/neon/pull/872) Implementation (https://github.com/neon-bindings/neon/pull/874)
  - Await `JoinHandle` from sending an event on a `Channel`
  - Adapt `JsPromise` to `JsFuture`
* API for thread-local data (i.e., instance data) (https://github.com/neon-bindings/neon/pull/902)
* Add Object::call_with() convenience method to call a method on an object (https://github.com/neon-bindings/neon/pull/879)

## Bug Fixes

* Relax the lifetime constraints on `TypedArray` borrows (https://github.com/neon-bindings/neon/pull/877)
* Allowing missing symbols at load time to support [bun](https://bun.sh) (https://github.com/neon-bindings/neon/pull/914)
* Prevent a panic when an async event is called after the JavaScript runtime has stopped (https://github.com/neon-bindings/neon/pull/913)
* Fix a soundness hole in `JsArrayBuffer::external` and `JsBuffer::external` (https://github.com/neon-bindings/neon/pull/897)

## Docs

* Fixed mistake in `Object::get` docs (https://github.com/neon-bindings/neon/pull/903)
* Fixed link in README to migration guide (https://github.com/neon-bindings/neon/pull/895)

## Internal

* Moved `cargo-cp-artirfact` into the monorepo (https://github.com/neon-bindings/neon/pull/905)
* Decreased the size of the Neon build matrix (https://github.com/neon-bindings/neon/pull/893)
* Removed scope abstraction from legacy backend (https://github.com/neon-bindings/neon/pull/888)
* Improved the monorepo structure of neon (https://github.com/neon-bindings/neon/pull/884)

# Version 0.10.1

Fix a soundness hole in `JsArrayBuffer::external`
and `JsBuffer::external` (https://github.com/neon-bindings/neon/pull/897).

Thanks to [@Cassy343](https://github.com/Cassy343) for finding the [issue](https://github.com/neon-bindings/neon/issues/896)!

In previous versions of Neon, it was possible to create a `JsArrayBuffer` or `JsBuffer` that references data without the `'static` lifetime.

```rust
pub fn soundness_hole(mut cx: FunctionContext) -> JsResult<JsArrayBuffer> {
    let mut data = vec![0u8, 1, 2, 3];

    // Creating an external from `&mut [u8]` instead of `Vec<u8>` since there is a blanket impl
    // of `AsMut<T> for &mut T`
    let buf = JsArrayBuffer::external(&mut cx, data.as_mut_slice());

    // `buf` is still holding a reference to `data`!
    drop(data);

    Ok(buf)
}
```

# Version 0.10

See the [Neon 0.10 Migration Guide](docs/MIGRATION_GUIDE_0.10.md) for more details about new features and breaking changes.

## Features

* New [buffer borrowing API](https://github.com/neon-bindings/neon/pull/780)
* Added [JoinHandle](https://github.com/neon-bindings/neon/pull/787) for `Channel::send`
* [`JsPromise` and `TaskBuilder`](https://github.com/neon-bindings/neon/pull/789)
* Handle [panics and exceptions](https://github.com/neon-bindings/neon/pull/808) in Channels and Tasks
* [Function call / construct builders](https://github.com/neon-bindings/neon/pull/829)
  and [simplify low level call](https://github.com/neon-bindings/neon/pull/825)
* Create [functions from closures](https://github.com/neon-bindings/neon/pull/811)

## Minor Improvements

* [Performance improvements](https://github.com/neon-bindings/neon/pull/815)
* [Rename N-API to Node-API](https://github.com/neon-bindings/neon/pull/753) in docs to match Node changes
* Remove unused [cslice dependency](https://github.com/neon-bindings/neon/pull/794)
* Switch to [`syn-mid`](https://github.com/neon-bindings/neon/pull/814) for faster compile times
* Downcast in [`Object::get`](https://github.com/neon-bindings/neon/pull/839)
* Added [migration guide](https://github.com/neon-bindings/neon/pull/859)
* Added [`Object::get_opt` and `Object::get_value`](https://github.com/neon-bindings/neon/pull/867)

## Fixes

* [Safety] Make it harder to store and forge [Throw](https://github.com/neon-bindings/neon/pull/797)
* [Soundness] [Make `JsValue` types `!Copy`](https://github.com/neon-bindings/neon/pull/832)
* [Soundness] [Tag `Root`](https://github.com/neon-bindings/neon/pull/847) with instance id
* `create-neon` no longer [leaves partial project on disk](https://github.com/neon-bindings/neon/pull/840)
* Fix legacy backend on [Electron and Windows](https://github.com/neon-bindings/neon/pull/785)
* [FreeBSD support](https://github.com/neon-bindings/neon/pull/856) on legacy backend

## Internal Improvements

* Replace Electron tests [with Playwright](https://github.com/neon-bindings/neon/pull/835)
* Re-organize Neon into an [npm workspace](https://github.com/neon-bindings/neon/pull/852)
* [Fix crates.io badge](https://github.com/neon-bindings/neon/pull/781)
* [Doc test fixes](https://github.com/neon-bindings/neon/pull/800)
* Fix [broken link](https://github.com/neon-bindings/neon/pull/804) in the README

# Version 0.9.1

* Expose the `Finalize` trait as `neon::types::Finalize` so that docs are visible
* Improved docs and build scripts in `create-neon` to make release builds more
  discoverable (https://github.com/neon-bindings/neon/pull/771)
* Update `nan` to fix an Electron 13 incompatibility (https://github.com/neon-bindings/neon/pull/778)

# Version 0.9.0

## Performance

`Channel`, formerly `EventQueue`, are now cloneable. Clones share a backing queue to take advantage of an [optimization](https://github.com/nodejs/node/pull/38506) in Node threadsafe functions. Additionally, when specifying Node API 6 or higher (`napi-6`), calling `cx.channel()` will return a shared queue (https://github.com/neon-bindings/neon/pull/739).

The change may cause a performance regression in some pathological use cases (https://github.com/neon-bindings/neon/issues/762).

## Deprecation

`EventQueue` and `EventQueueError` have been renamed to `Channel` and `ChannelError` respectively to clarify their function and similarity to Rust channels. The types are available as deprecated aliases (https://github.com/neon-bindings/neon/pull/752).

## Docs

* Document error causes for `Channel::try_send` docs (https://github.com/neon-bindings/neon/pull/767)
* Document `neon::object` (https://github.com/neon-bindings/neon/pull/740)

## Fixes

* Fix usage of a removed API in legacy buffers (https://github.com/neon-bindings/neon/pull/769)

# Version 0.8.3

* Fix crash caused by non-thread safety in napi_threadsafefunction on early termination (https://github.com/neon-bindings/neon/pull/744)
* Fix memory leak in `Root` (https://github.com/neon-bindings/neon/pull/750)

# Version 0.8.2

* More docs improvements
* Added a deprecation warning to `neon new` (https://github.com/neon-bindings/neon/pull/722)

# Version 0.8.1

* Fix `legacy-backend` for Node 16 (https://github.com/neon-bindings/neon/pull/715)
* Various docs improvements

# Version 0.8.0

## Fixes

* `as_slice` and `as_mut_slice` properly handle a `null` pointer from an empty buffer (https://github.com/neon-bindings/neon/pull/681)
* Global drop queue added to avoid panics on N-API 6+ when dropping a `Root` (https://github.com/neon-bindings/neon/pull/700)

## Features

* Added `neon::reflect::eval` (https://github.com/neon-bindings/neon/pull/692)
* Added `create-neon` for creating an N-API project (https://github.com/neon-bindings/neon/pull/690)
* Added details to the `README.md` generated by `create-neon` (https://github.com/neon-bindings/neon/pull/697)

## Improvements

* Switched N-API tests to `cargo-cp-artifact` (https://github.com/neon-bindings/neon/pull/687)
* Added `impl<T: Finalize> Finalize for Option<T>` (https://github.com/neon-bindings/neon/pull/680)
* Added a N-API migration guide (https://github.com/neon-bindings/neon/pull/685)

## Housekeeping

* Lint fixes (https://github.com/neon-bindings/neon/pull/609)
* Lint CI enforcement and `cargo fmt` (https://github.com/neon-bindings/neon/pull/698)

# Version 0.7.1

### Features

* Added `JsDate` to N-API backend (https://github.com/neon-bindings/neon/pull/639)
* Implement `JsBuffer::unitialized` for N-API backend (https://github.com/neon-bindings/neon/pull/664)

### Fixes

* Do not panic if a `Root` is leaked after the event loop has stopped (https://github.com/neon-bindings/neon/pull/677)
* Stubs for features that will not be implemented in the N-API backend are removed (https://github.com/neon-bindings/neon/pull/663)
* Fix doc URL link (https://github.com/neon-bindings/neon/pull/663)

# Version 0.7.0

## N-API

### Version Selection

Neon supports a large number of different Node versions which may have different N-API requirements. Neon now supports selecting the minimum required N-API version required by a module. For example, for N-API Version 4:

```toml
neon = { version = "0.7", default-features = false, features = ["napi-4"] }
```

If the Neon module is loaded in an older version of Node that does not support that N-API version, a `panic` message will inform the user.

### Threadsafe Functions

A prerelease version of `EventQueue` for calling into the main JavaScript thread from Rust threads can be enabled with the `event-queue-api` feature flag. The API is considered unstable and may change in the future until the [RFC](https://github.com/neon-bindings/rfcs/pull/32) is merged.

# Version 0.6.0

The `cx.try_catch(..)` API has been updated to return `T: Sized` instead of `T: Value` (https://github.com/neon-bindings/neon/pull/631). This API is strictly more powerful and allows users to return both JavaScript and Rust values from `try_catch` closures.

## N-API

* N-API symbols are now loaded dynamically (https://github.com/neon-bindings/neon/pull/646)
* Build process for N-API is greatly simplified by leveraging dynamic loading (https://github.com/neon-bindings/neon/pull/647)

# Version 0.5.3

## Bug Fixes

Upgrade `node-gyp` (https://github.com/neon-bindings/neon/pull/623)
* Fix Windows Node 15
* Fix Apple M1

## Features

Added `neon::main` macro as a replacement for `register_module!` (https://github.com/neon-bindings/neon/pull/636)

## Known Issues

Builds occassionally fail with Windows, Node 15 and npm 7 (https://github.com/neon-bindings/neon/issues/642)

# Version 0.5.2

## CLI

Added support for [additional arguments](https://github.com/neon-bindings/neon/pull/633) passed to `cargo build`. Resolves https://github.com/neon-bindings/neon/issues/471.

```sh
neon build --release -- --features awesome
```

## N-API

* Improved [arguments performance](https://github.com/neon-bindings/neon/pull/610)
* Add [redirect and `NPM_CONFIG_DISTURL`](https://github.com/neon-bindings/neon/pull/620) support

# Version 0.5.1

## Performance

* `smallvec` is used for collecting arguments and yields a small performance gain when calling `JsFunction`

## Broader Support

Thanks to @staltz, neon now builds for both iOS and Android with [nodejs-mobile](https://github.com/JaneaSystems/nodejs-mobile).

# Version 0.5.0

_Re-publish_

Versions `0.4.1` and `0.4.2` included a breaking change in `neon-runtime`. At the time, this was considered acceptable because `neon-runtime` is considered an internal crate and not part of the public API. However, it was discovered, after publishing, that `neon-serde`, a commonly used crate in the `neon` ecosystem, contained a direct dependency on `neon-runtime`. In order to best support users, versions `0.4.1` and `0.4.2` were "yanked" and re-published as `0.5.0`.

Additionally, the team is working with the authors of `neon-serde` to remove the dependency on `neon-runtime` to prevent future issues.

## Bug Fixes

* Fix stack overflow in `DowncastError` `Display` impl (https://github.com/neon-bindings/neon/pull/606)

# Version 0.4.2

_Unpublished / Yanked_

## Bug Fixes

* Fix memory leak and race condition in `EventHandler`

# Version 0.4.1

_Unpublished / Yanked_

## Features

### Try Catch

Added the `cx.try_catch` API of [RFC 29](https://github.com/neon-bindings/rfcs/pull/29). This feature is behind the `try-catch-api` feature flag.

## Bug Fixes

* Pass `async_context` to `node::MakeCallback` (https://github.com/neon-bindings/neon/pull/498)
* Cache bust neon if node version changes (https://github.com/neon-bindings/neon/pull/388)
* Fix debug builds in windows (https://github.com/neon-bindings/neon/pull/400)
* Fix cross compiling architectures (https://github.com/neon-bindings/neon/pull/491)
* Fix neon new hanging on Windows (https://github.com/neon-bindings/neon/pull/537)

## CI Improvements

The Neon Project now uses Github Actions thanks to @lhr0909! As part of this change, CI now runs on all of our supported platforms (macOS, Windows, linux) and Node versions.

# Version ✨0.4✨ 🎉

## `EventHandler` API

The [`EventHandler` API](https://github.com/neon-bindings/rfcs/blob/main/text/0025-event-handler.md) is a new feature for scheduling work on the javascript main thread from other threads. Big thanks to @geovie for the RFC and implementation.

This feature is currently _unstable_ and gated by a `event-handler-api` feature flag.

## Improvements

* New project template updated for Rust 2018

## Bug Fixes

* Workaround for nodejs/node-gyp#1933
* Docs build fixed
* Temporarily disable static tests which keep breaking CI

## N-API

* Context/Isolate threading
* Scopes
* Strings
* Primitive values (numbers, undefined, null, boolean)

# Version 0.3.3

Hot fix for `neon build` in projects with many dependencies.

# Version 0.3.2

## Bug fixes and Small Features

* Disable node module registration on test build, allowing `cargo test` to be used on neon modules
* Added support for alternate `CARGO_TARGET_DIR` locations (e.g., workspaces)
* Added macros to `neon::prelude` to improve ergonomics in Rust 2018
* Link `win_delay_hook` when building with `electron-build-env`, fixing Windows Electron
* Fixed missing `__cxa_pure_virtual` on Linux
* Copy native files into `OUT_DIR` and build there to fix `cargo publish` and follow best practices
* Eliminated `mem::uniitialized()` usage, reducing warnings and fixing an instance of undefined behavior

## Potentially Breaking

The macOS link arguments were moved from `neon-cli` to `neon-build`. This is more idiomatic, but makes `neon-build` _required_ for macOS builds where it was unnecessary before.

Since `neon-build` has been included in the project template since `0.1` this change was not deemed significant enough to warrant a major revision.

## N-API

Neon 0.3.2 lays the groundwork for the next major revision. Development of Neon against an ABI stable Node API (N-API) will occur on main.

* Added `legacy-runtime` and `n-api` feature flags for toggling neon runtime
* Moved the legacy runtime to `nodejs-sys` crate
* Stubbed required `n-api` implementation
* Added `feature` flag to `neon-cli` to help configuring `n-api` projects

# Version 0.3.1

* Build v0.3 project templates by default in the CLI

# Version 0.3

## Breaking Changes

* [Removed support for Node 6](https://github.com/neon-bindings/neon/pull/420)

## Bug Fixes

* Correctly fail the build if [custom build command fails](https://github.com/neon-bindings/neon/pull/421)
* Fix breaking changes with v8 [`GetFunction`](https://github.com/neon-bindings/neon/pull/410)
* Moved `nan` from `devDependencies` to `dependencies` in [`neon-runtime`](https://github.com/neon-bindings/neon/pull/367)
* Changed neon [crate type](https://github.com/neon-bindings/neon/pull/358) from `dylib` to `cdylib`
* Ensure that neon module loading is [not optimized away](https://github.com/neon-bindings/neon/pull/392)

## Improvements

* Added support for [`CARGO_BUILD_TARGET` environment variable](https://github.com/neon-bindings/neon/pull/411)

# Version ✨0.2✨ 🎉

See the [Neon 0.2 Migration Guide](https://github.com/neon-bindings/neon/wiki/Neon-0.2-Migration-Guide) for documentation on migrating your projects from the Neon 0.1.x series to Neon 0.2, and please [let us know](https://github.com/neon-bindings/neon#get-involved) if you need help!

* Release automation (#318)
* New `ArrayBuffer` views API -- see [RFC 5](https://github.com/neon-bindings/rfcs/blob/main/text/0005-array-buffer-views.md) (#306)
* VM 2.0 -- see [RFC 14](https://github.com/neon-bindings/rfcs/blob/main/text/0014-vm-2.0.md) (#306)
* New `JsString` constructor -- see [RFC 21](https://github.com/neon-bindings/rfcs/blob/main/text/0021-string-constructor.md) (#322)
* Eliminated `JsInteger`, `JsVariant`, `callee()` -- see [RFC 22](https://github.com/neon-bindings/rfcs/blob/main/text/0022-zero-dot-two.md) (#323)
* Renamed `Key` to `PropertyKey` and its method names -- see [RFC 22](https://github.com/neon-bindings/rfcs/blob/main/text/0022-zero-dot-two.md) (#323)
* Module reorganization -- see [RFC 20](https://github.com/neon-bindings/rfcs/blob/main/text/0020-module-reorg.md) (#324)
* New `JsError` API -- see [RFC 23](https://github.com/neon-bindings/rfcs/blob/main/text/0023-error-subtyping.md) (#325)
* Eliminated `ToJsString` API -- see [RFC 22](https://github.com/neon-bindings/rfcs/blob/main/text/0022-zero-dot-two.md) (#326)
* Eliminated `NEON_NODE_ABI` env var -- see [RFC 22](https://github.com/neon-bindings/rfcs/blob/main/text/0022-zero-dot-two.md) (#327)
* Default to release builds -- see [RFC 22](https://github.com/neon-bindings/rfcs/blob/main/text/0022-zero-dot-two.md) (#328)
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
