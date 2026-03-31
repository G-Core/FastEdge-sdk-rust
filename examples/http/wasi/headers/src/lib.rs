use fastedge::dictionary;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let custom_env_var = dictionary::get("MY_CUSTOM_ENV_VAR").unwrap_or_default();

    let mut builder = Response::builder().status(200);

    // Copy request headers to response
    for (name, value) in request.headers() {
        builder = builder.header(name.as_str(), value);
    }

    // Add custom header from env var
    builder = builder.header("my-custom-header", &custom_env_var);

    Ok(builder.body(Body::from("Returned all headers with a custom header added"))?)
}
