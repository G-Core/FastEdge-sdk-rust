use anyhow::Result;
use fastedge::body::Body;
use fastedge::dictionary;
use fastedge::http::{Request, Response, StatusCode};

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    let custom_env_var = dictionary::get("MY_CUSTOM_ENV_VAR").unwrap_or_default();

    let mut builder = Response::builder().status(StatusCode::OK);

    // Copy request headers to response
    for (name, value) in req.headers() {
        builder = builder.header(name.as_str(), value);
    }

    // Add custom header from env var
    builder = builder.header("x-my-custom-header", &custom_env_var);

    builder
        .body(Body::from(
            "Returned all headers with a custom header added",
        ))
        .map_err(Into::into)
}
