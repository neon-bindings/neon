---
title: Build a SQLite addon
description: Wrap a SQLite connection pool as a JavaScript class with `#[neon::class]`, then expose async methods backed by Tokio so queries return Promises and never block the event loop.
---

The previous tutorial moved a [CPU-bound](/tutorials/move-work-off-the-main-thread/)
function onto Node's worker pool. This one moves *I/O-bound* work —
database queries — onto a real async runtime, and introduces three
new Neon features along the way:

- [`#[neon::class]`](/api/neon/attr.class.html), which exposes a Rust
  struct as a JavaScript class with methods,
- [`async fn`](https://doc.rust-lang.org/std/keyword.async.html)
  exports, which run on a [tokio](https://docs.rs/tokio/latest/tokio/)
  runtime and resolve a [`Promise`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise)
  with their result,
- [`#[neon::main]`](/api/neon/attr.main.html), for one-time addon
  setup at module load.

We'll build a SQLite-backed addon — opening a database, running
queries, committing transactions — using
[sqlx](https://docs.rs/sqlx/latest/sqlx/), the popular async SQL
toolkit. SQLite needs no infrastructure, so the whole tutorial runs
on your laptop in a few seconds.

## What we're building

By the end you'll have an addon with a `Database` class:

```js
const db = await Database.connect(":memory:");
await db.execute("INSERT INTO users (name) VALUES ('alice'), ('bob')");
const rows = await db.query("SELECT id, name FROM users ORDER BY id");
console.log(rows); // [{ id: 1, name: 'alice' }, { id: 2, name: 'bob' }]
```

We'll get there in five steps:

1. Wrap a SQLite connection pool as a class with a synchronous constructor.
2. Add a `query` method that returns shaped JavaScript objects.
3. Add a `transfer` method that runs a multi-statement transaction.
4. Promote schema setup into Rust by replacing the synchronous
   constructor with an async [`connect`](https://docs.rs/sqlx/latest/sqlx/struct.Pool.html#method.connect)
   factory function.
5. Hang `connect` off the class as a static method, using
   `#[neon::main]` to wire it up at module load.

## Dependencies

Add [tokio](https://docs.rs/tokio/latest/tokio/) and
[sqlx](https://docs.rs/sqlx/latest/sqlx/) to your `Cargo.toml`, and
turn on Neon's `tokio` feature so a runtime is registered automatically
when the addon loads:

```toml
[dependencies]
neon = { version = "1", features = ["tokio", "serde"] }
serde_json = "1"
sqlx = { version = "0.8", default-features = false, features = [
    "runtime-tokio",
    "sqlite",
] }
```

The `serde` feature on Neon lights up
[`Json<T>`](/api/neon/types/extract/struct.Json.html), which we'll
use in step 2 to return result rows as plain JS objects.

The `tokio` feature is shorthand for `tokio-rt-multi-thread` and asks
Neon to spin up a multi-threaded tokio runtime at addon load. Every
`async fn` we export from now on runs on that runtime. If you ever
need to bring your own runtime, see the
[*Export async functions*](/how-to/async-fn/) how-to for
[`set_global_executor`](/api/neon/fn.set_global_executor.html).

## Step 1 — A class with a connection pool

We'll model the database as a Rust struct holding a sqlx
[`SqlitePool`](https://docs.rs/sqlx/latest/sqlx/sqlite/type.SqlitePool.html).
A pool is cheap to clone — internally it's just an `Arc` over the
real connection state — which matters because Neon will clone our
struct into every `async fn` call. In `src/lib.rs`:

```rust
use neon::types::extract::Error;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

#[derive(Clone)]
struct Database {
    pool: SqlitePool,
}

#[neon::export(class)]
impl Database {
    pub fn new(path: String) -> Result<Self, Error> {
        let opts = SqliteConnectOptions::from_str(&format!("sqlite:{path}"))?
            .create_if_missing(true);
        let pool = SqlitePool::connect_lazy_with(opts);
        Ok(Self { pool })
    }

    async fn execute(self, sql: String) -> Result<f64, Error> {
        let result = sqlx::query(&sql).execute(&self.pool).await?;
        Ok(result.rows_affected() as f64)
    }
}
```

A few things to call out:

- **`#[neon::export(class)]`** does two things: it makes the `impl`
  block describe a JavaScript class (so `new Database(...)` works on
  the JS side and methods become methods on the prototype), and it
  also adds `Database` to the addon's exports — so
  `require("./index.node").Database` resolves to the class.
- **The `new` method becomes the JS constructor.** It can return
  `Self` or `Result<Self, E>`; an `Err` becomes a thrown JavaScript
  exception. We return `Result` here because parsing the SQLite URL
  can fail. [`SqlitePool::connect_lazy_with`](https://docs.rs/sqlx/latest/sqlx/struct.Pool.html#method.connect_lazy_with)
  itself doesn't fail — it defers actually opening the database until
  the first query.
- **Async methods take `self` by value**, not `&self`. They also
  require `Database: Clone` (which is why we derived it). The reason:
  every async method's body becomes a `'static` future that may
  outlive the JS call, so it can't borrow from the instance — it has
  to own a clone of it. Sqlx pools are `Arc`-backed, so cloning is a
  cheap refcount bump.
- **`Result<T, Error>`** with the
  [`extract::Error`](/api/neon/types/extract/struct.Error.html) type
  lets `?` convert almost any error type into a JS exception. Sqlx's
  errors implement [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html),
  so they bridge automatically. See the
  [*Throw and catch JavaScript errors from Rust*](/how-to/errors/)
  how-to for the full story.
- **The return type is `f64`, not `u64`.** JavaScript numbers are
  IEEE 754 doubles, so Neon converts the eight integer types up to
  32 bits and the two float types directly. For wider integers like
  `rows_affected`'s `u64`, an `as f64` cast is the standard idiom —
  exact up to 2⁵³, which covers any plausible row count. If you need
  the full 64 bits, return a JS
  [`BigInt`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt)
  via [`JsBigInt`](/api/neon/types/struct.JsBigInt.html).

That's enough to drive the addon from JavaScript. Save this as
`example.cjs`:

```js
const { Database } = require("./index.node");

(async () => {
  const db = new Database(":memory:");
  await db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)");
  const inserted = await db.execute(
    "INSERT INTO users (name) VALUES ('alice'), ('bob')",
  );
  console.log(`inserted ${inserted} rows`); // => "inserted 2 rows"
})();
```

Build with `npm run build` and run with `node example.cjs`. The
two `await`s are not cosmetic: while sqlx is waiting on SQLite, the
JavaScript main thread is free to do anything else.

## Step 2 — Returning rows as JavaScript objects

`execute` returns a count. Most of the time you actually want the
rows back. Add a `query` method:

```rust
# use neon::types::extract::Error;
# use sqlx::sqlite::SqlitePool;
use neon::types::extract::Json;
use serde_json::{Map, Value};
use sqlx::{Column, Row, TypeInfo};

# #[derive(Clone)]
# struct Database { pool: SqlitePool }
# #[neon::export(class)]
# impl Database {
#   pub fn new(_path: String) -> Result<Self, Error> { unimplemented!() }
async fn query(self, sql: String) -> Result<Json<Vec<Value>>, Error> {
    let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;

    let result = rows
        .iter()
        .map(|row| {
            let mut obj = Map::new();
            for (i, col) in row.columns().iter().enumerate() {
                let v = match col.type_info().name() {
                    "INTEGER" => row.try_get::<Option<i64>, _>(i)?.into(),
                    "REAL" => row.try_get::<Option<f64>, _>(i)?.into(),
                    "TEXT" => row.try_get::<Option<String>, _>(i)?.into(),
                    _ => Value::Null,
                };
                obj.insert(col.name().to_string(), v);
            }
            Ok(Value::Object(obj))
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(result))
}
# }
```

What's new:

- **[`Json<T>`](/api/neon/types/extract/struct.Json.html)** wraps any
  `serde::Serialize` type and converts it to a JS value at the
  boundary. Returning `Json<Vec<Value>>` produces a JS array of
  objects — the natural shape for a result set. The
  [*Use serde for structured data*](/how-to/serde-json/) how-to has
  the wider story; here it just lets us hand back column-keyed rows
  without writing any conversion code by hand.
- **The body is plain async Rust.** `sqlx::query(&sql).fetch_all(&self.pool).await`
  acquires a connection, sends the SQL, and reads rows — all without
  blocking the event loop. The `.collect::<Result<_, _>>()?` line
  short-circuits with the first row that fails to convert.
- **We only handle three SQLite types** — `INTEGER`, `REAL`, `TEXT` —
  and fall through to `Null` for anything else. That keeps the
  example focused; a real addon would map `BLOB` to a buffer, dates
  to ISO strings, and so on.

Now in JavaScript:

```js
const { Database } = require("./index.node");

(async () => {
  const db = new Database(":memory:");
  await db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)");
  await db.execute("INSERT INTO users (name) VALUES ('alice'), ('bob')");

  const rows = await db.query("SELECT id, name FROM users ORDER BY id");
  console.log(rows);
  // => [ { id: 1, name: 'alice' }, { id: 2, name: 'bob' } ]
})();
```

## Step 3 — Transactions

Real database code rarely runs single statements in isolation. The
canonical example is a money transfer: debit one account, credit
another, all-or-nothing. SQLite makes this easy with
[transactions](https://www.sqlite.org/lang_transaction.html), and
sqlx exposes them through [`Pool::begin`](https://docs.rs/sqlx/latest/sqlx/struct.Pool.html#method.begin):

```rust
# use neon::types::extract::Error;
# use sqlx::sqlite::SqlitePool;
# #[derive(Clone)]
# struct Database { pool: SqlitePool }
# #[neon::export(class)]
# impl Database {
#   pub fn new(_path: String) -> Result<Self, Error> { unimplemented!() }
async fn transfer(
    self,
    from: f64,
    to: f64,
    amount: f64,
) -> Result<(), Error> {
    let mut tx = self.pool.begin().await?;

    sqlx::query("UPDATE accounts SET balance = balance - ? WHERE id = ?")
        .bind(amount as i64)
        .bind(from as i64)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE accounts SET balance = balance + ? WHERE id = ?")
        .bind(amount as i64)
        .bind(to as i64)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}
# }
```

A few notes:

- **Multiple `.await`s, single async function.** Each one yields back
  to the event loop; in between, your JS process can answer HTTP
  requests, fire timers, run other queries on this same `Database`.
  This is the entire reason `async fn` exists in Neon.
- **`tx.commit().await`** is the only thing that actually durably
  applies the changes. If `transfer` returns early — because either
  `UPDATE` fails, or because the `?` operator propagates an error —
  the transaction is dropped without committing, and SQLite rolls it
  back automatically.
- **`?` parameters and `.bind`** are sqlx's parameterised query API,
  the right way to pass user input into SQL. We didn't use them in
  earlier examples because the SQL was hard-coded; for anything
  user-controlled, always bind.

From JavaScript:

```js
const { Database } = require("./index.node");

(async () => {
  const db = new Database(":memory:");
  await db.execute(`
    CREATE TABLE accounts (id INTEGER PRIMARY KEY, balance INTEGER);
  `);
  await db.execute("INSERT INTO accounts VALUES (1, 100), (2, 0)");

  await db.transfer(1, 2, 30);

  console.log(await db.query("SELECT * FROM accounts ORDER BY id"));
  // => [ { id: 1, balance: 70 }, { id: 2, balance: 30 } ]
})();
```

## Step 4 — Move schema setup into Rust

The class is useful, but every JS caller still has to remember to
issue the `CREATE TABLE` statements. That's a problem: if we add a
column or a new table in a later version of the addon, *every* caller
has to be updated in lockstep.

The standard fix is to push schema setup into the addon itself. Open
the database, run the migration, hand back a ready-to-use instance.
But there's a snag: opening a real connection is async (the database
file might not exist, or might need its journal recovered), and
[constructors can't be async](https://github.com/rust-lang/rfcs/issues/2906)
in Rust **or** in JavaScript. Trying to mark `new` as `async fn` in a
`#[neon::class]` impl won't compile:

```text
error: Async functions in classes must take `self` by value, not `&self` or `&mut self`.
       This is required because async functions capture `self` in the
       Future, which must be `'static` for spawning.
```

The idiom Rust uses for this is to make `new` synchronous (or remove
it) and provide a free `connect` function that returns a fully
initialised instance. Returning a `Database` from a Neon-exported
function automatically constructs a JS class instance for it, so the
JS-side experience is identical to `new` — except this time it
returns a [`Promise`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise).

Replace the body of `new` with an error pointing at the new function,
and add a free `connect` next to the `impl`:

```rust
# use neon::types::extract::Error;
# use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
# use std::str::FromStr;
# #[derive(Clone)]
# struct Database { pool: SqlitePool }
# #[neon::export(class)]
# impl Database {
pub fn new(_path: String) -> Result<Self, Error> {
    Err(Error::new(
        "Database cannot be constructed directly; use addon.connect(path) instead.",
    ))
}
# async fn execute(self, _sql: String) -> Result<f64, Error> { Ok(0.0) }
# }

#[neon::export]
async fn connect(path: String) -> Result<Database, Error> {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite:{path}"))?
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id   INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    Ok(Database { pool })
}
```

What this is doing:

- **`#[neon::export] async fn connect`** is a free function — same
  shape as the other async exports we've written, no class
  involvement on the Rust side.
- **The return type is `Result<Database, Error>`.** Because
  `#[neon::class]` automatically implements
  [`TryIntoJs`](/api/neon/types/extract/trait.TryIntoJs.html) for the
  struct, returning a `Database` produces a real JS class instance —
  same prototype, same methods. From JS, the value is
  indistinguishable from `new Database(...)`, except that the
  instance is fully initialised before it's returned.
- **The `new` body now always errors.** This isn't ideal — a
  determined caller can still write `new Database(":memory:")` and
  get an exception — but it's the best we can do until Rust grows
  async constructors. The error message points at the right
  replacement.

The full JavaScript flow now looks like:

```js
const addon = require("./index.node");

(async () => {
  const db = await addon.connect(":memory:");
  await db.execute("INSERT INTO users (name) VALUES ('alice')");

  console.log(await db.query("SELECT * FROM users"));
  // => [ { id: 1, name: 'alice' } ]
})();
```

The `users` table is already there — `connect` made sure of it.

## Step 5 — Make `connect` a static method

The addon works, but the JS API is asymmetric: instances use
`db.method()`, while the factory is at the addon root —
`addon.connect(...)`. JavaScript developers expect the factory to
look like a static method on the class itself: `Database.connect(...)`.
We can wire that up at module load by attaching `connect` to the
class constructor as a property.

To do that we need two new pieces of Neon:

- [`#[neon::main]`](/api/neon/attr.main.html), which marks a function
  to run once when the addon is loaded — a place to do setup that
  isn't a single `#[neon::export]`.
- [`Database::constructor(cx)`](/api/neon/object/trait.Class.html#tymethod.constructor),
  which returns a handle to the JavaScript constructor function that
  `#[neon::class]` generated. From the JavaScript side this is the
  same function that `require("./index.node").Database` resolves to;
  from the Rust side it's a [`Handle<JsFunction>`](/api/neon/types/struct.JsFunction.html)
  we can hang properties off of.

There's also a wrinkle to deal with: providing your own
`#[neon::main]` replaces Neon's default startup logic, including the
auto-init of the tokio runtime that the [`tokio` feature](https://docs.rs/neon/latest/neon/#features)
gives you. We'll register a runtime ourselves with
[`set_global_executor`](/api/neon/fn.set_global_executor.html) to put
that back.

Add an `init` function at the bottom of `src/lib.rs`. Despite the
attribute name, the function itself can be called anything — Neon
just uses the attribute to find it.

```rust
# use neon::context::ModuleContext;
# use neon::handle::Handle;
# use neon::result::NeonResult;
# use neon::object::{Class, Object};
# use neon::types::JsFunction;
# use sqlx::sqlite::SqlitePool;
# use std::sync::OnceLock;
# #[derive(Clone)]
# struct Database { pool: SqlitePool }
# #[neon::export(class)]
# impl Database {
#   pub fn new(_path: String) -> Result<Self, neon::types::extract::Error> {
#       unimplemented!()
#   }
# }
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

#[neon::main]
fn init(mut cx: ModuleContext) -> NeonResult<()> {
    let rt = RUNTIME.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("failed to start tokio runtime")
    });
    let _ = neon::set_global_executor(&mut cx, rt);

    neon::registered().export(&mut cx)?;

    let class = Database::constructor(&mut cx)?;
    let exports = cx.exports_object()?;
    let connect: Handle<JsFunction> = exports.prop(&mut cx, "connect").get()?;
    class.prop(&mut cx, "connect").set(connect)?;

    Ok(())
}
```

What this does:

- **`#[neon::main]`** registers a function to run when Node loads the
  addon. There can be at most one in a crate, and providing one
  replaces Neon's default startup logic — so we have to do everything
  Neon would normally do for us by hand.
- **The `RUNTIME` static + `set_global_executor`** registers a tokio
  runtime ourselves. The default `main` did this automatically when
  the `tokio` feature was enabled; once we override it, we have to
  bring our own. The
  [`OnceLock`](https://doc.rust-lang.org/std/sync/struct.OnceLock.html)
  makes the runtime survive across calls. See the
  [*Export async functions*](/how-to/async-fn/) how-to for variations
  on this pattern.
- **`neon::registered().export(&mut cx)?`** publishes every
  `#[neon::export]` we wrote. The default `main` did this for us;
  we have to call it explicitly now.
- **`Database::constructor(cx)?`** hands back the JS constructor
  function as a [`Handle<JsFunction>`](/api/neon/types/struct.JsFunction.html).
  This is the same value JS sees as `require("./index.node").Database`.
- **`cx.exports_object()?.prop(...)`** reads the `connect` property
  off the addon's exports object — the one
  `#[neon::export] async fn connect` populated a moment earlier. We
  then write it back onto the class. After this, `Database.connect`
  and `addon.connect` are the same function.

You'll also want `tokio` as a direct dependency now that you're
referring to its runtime by name:

```toml
[dependencies]
tokio = { version = "1", features = ["rt-multi-thread"] }
```

JavaScript users now have a clean, symmetric API:

```js
const { Database } = require("./index.node");

(async () => {
  const db = await Database.connect(":memory:");
  await db.execute("INSERT INTO users (name) VALUES ('alice')");

  console.log(await db.query("SELECT * FROM users"));
  // => [ { id: 1, name: 'alice' } ]
})();
```

`new Database(...)` still throws (Step 4's leftover guard), and
`Database.connect(...)` is the canonical entry point.

## What you've learned

- **`#[neon::class]`** wraps a Rust `impl` block and exposes the
  struct as a JavaScript class. Methods become methods; `new` becomes
  the constructor.
- **`async fn` methods** take `self` by value and require the struct
  to be `Clone`. They run on the tokio runtime registered by the
  `tokio` feature flag and resolve a JS `Promise` with their result.
- **Returning a class type from a Neon-exported function** constructs
  a JS instance for you, which is the standard idiom for "async
  constructor".
- **`#[neon::main]`** runs once at addon load. It's the place to wire
  up things that don't fit a single `#[neon::export]`, like attaching
  a free function to a class as a static method.

## Where next

- [Export async functions](/how-to/async-fn/) — the no-narrative
  recipe for `async fn` exports, including how to bring your own
  runtime with [`set_global_executor`](/api/neon/fn.set_global_executor.html).
- [Expose a Rust struct as a JavaScript class](/how-to/classes/) —
  more on `#[neon::class]`: methods that take `&mut self`, returning
  `Result` from the constructor, finalisers.
- [Use serde for structured data](/how-to/serde-json/) — the wider
  story for [`Json<T>`](/api/neon/types/extract/struct.Json.html)
  arguments and return values.
- [Throw and catch JavaScript errors from Rust](/how-to/errors/) —
  how [`extract::Error`](/api/neon/types/extract/struct.Error.html),
  `?`, and rejected Promises fit together.
