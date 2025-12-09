# Integrating `markdown-rs` with Tower/Axum/Hyper

Examples of how to configure and use the `markdown-rs` crate within a Tower/Axum/Hyper architecture. These examples demonstrate different integration patterns, including middleware, request handlers, and state management.

## Basic Setup: Dependencies

Ensure your `Cargo.toml` includes the necessary dependencies:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
axum = "0.8"
tower = "0.5"
tower-http = { version = "0.6", features = ["trace"] }
markdown = "1.0.0-alpha.16"  # Use the latest version of markdown-rs
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

## Integration Patterns

### Example 1: Middleware for Markdown Processing

Create a Tower middleware that automatically processes Markdown in request bodies and converts it to HTML.

```rust
use axum::{
    body::Body,
    extract::Request,
    http::{StatusCode, Response},
    middleware::Next,
    response::IntoResponse,
};
use std::convert::Infallible;
use tower::Layer;
use markdown::{to_html, Options};

// Middleware to convert Markdown to HTML
async fn markdown_middleware<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, Infallible> {
    let (mut parts, body) = req.into_parts();

    // Read the request body (assuming it's Markdown)
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    let md_str = String::from_utf8(bytes.to_vec()).unwrap();

    // Convert Markdown to HTML
    let html = to_html_with_options(
        &md_str,
        &Options {
            parse: Default::default(),
            render: Default::default(),
        },
    );

    // Replace the body with the HTML
    let new_body = Body::from(html);
    let new_req = Request::from_parts(parts, new_body);

    next.run(new_req).await
}

// Apply the middleware to Axum
let app = axum::Router::new()
    .route("/render", axum::routing::post(render_handler))
    .layer(axum::middleware::from_fn(markdown_middleware));

async fn render_handler(body: String) -> impl IntoResponse {
    body // This will already be HTML due to middleware
}
```

### Example 2: Request Handler for Markdown Rendering

Directly use `markdown-rs` in an Axum handler to process Markdown payloads.

```rust
use axum::{extract::Json, response::Html, http::StatusCode};
use serde::Deserialize;
use markdown::to_html;

#[derive(Deserialize)]
struct MarkdownRequest {
    content: String,
}

async fn render_markdown(
    Json(payload): Json<MarkdownRequest>,
) -> Result<Html<String>, StatusCode> {
    let html = to_html(&payload.content);
    Ok(Html(html))
}

let app = axum::Router::new()
    .route("/api/render", axum::routing::post(render_markdown));
```

### Example 3: Stateful Markdown Service

Share a pre-configured `markdown::Options` instance across handlers using Axum state.

```rust
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
};
use markdown::{to_html_with_options, Options};
use std::sync::Arc;

// Shared state for Markdown options
#[derive(Clone)]
struct AppState {
    md_options: Arc<Options>,
}

// Initialize state with custom Markdown options
let state = AppState {
    md_options: Arc::new(Options {
        parse: Default::default(),
        render: Default::default(),
    }),
};

async fn render_with_state(
    State(state): State<AppState>,
    axum::extract::Json(payload): axum::extract::Json<MarkdownRequest>,
) -> impl IntoResponse {
    let html = to_html_with_options(&payload.content, &state.md_options);
    axum::response::Html(html)
}

let app = axum::Router::new()
    .route("/api/render", axum::routing::post(render_with_state))
    .with_state(state);
```

## Advanced Integration with Tower Service

Implement a custom Tower `Service` that wraps another service and processes Markdown responses.

```rust
use tower::{Service, Layer};
use std::future::Future;
use std::pin::Pin;
use markdown::to_html;

// Middleware service that converts Markdown responses to HTML
struct MarkdownService<S> {
    inner: S,
}

impl<S, Request> Service<Request> for MarkdownService<S>
where
    S: Service<Request> + Clone,
    S::Response: Into<axum::body::Body>,
    S::Error: Into<axum::BoxError>,
{
    type Response = axum::response::Response;
    type Error = axum::BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let inner = self.inner.clone();
        Box::pin(async move {
            let response = inner.call(req).await.map_err(Into::into)?;
            let (parts, body) = response.into_parts();
            let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
            let md_str = String::from_utf8(bytes.to_vec()).unwrap();
            let html = to_html(&md_str);
            Ok(axum::response::Response::from_parts(parts, axum::body::Body::from(html)))
        })
    }
}

// Layer to apply the service
struct MarkdownLayer;

impl<S> Layer<S> for MarkdownLayer {
    type Service = MarkdownService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MarkdownService { inner }
    }
}

// Apply the layer
let app = axum::Router::new()
    .route("/markdown", axum::routing::get(|| async { "# Hello Markdown" }))
    .layer(MarkdownLayer);
```

## Comparison of Integration Approaches

| **Approach**       | **Use Case**                          | **Complexity** | **Flexibility** |
|---------------------|---------------------------------------|----------------|-----------------|
| **Middleware**       | Global Markdown processing               | Medium         | High            |
| **Handler**          | Per-endpoint processing                 | Low            | Medium          |
| **Stateful Service** | Shared configuration across handlers    | Medium         | High            |
| **Tower Service**    | Deep integration with Tower ecosystem    | High           | Very High       |

## Best Practices

1. **Error Handling**: Always validate Markdown input and handle parsing errors gracefully.
2. **Performance**: For large Markdown content, consider streaming parsing (if supported by `markdown-rs`).
3. **Security**: Sanitize HTML output if rendering untrusted Markdown to prevent XSS attacks.
4. **Caching**: Cache parsed HTML to avoid reprocessing the same Markdown content.

## Full Example with Axum and Hyper

```rust
use axum::{routing::post, Json, Router, ServiceExt};
use markdown::to_html;
use serde_json::{json, Value};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/markdown", post(render_markdown))
        .into_make_service();

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn render_markdown(Json(payload): Json<Value>) -> Json<Value> {
    let md = payload["content"].as_str().unwrap_or("");
    let html = to_html(md);
    Json(json!({ "html": html }))
}
```

## Additional Resources

- [`markdown-rs` Documentation](https://docs.rs/markdown)
- [Axum Guides](https://docs.rs/axum/latest/axum/)
- [Tower Middleware Patterns](https://github.com/tower-rs/tower/blob/master/guides/building-a-middleware-from-scratch.md)
