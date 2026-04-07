[← Back to examples](../../README.md)

# Custom Error Pages (CDN)

Intercepts 4xx and 5xx error responses and replaces them with branded HTML error pages using Handlebars templates.

## How it works

A [build script](./build.rs) runs at compile time to embed images and messages from the `public/` folder into the WASM binary (since there is no filesystem at runtime).

At runtime, when an error response is detected:
1. **on_response_headers** — sets `Content-Type` to `text/html` for error responses
2. **on_response_body** — looks up the status code in the embedded image/message maps, falls back to generic `4xx`/`5xx` templates, and renders the error page using Handlebars

## Adding a custom error page

1. Add an image: `public/images/<status>.jpg`
2. Add a message file: `public/messages/<status>.hbs` (first line = title, second line = description)
3. Recompile and deploy

## Styling

Styles are in [`public/styles.css`](./public/styles.css) — plain CSS, no build tools required. Edit directly.
