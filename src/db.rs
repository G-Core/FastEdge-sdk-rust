//! Customer-owned databases bound to the app by name (`gcore:db`).
//!
//! The connection details stay with the host; the app only knows the binding name.

/// SQL database (PostgreSQL), `gcore:db/sql-sync`.
///
/// Calls are blocking. Parameters use positional placeholders (`$1`, `$2`, ...).
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

/// Builds a `&[Value]` parameter list from plain Rust values; see [`sql::ToSql`].
#[doc(hidden)]
#[macro_export]
macro_rules! __db_sql_params {
    ($($p:expr),* $(,)?) => { &[$($crate::db::sql::ToSql::to_sql($p)),*] };
}
