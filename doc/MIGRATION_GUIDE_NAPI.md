# N-API Migration Guide

## What is this about?

Since v10, Node.js supports an improved API for building native modules, known as [N-API](https://nodejs.org/api/n-api.html). N-API offers a clearer, more complete, and more stable API layer for writing Node.js plugins than previous Node versions did.

The Neon community has been [hard at work](https://github.com/neon-bindings/neon/issues/444) porting the library to a new backend based on N-API.

### Why N-API?

Some key benefits of the new backend include:
- Compiled Neon modules will work in all versions of Node _without needing to be recompiled_, guaranteed!
- You can precompile Neon-based libraries to be completely transparent to downstream consumers.
- The build process is streamlined, making Neon apps more reliable and easier to debug.
- The stability guarantees of N-API allow us to avoid risk of incompatible changes to future releases of Neon.

### What does this mean for me?

Porting Neon to N-API has been mostly transparent, but it has required a few backwards-incompatible changes. This guide provides instructions on how to migrate existing apps to the new N-API backend.

Fortunately, the guaranteed stability of N-API means that once Neon users do this migration, we have increased confidence in the stability of Neon. We expect this to be the **last major breaking change before reaching Neon 1.0.**

If you have any trouble porting, **please reach out to us** with a Neon issue or on the community Slack! We want to help everyone upgrade as smoothly and seamlessly as possible.


## Getting started

### Supported Node versions

The N-API backend of Neon requires a minimum Node version of 10.0.

### Enabling the N-API backend

To enable the N-API backend, you need to:

1. Remove `build.rs` from the project directory and `build = "build.rs"` from the `Cargo.toml`. The N-API backend does not require a Cargo build script.
2. Disable the default features (for now, the default features select the legacy backend) by setting `default-features = false`; and
3. Enable the appropriate feature flag in your `Cargo.toml` to select the N-API version you need support for (each N-API version N uses the feature flag `"napi-N"`, for example `"napi-4"` for N-API version 4).

As a rule, you should choose the **oldest version of N-API that has the APIs you need.** (We will be adding N-API version requirements to the Neon API docs to make this clearer in the future.) You can consult the [official N-API feature matrix](https://nodejs.org/api/n-api.html#n_api_node_api_version_matrix) to see which N-API versions come with various versions of Node.

```toml
[dependencies.neon]
version = "0.9.1"
default-features = false
features = ["napi-4"]
```


## Minor API changes

### Context

Many methods that previously did not require context (e.g., `JsString::size`) now require a context. In many cases, this means adding an additional argument or using a convenience method on the `Context` trait.

#### Affected methods

##### Handle

* `Handle`
    - `is_a`
    - `downcast`

`Handle::downcast` also requires a second type argument for the context type. This can usually be inferred, so you can typically use `_`.

**Before:**
```rust
value.downcast::<JsNumber>()
```

**After:**

```rust
value.downcast::<JsNumber, _>(&mut cx)
```

##### Primitive types

* `JsBoolean`
    - `value`
* `JsNull`
    - `new`
* `JsString`
    - `size`
    - `value`
* `JsNumber`
    - `value`
* `JsUndefined`
    - `new`

##### Object APIs

* `PropertyKey`
    - `get_from`
    - `set_from`

### Handle equality

Handles no longer implement `Eq` or `PartialEq`, which had underspecified behavior. Use `Value::strict_equals` instead to invoke the behavior of JavaScript's `===` operator.

## Major API changes

The N-API backend introduces two categories of significant change:

1. Embedding Rust data, which is no longer done through the awkward and complex `declare_types!` (i.e. classes) macro, but through a simpler primitive: the `JsBox` API.
2. Concurrency, which is offered through the Event Queue API instead of the Task API or Event Handlers, both of which are deprecated and removed in the N-API backend.

### Embedding Rust data

The `declare_types!` macro is deprecated and replaced by the `JsBox` type.

_Rationale:_ The `declare_types!` macro provides a syntax for defining classes, but requires substantial boilerplate and is unergonomic for simple cases and tends to interact poorly with IDEs. It's also not flexible enough to express the full range of JavaScript classes syntax and semantics. With the `JsBox` type, it's easy to embed Rust data in JavaScript objects, which can then be nested inside of more feature-rich classes defined in pure JavaScript (or TypeScript).

**Before:**

```rust
struct User {
    first_name: String,
    last_name: String,
}

impl User {
    fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

declare_types! {
    class JsUser for User {
        init(mut cx) {
            let first_name = cx.argument::<JsString>(0)?;
            let last_name = cx.argument::<JsString>(1)?;
            Ok(User { first_name, last_name })
        }

        method full_name(mut cx) {
            let this = cx.this();
            let guard = cx.lock();
            let user = this.borrow(&guard);
            let full_name = user.full_name();
            Ok(cx.string(full_name).upcast())
        }
    }
}
```

**After:**

On the Rust side, the wrapped type must implement the `Finalize` trait, but this comes with a default implementation so it can be implemented with an empty `impl` block:

```rust
struct User {
    first_name: String,
    last_name: String,
}

impl Finalize for User { }
```

The type can then be exposed to JavaScript with simple functions that wrap `User` in a `JsBox`:

```rust
fn create_user(mut cx: FunctionContext) -> JsResult<JsBox<User>> {
    let first_name = cx.argument::<JsString>(0)?.value(&mut cx);
    let last_name = cx.argument::<JsString>(1)?.value(&mut cx);
    Ok(cx.boxed(User { first_name, last_name }))
}

fn user_full_name(mut cx: FunctionContext) -> JsResult<JsString> {
    let user = cx.argument::<JsBox<User>>(0)?;
    let full_name = user.full_name();
    Ok(cx.string(full_name))
}
```

Finally, you can provide an idiomatic JavaScript interface to the type by wrapping the boxed type in a class:

```js
class User {
    constructor(firstName, lastName) {
        this.boxed = addon.createUser(firstName, lastName);
    }

    fullName() {
        return addon.userFullName(this.boxed);
    }
}
```

### Concurrency

The supported mechanism for concurrency is the Channel API (`neon::event::Channel`). This feature has not yet stabilized, so to use this API, you'll also need to enable the `"channel-api"` feature flag as well:

```toml
[dependencies.neon]
version = "0.9.1"
default-features = false
features = ["napi-6", "channel-api"]
```

#### Deprecated: Task API

The Task API (`neon::task`) is deprecated, and should in most cases be translated to using the Event Queue API.

_Rationale:_ The Task API was built on top of the low-level libuv thread pool, which manages the concurrency of the Node.js system internals and should rarely be exposed to user-level programs. For most use cases, Neon users took advantage of this API as the only way to implement background, asynchronous computations. The Event Queue API is a more general-purpose, convenient, and safe way of achieving that purpose.

That said, **if you believe you need access to the libuv thread pool, please [file an issue in the Neon repository](https://github.com/neon-bindings/neon/issues) with a description of your use case to let us know about it.** We don't believe this is commonly needed, but we don't want to leave you stuck! 

**Before:**

With the `Task` API it was possible to define background computations off the main JavaScript thread, but these could only be run within the libuv thread pool--which runs all the system logic for the internals of Node.js. This gave Neon programmers a real power but forced them to contend with Node.js system tasks.

```rust
impl Task for MyTask {
    type Output = i32;
    type Error = String;
    type JsEvent = JsNumber;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        // compute the result...
    }

    fn complete(self, mut cx: TaskContext, result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        match result {
            Ok(n) => {
                Ok(cx.number(n))
            }
            Err(s) => {
                cx.throw_error(s)
            }
        }
    }
}

pub fn start_task(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let callback = cx.argument::<JsFunction>(0)?;
    MyTask.schedule(callback);
    Ok(cx.undefined())
}
```

**After:**

With the N-API backend, Neon programmers can use their own native threads and avoid competing with the Node.js system internals. This also brings some convenience since it doesn't require defining any custom trait implementations.

```rust
pub fn start_task(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);
    let queue = cx.queue();

    std::thread::spawn(move || {
        let result = // compute the result...
        queue.send(move |mut cx| {
            let callback = callback.into_inner(&mut cx);
            let this = cx.undefined();
            let args = match result {
                Ok(n) => vec![
                    cx.null().upcast::<JsValue>(),
                    cx.number(result).upcast()
                ],
                Err(msg) => vec![
                    cx.error(msg).upcast()
                ]
            };
            callback.call(&mut cx, this, args)?;
            Ok(())
        });
    });

    Ok(cx.undefined())
}
```

#### Deprecated: Event Handler API

The Event Handler API (`neon::event::EventHandler`) is deprecated and should be replaced by the Event Queue API.

_Rationale_: The Event Handler API had multiple issues with safety, memory leaks, and ergonomics ([1](https://github.com/neon-bindings/neon/issues/551), [2](https://github.com/neon-bindings/rfcs/issues/31)).

**Before:**

```rust
pub fn start_task(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let callback = cx.argument::<JsFunction>(0)?;
    let handler = EventHandler::new(callback);
    thread::spawn(move || {
        let result = // compute the result...
        handler.schedule(move |cx| {
            vec![cx.number(result).upcast()]
        });
    });
    Ok(cx.undefined())
}
```

**After:**

```rust
pub fn start_task(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);
    let queue = cx.queue();

    std::thread::spawn(move || {
        let result = // compute the result...
        queue.send(move |mut cx| {
            let callback = callback.into_inner(&mut cx);
            let this = cx.undefined();
            let args = vec![
                cx.null().upcast::<JsValue>(),
                cx.number(result).upcast()
            ];
            callback.call(&mut cx, this, args)?;
            Ok(())
        });
    });

    Ok(cx.undefined())
}
```

