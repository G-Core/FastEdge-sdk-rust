[← Back to examples](../../../README.md)

# SQL database (WASI)

Reads users from a PostgreSQL database bound to the app as `main`, decoding rows into a Rust struct
with `query_as`.

## Setup

1. Bind the database: add a secret `FASTEDGE_DB_SQL_main` with the Postgres connection string
   (e.g. a Neon pooled URL). The app never sees it; the host resolves `Database::open("main")`.
2. Create the schema once, outside the app:

```sh
psql "$DATABASE_URL" -f schema.sql
```

Schema changes (migrations) are run from your machine or CI with your usual tool, not by the app.

## Usage

```
GET /       → 200 [{"id":1,"email":"ann@example.com","nickname":null,"created":"…+00","meta":{"plan":"pro"}}, …]
GET /?id=2  → 200 {"id":2,"email":"bob@example.com","nickname":"bobby","created":"…+00","meta":{"plan":"free"}}
GET /?id=9  → 404 {"error":"not found"}
```

## Error responses

| Condition | Status | Body |
|---|---|---|
| No database bound to `main` | 500 | `{"error":"NoSuchDatabase"}` |
| Schema not applied | 500 | `{"error":"relation \"example_users\" does not exist","code":"42P01"}` |
| Row doesn't match `User` | 500 | `{"error":"row decode: …"}` |

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip2/release/db_sql.wasm
```

Local run: `fastedge-run http -w target/wasm32-wasip2/release/db_sql.wasm --wasi-http true --secret "FASTEDGE_DB_SQL_main=postgresql://…"`

## APIs used

- `fastedge::db::sql::Database::open(name)` — open the database bound to `name`
- `db.query_as::<T>(sql, params)` — run one statement and decode rows into `T` (feature `json`)
- `params![…]` — build `$1, $2, …` parameters from plain Rust values
- `Error::Query(DbError { code, message, … })` — Postgres error with SQLSTATE
