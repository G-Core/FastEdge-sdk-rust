//! Customer-owned databases bound to the app by name (`gcore:db`).
//!
//! The connection details stay with the host; the app only knows the binding name.

/// SQL database, `gcore:db/sql-sync`. The SQL dialect (placeholders, types, error codes) is that of
/// the database bound to the name. For a PostgreSQL database, use [`postgres`]: the same API, with
/// PostgreSQL docs and SQLSTATE helpers.
///
/// Calls are blocking. Parameters are positional, in the database's placeholder syntax (`$1`, `$2`,
/// ... for PostgreSQL).
///
/// # Examples
///
/// ```no_run
/// use fastedge::db::sql::{params, Database};
///
/// let db = Database::open("main")?;
/// let result = db.query("select id, email from users where id = $1", params![42i64])?;
/// println!("{} rows", result.row_count);
/// # Ok::<(), fastedge::db::sql::Error>(())
/// ```
///
/// With the `json` feature, rows decode into any `serde::Deserialize` type by column name:
///
/// ```ignore
/// use fastedge::db::sql::{params, Database, QueryAs};
///
/// #[derive(serde::Deserialize)]
/// struct User { id: i64, email: String, nickname: Option<String> }
///
/// let users: Vec<User> = Database::open("main")?
///     .query_as("select id, email, nickname from users where id = $1", params![42i64])?;
/// ```
pub mod sql {
    #[doc(inline)]
    pub use crate::gcore::db::sql_sync::Database;
    #[doc(inline)]
    pub use crate::gcore::db::sql_types::{Column, DbError, Error, QueryResult, Statement, Value};

    #[doc(inline)]
    pub use crate::__db_sql_params as params;

    /// Converts a Rust value into a SQL parameter [`Value`].
    ///
    /// Implemented for `bool`, `i32`, `i64`, `f64`, `&str`, `String`, `Vec<u8>` (bytea) and
    /// `Option<T>` (`None` is SQL `NULL`).
    pub trait ToSql {
        /// The parameter value.
        fn to_sql(self) -> Value;
    }

    macro_rules! to_sql {
        ($($t:ty => $v:ident),*) => {$(
            impl ToSql for $t {
                fn to_sql(self) -> Value {
                    Value::$v(self.into())
                }
            }
        )*};
    }
    to_sql!(bool => Boolean, i32 => Int32, i64 => Int64, f64 => Float64, String => Text,
        &str => Text, Vec<u8> => Bytes);

    impl<T: ToSql> ToSql for Option<T> {
        fn to_sql(self) -> Value {
            self.map_or(Value::Null, ToSql::to_sql)
        }
    }

    #[cfg(feature = "json")]
    pub use typed::{rows_as, QueryAs, QueryAsError};

    #[cfg(feature = "json")]
    mod typed {
        use super::{Database, Error, QueryResult, Value};
        use serde::de::DeserializeOwned;
        use serde_json::{Map, Value as Json};

        /// Error from [`QueryAs::query_as`].
        #[derive(Debug, thiserror::Error)]
        pub enum QueryAsError {
            /// The database or host reported an error.
            #[error("{0}")]
            Db(#[from] Error),
            /// A row didn't match the target type (missing column, wrong type).
            #[error("row decode: {0}")]
            Decode(String),
        }

        /// Typed queries: decode rows into any `serde::Deserialize` type by column name.
        pub trait QueryAs {
            /// Run one statement and decode every row into `T`.
            fn query_as<T: DeserializeOwned>(
                &self,
                sql: &str,
                params: &[Value],
            ) -> Result<Vec<T>, QueryAsError>;
        }

        impl QueryAs for Database {
            fn query_as<T: DeserializeOwned>(
                &self,
                sql: &str,
                params: &[Value],
            ) -> Result<Vec<T>, QueryAsError> {
                rows_as(self.query(sql, params)?)
            }
        }

        /// Decode every row of a result (e.g. one entry of a `batch` result) into `T`.
        ///
        /// `json`/`jsonb` columns are parsed, so a field can be a nested struct. `int8` is a
        /// JSON number, `numeric` and text-format types (timestamps, uuid, ...) are strings,
        /// `bytea` is a byte array, `NULL` is `None`.
        pub fn rows_as<T: DeserializeOwned>(result: QueryResult) -> Result<Vec<T>, QueryAsError> {
            result
                .rows
                .into_iter()
                .map(|row| {
                    let obj: Map<String, Json> = result
                        .columns
                        .iter()
                        .zip(row)
                        .map(|(c, v)| (c.name.clone(), to_json(v)))
                        .collect();
                    serde_json::from_value(Json::Object(obj))
                        .map_err(|e| QueryAsError::Decode(e.to_string()))
                })
                .collect()
        }

        fn to_json(v: Value) -> Json {
            match v {
                Value::Null => Json::Null,
                Value::Boolean(b) => b.into(),
                Value::Int32(i) => i.into(),
                Value::Int64(i) => i.into(),
                // JSON has no NaN/Infinity (serde_json would make them null): keep them as
                // strings so an `f64` field fails to decode instead of reading NULL.
                Value::Float64(f) if !f.is_finite() => Json::String(f.to_string()),
                Value::Float64(f) => f.into(),
                Value::Json(s) => serde_json::from_str(&s).unwrap_or(Json::String(s)),
                Value::Numeric(s) | Value::Text(s) => s.into(),
                Value::Bytes(b) => b.into(),
            }
        }
    }
}

/// PostgreSQL database, over `gcore:db/sql-sync`: the [`sql`] API, named for PostgreSQL, plus
/// SQLSTATE helpers.
///
/// Write PostgreSQL: `$1`, `$2`, ... placeholders, `RETURNING`, `jsonb`. The host does not check the
/// dialect: bind a PostgreSQL database (e.g. Neon) to the name. Calls are blocking; each `query`
/// auto-commits and each `batch` runs in one transaction.
///
/// Values: `int2`/`int4` → `Int32`, `int8` → `Int64`, `float4`/`float8` → `Float64`, `numeric` →
/// `Numeric` (decimal text), `bool` → `Boolean`, `json`/`jsonb` → `Json` (text), `bytea` → `Bytes`,
/// anything else → `Text` in PostgreSQL text format (`timestamptz` in UTC). `NULL` → `Null`.
///
/// Extension and user-defined types (pgvector `vector`, PostGIS `geometry`, enums, domains) work as
/// text: send them as text parameters, casting in SQL where PostgreSQL cannot infer the type (e.g.
/// `order by embedding <-> $1::vector`), and they come back as `Text` in PostgreSQL's text format
/// (`"[0.1,0.2]"` for a vector). Their `Column::type_name` is empty: the host names only
/// PostgreSQL's built-in types.
///
/// # Examples
///
/// ```no_run
/// use fastedge::db::postgres::{params, sqlstate, Database, Error, SqlState};
///
/// let db = Database::open("main")?;
/// match db.query("insert into users (email) values ($1)", params!["ann@example.com"]) {
///     Ok(_) => {}
///     Err(e) if e.sqlstate() == Some(sqlstate::UNIQUE_VIOLATION) => println!("already registered"),
///     Err(e) => return Err(e),
/// }
/// # Ok::<(), Error>(())
/// ```
pub mod postgres {
    #[cfg(feature = "json")]
    #[doc(inline)]
    pub use super::sql::{rows_as, QueryAs, QueryAsError};
    #[doc(inline)]
    pub use super::sql::{Column, Database, DbError, Error, QueryResult, Statement, ToSql, Value};

    #[doc(inline)]
    pub use crate::__db_sql_params as params;

    /// SQLSTATE codes apps commonly branch on (PostgreSQL `errcodes.txt`). Any other code is
    /// still reported in [`DbError::code`].
    pub mod sqlstate {
        /// `23505`: a unique constraint or index rejected a duplicate.
        pub const UNIQUE_VIOLATION: &str = "23505";
        /// `23503`: a foreign key points at a missing row.
        pub const FOREIGN_KEY_VIOLATION: &str = "23503";
        /// `23502`: `NULL` in a `NOT NULL` column.
        pub const NOT_NULL_VIOLATION: &str = "23502";
        /// `23514`: a `CHECK` constraint failed.
        pub const CHECK_VIOLATION: &str = "23514";
        /// `23P01`: an exclusion constraint failed.
        pub const EXCLUSION_VIOLATION: &str = "23P01";
        /// `22P02`: text could not be parsed as the column type (e.g. `'abc'` for an integer).
        pub const INVALID_TEXT_REPRESENTATION: &str = "22P02";
        /// `22003`: a number out of the column type's range.
        pub const NUMERIC_VALUE_OUT_OF_RANGE: &str = "22003";
        /// `22001`: a string longer than the column allows.
        pub const STRING_DATA_RIGHT_TRUNCATION: &str = "22001";
        /// `22012`: division by zero.
        pub const DIVISION_BY_ZERO: &str = "22012";
        /// `42P01`: the table does not exist (e.g. migrations not applied).
        pub const UNDEFINED_TABLE: &str = "42P01";
        /// `42703`: the column does not exist.
        pub const UNDEFINED_COLUMN: &str = "42703";
        /// `42883`: no function matches the name and argument types.
        pub const UNDEFINED_FUNCTION: &str = "42883";
        /// `42601`: SQL syntax error (e.g. `?` placeholders instead of `$1`).
        pub const SYNTAX_ERROR: &str = "42601";
        /// `42501`: the database role lacks a privilege.
        pub const INSUFFICIENT_PRIVILEGE: &str = "42501";
        /// `40001`: serialization failure; the transaction may be retried.
        pub const SERIALIZATION_FAILURE: &str = "40001";
        /// `40P01`: deadlock detected; the transaction may be retried.
        pub const DEADLOCK_DETECTED: &str = "40P01";
        /// `57014`: the statement was cancelled (e.g. `statement_timeout`).
        pub const QUERY_CANCELED: &str = "57014";
        /// `28P01`: wrong password for the database role (reported as `Error::Config`).
        pub const INVALID_PASSWORD: &str = "28P01";
        /// `3D000`: the database does not exist (reported as `Error::Config`).
        pub const INVALID_CATALOG_NAME: &str = "3D000";
    }

    /// The SQLSTATE of an error, when the database reported one.
    pub trait SqlState {
        /// The SQLSTATE code (compare with the [`sqlstate`] constants), or `None` for errors that
        /// did not come from the database (no binding, timeout, connection, limit).
        fn sqlstate(&self) -> Option<&str>;
    }

    impl SqlState for Error {
        fn sqlstate(&self) -> Option<&str> {
            match self {
                Error::Query(e) | Error::Config(e) => e.code.as_deref(),
                _ => None,
            }
        }
    }

    #[cfg(feature = "json")]
    impl SqlState for QueryAsError {
        fn sqlstate(&self) -> Option<&str> {
            match self {
                QueryAsError::Db(e) => e.sqlstate(),
                QueryAsError::Decode(_) => None,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{sqlstate, DbError, Error, SqlState};

        fn db_error(code: Option<&str>) -> DbError {
            DbError {
                code: code.map(String::from),
                message: "m".into(),
                detail: None,
                hint: None,
                constraint: None,
            }
        }

        #[test]
        fn sqlstate_comes_from_database_errors_only() {
            let unique = Error::Query(db_error(Some(sqlstate::UNIQUE_VIOLATION)));
            assert_eq!(unique.sqlstate(), Some("23505"));
            let auth = Error::Config(db_error(Some(sqlstate::INVALID_PASSWORD)));
            assert_eq!(auth.sqlstate(), Some("28P01"));
            assert_eq!(Error::Query(db_error(None)).sqlstate(), None);
            assert_eq!(Error::NoSuchDatabase.sqlstate(), None);
            assert_eq!(Error::Timeout.sqlstate(), None);
        }
    }
}

/// Database KV (regional, durable, writable key-value), `gcore:db/kv-sync`.
///
/// Not Edge Storage ([`crate::key_value`], read-only, replicated to every edge) and not the
/// PoP-local cache. Calls are blocking. Values are bytes and must be valid UTF-8 for now.
///
/// Within one request, reads see the request's own writes. Other requests may see a write only
/// after it has replicated (eventually, for a database with read replicas in other regions).
///
/// # Examples
///
/// ```no_run
/// use fastedge::db::kv::{Database, Op, SetCondition, SetOp, SetOptions, IncrOp};
///
/// let db = Database::open("main")?;
/// db.set("greeting", b"hello", SetOptions::default())?;
/// let visits = db.incr("visits", 1)?;
///
/// // Atomic: create a 60 s window counter if missing, then count this request.
/// let results = db.batch(&[
///     Op::Set(SetOp {
///         key: "rl:ann".into(),
///         value: b"0".to_vec(),
///         options: SetOptions { ttl_ms: Some(60_000), condition: SetCondition::IfNotExists },
///     }),
///     Op::Incr(IncrOp { key: "rl:ann".into(), delta: 1 }),
/// ])?;
/// # Ok::<(), fastedge::db::kv::Error>(())
/// ```
pub mod kv {
    #[doc(inline)]
    pub use crate::gcore::db::kv_sync::Database;
    #[doc(inline)]
    pub use crate::gcore::db::kv_types::{
        Error, ExpireOp, IncrOp, KvValue, Op, OpError, OpResult, SetCondition, SetOp, SetOptions,
    };

    /// Always write, no expiry.
    impl Default for SetOptions {
        fn default() -> Self {
            Self {
                ttl_ms: None,
                condition: SetCondition::Always,
            }
        }
    }
}

/// Builds a `&[Value]` parameter list from plain Rust values; see [`sql::ToSql`].
#[doc(hidden)]
#[macro_export]
macro_rules! __db_sql_params {
    ($($p:expr),* $(,)?) => { &[$($crate::db::sql::ToSql::to_sql($p)),*] };
}
