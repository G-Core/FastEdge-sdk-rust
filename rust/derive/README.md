# Derive proc macro #[fastedge::main]
## Sample example
```rust
 use fastedge::http::{Error, Request, Response, StatusCode};
 use fastedge::hyper::body::Body;

 #[fastedge::main]
 fn main(req: Request<Body>) -> Result<Response<Body>, Error> {
     Response::builder().status(StatusCode::OK).body(Body::empty())
 }
```