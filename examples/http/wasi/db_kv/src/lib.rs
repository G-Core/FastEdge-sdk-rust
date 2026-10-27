use fastedge::db::kv::{Database, Error, IncrOp, Op, OpResult, SetCondition, SetOp, SetOptions};
use serde_json::json;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

/// Requests one user may make per window.
const LIMIT: i64 = 5;
const WINDOW_MS: u64 = 60_000;

#[wstd::http_server]
async fn main(request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let user = request
        .uri()
        .query()
        .and_then(|q| q.split('&').find_map(|kv| kv.strip_prefix("user=")))
        .unwrap_or("anonymous")
        .to_string();

    // "main" is the binding name; the host resolves it to the app's database.
    let db = match Database::open("main") {
        Ok(db) => db,
        Err(Error::NoSuchDatabase) => return json(500, json!({ "error": "no such database" })),
        Err(e) => return json(500, json!({ "error": format!("{e:?}") })),
    };

    // One atomic batch: start the user's window counter if missing (it expires with the
    // window), count this request, count the visit.
    let window = format!("rl:{user}");
    let ops = [
        Op::Set(SetOp {
            key: window.clone(),
            value: b"0".to_vec(),
            options: SetOptions {
                ttl_ms: Some(WINDOW_MS),
                condition: SetCondition::IfNotExists,
            },
        }),
        Op::Incr(IncrOp {
            key: window,
            delta: 1,
        }),
        Op::Incr(IncrOp {
            key: "visits".into(),
            delta: 1,
        }),
    ];
    let (requests, visits) = match db.batch(&ops).as_deref() {
        Ok([_, OpResult::Incr(requests), OpResult::Incr(visits)]) => (*requests, *visits),
        Ok(_) => return json(500, json!({ "error": "unexpected batch result" })),
        // e.g. WRONGTYPE: "visits" holds something that isn't a counter
        Err(Error::Operation(e)) => {
            return json(500, json!({ "error": e.message, "code": e.code }))
        }
        Err(e) => return json(500, json!({ "error": format!("{e:?}") })),
    };

    if requests > LIMIT {
        return json(
            429,
            json!({ "error": "rate limited", "user": user, "limit": LIMIT }),
        );
    }
    json(
        200,
        json!({ "user": user, "requests": requests, "limit": LIMIT, "visits": visits }),
    )
}

fn json(status: u16, body: serde_json::Value) -> anyhow::Result<Response<Body>> {
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))?)
}
