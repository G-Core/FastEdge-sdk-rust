use fastedge::db::sql::{params, Database, Error, QueryAs, QueryAsError};
use serde::{Deserialize, Serialize};
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[derive(Debug, Deserialize, Serialize)]
struct User {
    id: i64,
    email: String,
    nickname: Option<String>,
    created: String, // timestamptz, UTC text
    meta: Meta,      // jsonb, decoded into a struct
}

#[derive(Debug, Deserialize, Serialize)]
struct Meta {
    plan: String,
}

const SELECT_USERS: &str = "select id, email, nickname, created, meta from example_users";

#[wstd::http_server]
async fn main(request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let id = request
        .uri()
        .query()
        .and_then(|q| q.split('&').find_map(|kv| kv.strip_prefix("id=")))
        .map(str::parse::<i64>)
        .transpose()?;

    // "main" is the binding name; the host resolves it to the app's database.
    let db = match Database::open("main") {
        Ok(db) => db,
        Err(Error::NoSuchDatabase) => {
            return json(500, &serde_json::json!({ "error": "no such database" }))
        }
        Err(e) => return json(500, &serde_json::json!({ "error": format!("{e:?}") })),
    };

    let users = match id {
        Some(id) => db.query_as::<User>(&format!("{SELECT_USERS} where id = $1"), params![id]),
        None => db.query_as::<User>(&format!("{SELECT_USERS} order by id"), params![]),
    };

    match (id, users) {
        (Some(_), Ok(mut users)) => match users.pop() {
            Some(user) => json(200, &user),
            None => json(404, &serde_json::json!({ "error": "not found" })),
        },
        (None, Ok(users)) => json(200, &users),
        // e.g. 42P01 (undefined_table): schema.sql not applied yet
        (_, Err(QueryAsError::Db(Error::Query(e)))) => {
            json(500, &serde_json::json!({ "error": e.message, "code": e.code }))
        }
        (_, Err(e)) => json(500, &serde_json::json!({ "error": e.to_string() })),
    }
}

fn json(status: u16, body: &impl Serialize) -> anyhow::Result<Response<Body>> {
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(body)?))?)
}
