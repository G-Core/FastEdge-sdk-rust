# FastEdge Rust SDK — Host Services

Host-provided service modules for key-value storage, secret management, configuration dictionaries, diagnostic utilities, and customer-owned databases.

The host-service modules documented here (`fastedge::key_value`, `fastedge::secret`, `fastedge::dictionary`, `fastedge::utils`, `fastedge::db`) are part of the standard `fastedge` crate. No additional Cargo.toml changes are needed beyond the standard `fastedge` dependency to use these services. The only optional addition is the `json` feature, which enables the typed query helpers in `fastedge::db` (see [Databases](#databases)).

---

## Key-Value Storage

The `fastedge::key_value` module provides persistent storage with support for simple key-value pairs, glob-style key scanning, sorted sets, and bloom filters. Data is organized into named stores; access to a store must be granted via platform configuration.

### Opening a Store

```rust
pub fn new() -> Result<Self, Error>
pub fn open(name: &str) -> Result<Self, Error>
```

`Store::new()` opens the default store. `Store::open(name)` opens a named store. Both return `Err(Error::NoSuchStore)` if the store label is not recognized and `Err(Error::AccessDenied)` if the application is not authorized.

```rust,no_run
use fastedge::key_value::Store;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    // Open the default store
    let _default_store = Store::new()?;

    // Open a named store
    let _named_store = Store::open("user-data")?;

    Ok(Response::builder()
        .status(200)
        .body(Body::from("ok"))?)
}
```

### Reading Values

```rust
pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>, Error>
```

Returns `Ok(Some(bytes))` if the key exists, `Ok(None)` if it does not.

```rust,no_run
use fastedge::key_value::Store;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let store = Store::open("user-data")?;

    match store.get("user:123:profile")? {
        Some(data) => Ok(Response::builder()
            .status(200)
            .body(Body::from(data))?),
        None => Ok(Response::builder()
            .status(404)
            .body(Body::from("not found"))?),
    }
}
```

### Pattern Scanning

```rust
pub fn scan(&self, pattern: &str) -> Result<Vec<String>, Error>
```

Scans the store for keys matching a glob-style pattern. Returns a list of matching key names. Returns an empty `Vec` if no keys match.

Supported glob syntax:

| Pattern | Matches                                     |
| ------- | ------------------------------------------- |
| `*`     | Any sequence of characters within a segment |
| `?`     | Any single character                        |
| `[abc]` | Any character in the set                    |

```rust,no_run
use fastedge::key_value::Store;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let store = Store::open("user-data")?;

    let keys = store.scan("user:123:*")?;
    for key in &keys {
        println!("Found key: {}", key);
    }

    Ok(Response::builder()
        .status(200)
        .body(Body::from(format!("{} keys found", keys.len())))?)
}
```

### Sorted Sets

Sorted sets store members associated with a `f64` score. Members are ordered from lowest to highest score.

```rust
pub fn zrange_by_score(&self, key: &str, min: f64, max: f64) -> Result<Vec<(Vec<u8>, f64)>, Error>
pub fn zscan(&self, key: &str, pattern: &str) -> Result<Vec<(Vec<u8>, f64)>, Error>
```

`zrange_by_score` returns all members of the sorted set stored at `key` whose score falls in the inclusive range `[min, max]`. Use `f64::NEG_INFINITY` and `f64::INFINITY` for unbounded ranges.

`zscan` returns members of the sorted set at `key` whose member value matches the glob-style `pattern`.

Both return an empty `Vec` when the key does not exist or no members fall within the specified range or pattern.

```rust,no_run
use fastedge::key_value::Store;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let store = Store::open("game-data")?;

    // Retrieve all leaderboard entries with scores >= 1000
    let top_players = store.zrange_by_score("leaderboard", 1000.0, f64::INFINITY)?;
    for (member, score) in &top_players {
        let name = String::from_utf8_lossy(member);
        println!("Player: {}, Score: {}", name, score);
    }

    // Retrieve sorted set members matching a pattern
    let guild_members = store.zscan("guild:42:members", "player:*")?;
    println!("Guild members found: {}", guild_members.len());

    Ok(Response::builder()
        .status(200)
        .body(Body::from(format!("{} top players", top_players.len())))?)
}
```

### Bloom Filters

```rust
pub fn bf_exists(&self, key: &str, item: &str) -> Result<bool, Error>
```

Tests whether `item` is a member of the bloom filter stored at `key`. Returns `true` if the item was probably added to the filter (subject to the false-positive rate of the filter), or `false` if the key does not exist or the item was definitely not added.

Bloom filters cannot produce false negatives: if `bf_exists` returns `false`, the item has not been added.

```rust,no_run
use fastedge::key_value::Store;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let store = Store::open("rate-limit")?;

    let client_ip = "203.0.113.42";

    if store.bf_exists("blocked_ips", client_ip)? {
        return Ok(Response::builder()
            .status(403)
            .body(Body::from("Blocked"))?);
    }

    Ok(Response::builder()
        .status(200)
        .body(Body::empty())?)
}
```

### Error Handling

All `Store` methods return `Result<_, Error>`. The `Error` type has three variants:

| Variant                | Description                                                     |
| ---------------------- | --------------------------------------------------------------- |
| `Error::NoSuchStore`   | The requested store label is not recognized by the host         |
| `Error::AccessDenied`  | The application does not have permission to access the store    |
| `Error::Other(String)` | An implementation-specific error (I/O or internal host failure) |

```rust,no_run
use fastedge::key_value::{Error, Store};
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let store = match Store::open("config") {
        Ok(s) => s,
        Err(Error::NoSuchStore) => {
            return Ok(Response::builder()
                .status(500)
                .body(Body::from("Store not configured"))?);
        }
        Err(Error::AccessDenied) => {
            return Ok(Response::builder()
                .status(403)
                .body(Body::from("Access denied"))?);
        }
        Err(Error::Other(msg)) => {
            return Err(anyhow::anyhow!("KV store error: {}", msg));
        }
    };

    let _ = store.get("key")?;

    Ok(Response::builder()
        .status(200)
        .body(Body::empty())?)
}
```

CDN apps use `fastedge::proxywasm::key_value` instead — see [CDN_APPS.md](CDN_APPS.md) for the ProxyWasm API surface and usage examples.

---

## Secret Management

The `fastedge::secret` module provides access to encrypted secrets such as API keys, passwords, and certificates. Secrets are encrypted at rest and support versioned retrieval for rotation scenarios.

### Reading Secrets

```rust
pub fn get(key: &str) -> Result<Option<Vec<u8>>, Error>
```

Returns the currently effective value of the named secret. Returns `Ok(None)` if no secret with that name is configured.

```rust,no_run
use fastedge::secret;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let api_key = match secret::get("UPSTREAM_API_KEY")? {
        Some(key) => key,
        None => {
            return Ok(Response::builder()
                .status(500)
                .body(Body::from("API key not configured"))?);
        }
    };

    // Use api_key bytes for authentication — do not log or include in responses
    let _ = api_key;

    Ok(Response::builder()
        .status(200)
        .body(Body::empty())?)
}
```

### Time-Based Retrieval

```rust
pub fn get_effective_at(key: &str, at: u32) -> Result<Option<Vec<u8>>, Error>
```

Returns the value of the named secret that was effective at the given Unix timestamp (`at`, seconds since epoch). This is useful during secret rotation: you can verify that both the old and new versions of a secret are accessible before completing a rotation.

Returns `Ok(None)` if no version of the secret was configured at that time.

```rust,no_run
use fastedge::secret;
use std::time::{SystemTime, UNIX_EPOCH};
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    // Retrieve the secret that was valid 5 minutes ago
    let five_minutes_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() as u32
        - 300;

    let previous_secret = secret::get_effective_at("SIGNING_KEY", five_minutes_ago)?;
    let current_secret = secret::get("SIGNING_KEY")?;

    // Use previous_secret and current_secret for rotation validation
    let _ = (previous_secret, current_secret);

    Ok(Response::builder()
        .status(200)
        .body(Body::empty())?)
}
```

### Security Notes

- Never include secret values in HTTP responses, log output, or diagnostic messages.
- Secret values are returned as raw bytes (`Vec<u8>`). Convert to a string only when the secret is defined as UTF-8 text, and handle the conversion error explicitly.
- Access to secrets is controlled by platform configuration. Unauthorized access returns `Err(secret::Error)`, not `Ok(None)`. `Ok(None)` means the secret is not configured or not found — it does not indicate an authorization failure.
- Clear secret material from memory as soon as it is no longer needed. Rust's ownership model helps with this: binding a secret to a local variable ensures it is dropped at the end of its scope.

CDN apps use `fastedge::proxywasm::secret` instead — see [CDN_APPS.md](CDN_APPS.md) for the ProxyWasm API surface and usage examples.

---

## Dictionary

The `fastedge::dictionary` module provides fast, read-only lookups for configuration values that do not change during the lifetime of a deployment.

### Configuration Lookups

```rust
pub fn get(key: &str) -> Option<String>
```

Returns `Some(value)` if the key exists and its value is valid UTF-8, or `None` if the key is not found or the value cannot be decoded as UTF-8.

Dictionary values are environment variables set at deployment time via the platform — the same management mechanism as secrets, but without encryption. They are not writable from application code.

```rust,no_run
use fastedge::dictionary;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let upstream = dictionary::get("upstream_origin")
        .unwrap_or_else(|| "https://default.example.com".to_string());

    let timeout_ms: u64 = dictionary::get("timeout_ms")
        .and_then(|v| v.parse().ok())
        .unwrap_or(5000);

    Ok(Response::builder()
        .status(200)
        .body(Body::from(format!(
            "Upstream: {}, Timeout: {}ms",
            upstream, timeout_ms
        )))?)
}
```

### When to Use Dictionary vs Key-Value vs Secrets

| Criterion                    | `dictionary`                          | `key_value`                               | `secret`                                    |
| ---------------------------- | ------------------------------------- | ----------------------------------------- | ------------------------------------------- |
| **Mutability**               | Read-only; set at deployment time     | Read-only from application code           | Read-only; managed by platform              |
| **Value type**               | UTF-8 strings only                    | Arbitrary bytes                           | Arbitrary bytes                             |
| **Advanced data structures** | No                                    | Sorted sets, bloom filters, glob scan     | No                                          |
| **Confidentiality**          | Not encrypted; visible in config      | Not encrypted at the application layer    | Encrypted at rest; access-controlled        |
| **Typical use cases**        | Feature flags, routing config, tuning | Caching, counters, state, rate-limit data | API keys, tokens, certificates, credentials |
| **Versioning / rotation**    | No                                    | No                                        | Yes, via `get_effective_at`                 |

Use `dictionary` for simple, non-sensitive string configuration that is known at deployment time. Use `key_value` for larger datasets, binary values, or data that requires advanced query patterns. Use `secret` for any value that must be kept confidential. Data the application itself must write belongs in a database, see [Databases](#databases).

CDN apps use `fastedge::proxywasm::dictionary` instead — see [CDN_APPS.md](CDN_APPS.md) for the ProxyWasm API surface and usage examples.

---

## Utilities

The `fastedge::utils` module provides diagnostic functions for monitoring and debugging edge applications.

### Diagnostics

```rust
pub fn set_user_diag(value: &str)
```

Writes a diagnostic string that appears in the FastEdge platform logs associated with the current request. This is intended for debugging and operational monitoring. There is no return value; the function panics if the host rejects the call.

The FastEdge platform captures only **stdout** for application log output. `stderr` is silently discarded and will not appear in the platform's log viewer. Use `println!` (or logging crates that write to stdout) for any output you need to observe. Do not use `eprintln!` — it produces no visible output on the platform.

```rust,no_run
use fastedge::key_value::Store;
use fastedge::utils::set_user_diag;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    set_user_diag("handler entered");

    let store = Store::open("cache")?;

    match store.get("config:version")? {
        Some(v) => {
            set_user_diag(&format!("config version: {}", String::from_utf8_lossy(&v)));
        }
        None => {
            set_user_diag("config version: not found");
        }
    }

    Ok(Response::builder()
        .status(200)
        .body(Body::empty())?)
}
```

One diagnostic message per request is the typical pattern. If `set_user_diag` is called multiple times, the platform may record only the last value or concatenate them depending on runtime behavior.

Do not write sensitive values (secrets, credentials, personally identifiable information) to diagnostics, as the output appears in platform logs that may be accessible to operations personnel.

CDN apps use `fastedge::proxywasm::utils` instead — see [CDN_APPS.md](CDN_APPS.md) for the ProxyWasm API surface and usage examples.

---

## Databases

The `fastedge::db` module gives an HTTP app access to a database the customer owns. The database is bound to the app by name (for example `"main"`): the host holds the connection details, and the app only knows the name. There are two kinds:

- SQL: `fastedge::db::sql` (dialect-neutral) and `fastedge::db::postgres` (the same API named for PostgreSQL).
- Database KV: `fastedge::db::kv`, a regional, durable, writable key-value database.

All calls are blocking. Each `query` auto-commits; each `batch` runs in one transaction. Calls are subject to host limits (timeouts, response and value sizes, statements or operations per batch); exceeding one returns `Error::Timeout` or `Error::Limit`.

`db::sql`, `db::postgres` and `db::kv` are HTTP-app APIs only. CDN apps have no `gcore:db` access, and `fastedge::db::kv` is not `fastedge::key_value`.

To use the typed query helpers (`QueryAs`, `rows_as`), enable the `json` feature:

```toml
[dependencies]
fastedge = { version = "0.4.5", features = ["json"] }
wstd = "0.6"
anyhow = "1"
serde = { version = "1", features = ["derive"] }
```

Build for the `wasm32-wasip2` target:

```bash
cargo build --release --target wasm32-wasip2
```

### Choosing Storage

| Storage      | Module                                                                     | Writable from the app | Location                 | Use for                        |
| ------------ | -------------------------------------------------------------------------- | --------------------- | ------------------------ | ------------------------------ |
| Edge Storage | `fastedge::key_value`                                                      | No                    | Replicated to every edge | Read-mostly data at every edge |
| Database KV  | `fastedge::db::kv`                                                         | Yes                   | Regional, durable        | Counters, rate limits, state   |
| SQL database | `fastedge::db::sql`, or `fastedge::db::postgres` for a PostgreSQL database | Yes                   | Regional, durable        | Relational data, transactions  |

Database KV is also not the PoP-local cache.

#### Binding a database

The app opens a database by name with `Database::open("main")`. The name is bound to a database in the app's configuration, outside the code; the app never sees the connection details.

Schema changes (migrations) are run from your machine or CI with your usual tool, not by the app.

The host does not check that a binding's database matches the module used to open it. `db::postgres` names the dialect; it does not verify it.

### SQL (`fastedge::db::sql`)

The dialect-neutral SQL module. The SQL text, the placeholder syntax and the error codes are those of the bound database (`$1`, `$2`, ... for PostgreSQL). A binding can move to another provider of the same dialect without code changes; moving to a database of another dialect means changing the SQL.

```rust
impl Database {
    pub fn open(name: &str) -> Result<Database, Error>
    pub fn query(&self, sql: &str, params: &[Value]) -> Result<QueryResult, Error>
    pub fn batch(&self, statements: &[Statement]) -> Result<Vec<QueryResult>, Error>
}
```

- `open` returns `Error::NoSuchDatabase` if the name is not bound.
- `query` runs one statement with positional parameters and auto-commits. `sql` must contain exactly one statement.
- `batch` runs several statements in one transaction: either all commit or none do. It returns one `QueryResult` per statement, in order. On failure nothing is committed and the error of the failing statement is returned.

#### Result and record types

| Type        | Field     | Type            | Description                              |
| ----------- | --------- | --------------- | ---------------------------------------- |
| QueryResult | columns   | Vec<Column>     | Result columns, in order                 |
| QueryResult | rows      | Vec<Vec<Value>> | Rows in column order; empty when no rows |
| QueryResult | row_count | u64             | Rows returned or affected                |
| Column      | name      | String          | Column name or alias                     |
| Column      | type_name | String          | Lower-case type name; empty if unnamed   |
| Statement   | sql       | String          | One SQL statement                        |
| Statement   | params    | Vec<Value>      | Positional parameters                    |

`row_count` is the number of rows returned (`SELECT`) or affected (`INSERT`, `UPDATE`, `DELETE`).

#### `Value`

Parameters and result cells are `Value`. Results use the narrowest variant that holds the column's type without loss.

| Variant | Holds   | Meaning                                |
| ------- | ------- | -------------------------------------- |
| Null    | none    | SQL `NULL`                             |
| Boolean | bool    | Boolean                                |
| Int32   | i32     | 32-bit integer                         |
| Int64   | i64     | 64-bit integer                         |
| Float64 | f64     | Floating point                         |
| Numeric | String  | Exact decimal, as text                 |
| Text    | String  | Any other type, as the database's text |
| Bytes   | Vec<u8> | Binary data                            |
| Json    | String  | JSON, as text                          |

Parameters are converted to the target column type by the database, as its dialect does.

#### `params!` and `ToSql`

```rust
pub trait ToSql {
    fn to_sql(self) -> Value;
}
```

`params![a, b, ...]` builds a `&[Value]` from plain Rust values by calling `ToSql::to_sql` on each; `params![]` is an empty list. `ToSql` is implemented for:

| Rust type   | Value                                      |
| ----------- | ------------------------------------------ |
| `bool`      | `Value::Boolean`                           |
| `i32`       | `Value::Int32`                             |
| `i64`       | `Value::Int64`                             |
| `f64`       | `Value::Float64`                           |
| `&str`      | `Value::Text`                              |
| `String`    | `Value::Text`                              |
| `Vec<u8>`   | `Value::Bytes`                             |
| `Option<T>` | `Value::Null` for `None`; otherwise as `T` |

Integer literals default to `i32`; write `42i64` for an `int8` column.

```rust,no_run
use fastedge::db::sql::{params, Database, Statement, Value};
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    // "main" is the binding name; the host resolves it to the app's database.
    let db = Database::open("main")?;

    let result = db.query("select id, email from users where id = $1", params![42i64])?;
    println!("{} rows, {} columns", result.row_count, result.columns.len());
    for row in &result.rows {
        if let [Value::Int64(id), Value::Text(email)] = row.as_slice() {
            println!("{id}: {email}");
        }
    }

    // One transaction: both statements commit, or neither does.
    let results = db.batch(&[
        Statement {
            sql: "update counters set n = n + 1 where name = $1".to_string(),
            params: params!["hits"].to_vec(),
        },
        Statement {
            sql: "select n from counters where name = $1".to_string(),
            params: params!["hits"].to_vec(),
        },
    ])?;

    Ok(Response::builder()
        .status(200)
        .body(Body::from(format!("{} results", results.len())))?)
}
```

#### Typed queries (feature `json`)

```rust
pub trait QueryAs {
    fn query_as<T: DeserializeOwned>(&self, sql: &str, params: &[Value]) -> Result<Vec<T>, QueryAsError>;
}

pub fn rows_as<T: DeserializeOwned>(result: QueryResult) -> Result<Vec<T>, QueryAsError>
```

`Database::query_as::<T>` runs one statement and decodes every row into any `serde::Deserialize` type by column name. `rows_as` does the same for an existing `QueryResult`, for example one entry of a `batch` result. Decoding rules:

- `json` / `jsonb` columns are parsed, so a field can be a nested struct.
- `int8` is a JSON number; `numeric` and text-format types (timestamps, UUIDs, ...) are strings; `bytea` is a byte array; `NULL` is `None`.
- Non-finite floats (`NaN`, infinities) are kept as strings, so an `f64` field fails to decode instead of reading `NULL`.

| Variant                       | Description                            |
| ----------------------------- | -------------------------------------- |
| `QueryAsError::Db(Error)`     | The database or host reported an error |
| `QueryAsError::Decode(String)` | A row did not match the target type    |

`Decode` is returned for a missing column or a wrong type.

```rust,ignore
use fastedge::db::sql::{params, Database, QueryAs};

#[derive(serde::Deserialize)]
struct User {
    id: i64,
    email: String,
    nickname: Option<String>,
}

let users: Vec<User> = Database::open("main")?
    .query_as("select id, email, nickname from users where id = $1", params![42i64])?;
```

### PostgreSQL (`fastedge::db::postgres`)

The `sql` API named for PostgreSQL, plus SQLSTATE helpers. Use this module with a PostgreSQL database (for example Neon). It re-exports `Database`, `Column`, `DbError`, `Error`, `QueryResult`, `Statement`, `ToSql`, `Value`, `params!`, and, with the `json` feature, `QueryAs`, `QueryAsError` and `rows_as`; their definitions are in [SQL](#sql-fastedgedbsql).

Write PostgreSQL: `$1`, `$2`, ... placeholders, `RETURNING`, `jsonb`. The host does not check the dialect: bind a PostgreSQL database to the name.

#### Value mapping

| PostgreSQL type    | Value     | Notes                                        |
| ------------------ | --------- | -------------------------------------------- |
| `int2`, `int4`     | `Int32`   |                                              |
| `int8`             | `Int64`   |                                              |
| `float4`, `float8` | `Float64` |                                              |
| `numeric`          | `Numeric` | Decimal text                                 |
| `bool`             | `Boolean` |                                              |
| `json`, `jsonb`    | `Json`    | JSON text                                    |
| `bytea`            | `Bytes`   |                                              |
| Anything else      | `Text`    | PostgreSQL text format; `timestamptz` in UTC |
| `NULL`             | `Null`    |                                              |

#### Extension and user-defined types

Extension and user-defined types (pgvector `vector`, PostGIS `geometry`, enums, domains) work as text:

- Send them as text parameters, casting in SQL where PostgreSQL cannot infer the type, for example `order by embedding <-> $1::vector`.
- They come back as `Text` in PostgreSQL's text format (`"[0.1,0.2]"` for a vector).
- Their `Column::type_name` is empty: the host names only PostgreSQL's built-in types.

#### SQLSTATE

```rust
pub trait SqlState {
    fn sqlstate(&self) -> Option<&str>;
}
```

`SqlState` is implemented for `Error` and, with the `json` feature, for `QueryAsError`. `sqlstate()` returns the SQLSTATE code for `Error::Query` and `Error::Config`, and `None` for errors that did not come from the database (no binding, timeout, connection, limit) and for `QueryAsError::Decode`. Compare it with the constants in `fastedge::db::postgres::sqlstate`. Any other code is still reported in `DbError::code`.

| Constant                       | Code    | Meaning                                      |
| ------------------------------ | ------- | -------------------------------------------- |
| `UNIQUE_VIOLATION`             | `23505` | Duplicate in a unique constraint or index    |
| `FOREIGN_KEY_VIOLATION`        | `23503` | Foreign key points at a missing row          |
| `NOT_NULL_VIOLATION`           | `23502` | `NULL` in a `NOT NULL` column                |
| `CHECK_VIOLATION`              | `23514` | A `CHECK` constraint failed                  |
| `EXCLUSION_VIOLATION`          | `23P01` | An exclusion constraint failed               |
| `INVALID_TEXT_REPRESENTATION`  | `22P02` | Text not parseable as the column type        |
| `NUMERIC_VALUE_OUT_OF_RANGE`   | `22003` | Number out of the column type's range        |
| `STRING_DATA_RIGHT_TRUNCATION` | `22001` | String longer than the column allows         |
| `DIVISION_BY_ZERO`             | `22012` | Division by zero                             |
| `UNDEFINED_TABLE`              | `42P01` | Table does not exist                         |
| `UNDEFINED_COLUMN`             | `42703` | Column does not exist                        |
| `UNDEFINED_FUNCTION`           | `42883` | No function matches the name and arguments   |
| `SYNTAX_ERROR`                 | `42601` | SQL syntax error (e.g. `?` instead of `$1`)  |
| `INSUFFICIENT_PRIVILEGE`       | `42501` | Role lacks a privilege                       |
| `SERIALIZATION_FAILURE`        | `40001` | Serialization failure; retry the transaction |
| `DEADLOCK_DETECTED`            | `40P01` | Deadlock detected; retry the transaction     |
| `QUERY_CANCELED`               | `57014` | Statement cancelled (e.g. a timeout)         |
| `INVALID_PASSWORD`             | `28P01` | Wrong role password (`Error::Config`)        |
| `INVALID_CATALOG_NAME`         | `3D000` | Database does not exist (`Error::Config`)    |

All constants are `&str`.

```rust,no_run
use fastedge::db::postgres::{params, sqlstate, Database, SqlState};
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    // "main" is the binding name; the host resolves it to the app's database.
    let db = Database::open("main").map_err(|e| anyhow::anyhow!("open database: {e:?}"))?;

    let status = match db.query(
        "insert into users (email) values ($1)",
        params!["ann@example.com"],
    ) {
        Ok(_) => 201,
        Err(e) if e.sqlstate() == Some(sqlstate::UNIQUE_VIOLATION) => 409,
        Err(e) => return Err(anyhow::anyhow!("insert failed: {e:?}")),
    };

    Ok(Response::builder().status(status).body(Body::empty())?)
}
```

### Database KV (`fastedge::db::kv`)

Regional, durable key-value storage that the app can write. It is not Edge Storage (`fastedge::key_value`, read-only, replicated to every edge) and not the PoP-local cache.

```rust
impl Database {
    pub fn open(name: &str) -> Result<Database, Error>
    pub fn get(&self, key: &str) -> Result<Option<KvValue>, Error>
    pub fn exists(&self, key: &str) -> Result<bool, Error>
    pub fn set(&self, key: &str, value: &[u8], options: SetOptions) -> Result<bool, Error>
    pub fn delete(&self, key: &str) -> Result<bool, Error>
    pub fn incr(&self, key: &str, delta: i64) -> Result<i64, Error>
    pub fn expire(&self, key: &str, ttl_ms: u64) -> Result<bool, Error>
    pub fn batch(&self, ops: &[Op]) -> Result<Vec<OpResult>, Error>
}

pub type KvValue = Vec<u8>;
```

| Method   | Behavior                                                                                                                                                         |
| -------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `open`   | Opens the database bound to `name`; `Error::NoSuchDatabase` if the name is not bound                                                                             |
| `get`    | `Ok(None)` if the key does not exist or has expired                                                                                                              |
| `exists` | Whether the key exists and has not expired, without fetching its value                                                                                           |
| `set`    | `Ok(true)` if written, `Ok(false)` if `options.condition` was not met. A successful `set` always replaces the key's previous expiry                              |
| `delete` | `Ok(true)` if the key existed, `Ok(false)` otherwise                                                                                                             |
| `incr`   | Adds `delta` (may be negative) to the integer at `key`; a missing key counts as `0`; the key's expiry is unchanged. Returns the new value                        |
| `expire` | Sets the expiry to `ttl_ms` milliseconds from now; `Ok(false)` if the key does not exist                                                                         |
| `batch`  | Runs the operations atomically; see below                                                                                                                        |

`incr` fails with `Error::Operation` if the stored value is not a decimal 64-bit integer or the result would overflow.

#### Set options

```rust
pub struct SetOptions {
    pub ttl_ms: Option<u64>,
    pub condition: SetCondition,
}

pub enum SetCondition {
    Always,
    IfNotExists,
    IfExists,
}
```

`ttl_ms` of `None` means no expiry. `SetOptions::default()` is no expiry and `SetCondition::Always`.

| Variant       | Writes                         |
| ------------- | ------------------------------ |
| `Always`      | Always; the default            |
| `IfNotExists` | Only if the key does not exist |
| `IfExists`    | Only if the key exists         |

#### Atomic batches

`batch` applies all operations or none, and no other client observes or interleaves with a partial batch. Operations see the effects of earlier operations in the same batch. Each `Op` has the same semantics as the method of the same name, and `batch` returns one `OpResult` per operation, in order.

```rust
pub struct SetOp { pub key: String, pub value: Vec<u8>, pub options: SetOptions }
pub struct IncrOp { pub key: String, pub delta: i64 }
pub struct ExpireOp { pub key: String, pub ttl_ms: u64 }
```

| Op                     | OpResult                         |
| ---------------------- | -------------------------------- |
| `Op::Get(String)`      | `OpResult::Get(Option<Vec<u8>>)` |
| `Op::Exists(String)`   | `OpResult::Exists(bool)`         |
| `Op::Set(SetOp)`       | `OpResult::Set(bool)`            |
| `Op::Delete(String)`   | `OpResult::Delete(bool)`         |
| `Op::Incr(IncrOp)`     | `OpResult::Incr(i64)`            |
| `Op::Expire(ExpireOp)` | `OpResult::Expire(bool)`         |

A `set` whose condition is not met is not a failure: it yields `OpResult::Set(false)` and the batch continues. Any other failing operation aborts the whole batch with `Error::Operation`, whose `op_index` identifies the operation.

#### Values and consistency

- Values are bytes and must be valid UTF-8 for now. Writing a non-UTF-8 value fails with `Error::Limit`.
- Within one request, reads see that request's own writes (read-your-writes).
- Other requests, possibly on other edges, see a write once it has replicated: immediately for a database in one region, eventually for one with read replicas in other regions. `batch` is atomic either way.

```rust,no_run
use fastedge::db::kv::{Database, Error, IncrOp, Op, OpResult, SetCondition, SetOp, SetOptions};
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    // "main" is the binding name; the host resolves it to the app's database.
    let db = Database::open("main").map_err(|e| anyhow::anyhow!("open database: {e:?}"))?;

    // One atomic batch: start a 60 s window counter if missing, then count this request.
    let ops = [
        Op::Set(SetOp {
            key: "rl:ann".into(),
            value: b"0".to_vec(),
            options: SetOptions {
                ttl_ms: Some(60_000),
                condition: SetCondition::IfNotExists,
            },
        }),
        Op::Incr(IncrOp {
            key: "rl:ann".into(),
            delta: 1,
        }),
    ];

    let requests = match db.batch(&ops) {
        Ok(results) => match results.as_slice() {
            [_, OpResult::Incr(n)] => *n,
            _ => {
                return Ok(Response::builder()
                    .status(500)
                    .body(Body::from("unexpected batch result"))?)
            }
        },
        // e.g. code "WRONGTYPE": the key holds something that is not a counter
        Err(Error::Operation(e)) => {
            return Ok(Response::builder().status(500).body(Body::from(e.message))?)
        }
        Err(e) => return Err(anyhow::anyhow!("batch failed: {e:?}")),
    };

    if requests > 5 {
        return Ok(Response::builder()
            .status(429)
            .body(Body::from("rate limited"))?);
    }

    Ok(Response::builder()
        .status(200)
        .body(Body::from(format!("request {requests} in this window")))?)
}
```

### Error Handling

#### SQL errors

`fastedge::db::sql::Error` (re-exported by `fastedge::db::postgres`):

| Variant                    | Description                                         |
| -------------------------- | --------------------------------------------------- |
| `Error::NoSuchDatabase`    | No database is bound to the name                    |
| `Error::AccessDenied`      | The app has no access to the database               |
| `Error::Config(DbError)`   | Binding misconfigured; needs a configuration change |
| `Error::Connection(String)` | Database unreachable (network, DNS, TLS)           |
| `Error::Timeout`           | Time limit exceeded; outcome is unknown             |
| `Error::Limit(String)`     | A host or backend limit was exceeded                |
| `Error::Query(DbError)`    | The database rejected the statement                 |
| `Error::InternalError`     | Unexpected internal error                           |

On `Error::Timeout` the statement may or may not have been applied. `Error::Config` is not retryable without a configuration change (bad credentials, unknown database name, invalid connection settings). `Error::Query` covers syntax errors, constraint violations and permission errors.

`DbError` carries the details the database reported:

| Field      | Type           | Description                                   |
| ---------- | -------------- | --------------------------------------------- |
| code       | Option<String> | Database error code (SQLSTATE for PostgreSQL) |
| message    | String         | Error message                                 |
| detail     | Option<String> | Further detail, when reported                 |
| hint       | Option<String> | Hint, when reported                           |
| constraint | Option<String> | Name of the violated constraint, if any       |

#### Database KV errors

`fastedge::db::kv::Error`:

| Variant                     | Description                                         |
| --------------------------- | --------------------------------------------------- |
| `Error::NoSuchDatabase`     | No database is bound to the name                    |
| `Error::AccessDenied`       | No access to the database or operation              |
| `Error::Config(String)`     | Binding misconfigured; needs a configuration change |
| `Error::Connection(String)` | Database unreachable (network, DNS, TLS)            |
| `Error::Timeout`            | Time limit exceeded; outcome is unknown             |
| `Error::Limit(String)`      | A host or backend limit was exceeded                |
| `Error::Operation(OpError)` | An operation failed (e.g. `incr` on a non-integer)  |
| `Error::InternalError`      | Unexpected internal error                           |

`Error::AccessDenied` also covers an operation the binding does not permit, for example a write through a read-only binding. On `Error::Timeout` the operation may or may not have been applied.

`OpError` carries the details the backend reported:

| Field    | Type           | Description                                       |
| -------- | -------------- | ------------------------------------------------- |
| op_index | Option<u32>    | Failing operation's index; `None` outside `batch` |
| code     | Option<String> | Backend error code (e.g. `WRONGTYPE`), if any     |
| message  | String         | Error message                                     |

---

## See Also

- [SDK_API.md](SDK_API.md) — Core HTTP handler, `send_request`, `Body`, and the `#[fastedge::http]` macro
- [CDN_APPS.md](CDN_APPS.md) — ProxyWasm API for CDN apps (`fastedge::proxywasm::*`)
- [quickstart.md](quickstart.md) — Getting started guide
- [INDEX.md](INDEX.md) — Documentation index
