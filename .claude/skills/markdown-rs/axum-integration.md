# Axum Integration with markdown-rs

Integrate markdown-rs into Axum web applications for rendering Markdown content, building APIs, and creating documentation systems.

## Dependencies

```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
markdown = "1.0.0-alpha.21"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Basic Patterns

### Simple Handler

```rust
use axum::{extract::Json, response::Html};
use markdown::to_html;
use serde::Deserialize;

#[derive(Deserialize)]
struct MarkdownRequest {
    content: String,
}

async fn render_markdown(Json(payload): Json<MarkdownRequest>) -> Html<String> {
    Html(to_html(&payload.content))
}

let app = axum::Router::new()
    .route("/api/render", axum::routing::post(render_markdown));
```

### With GFM Options

```rust
use axum::{extract::Json, response::Html, http::StatusCode};
use markdown::{to_html_with_options, Options};
use serde::Deserialize;

#[derive(Deserialize)]
struct MarkdownRequest {
    content: String,
    gfm: Option<bool>,
}

async fn render_markdown(
    Json(payload): Json<MarkdownRequest>,
) -> Result<Html<String>, (StatusCode, String)> {
    let options = if payload.gfm.unwrap_or(false) {
        Options::gfm()
    } else {
        Options::default()
    };

    to_html_with_options(&payload.content, &options)
        .map(Html)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.reason.to_string()))
}
```

## Stateful Service Pattern

Share pre-configured options across handlers:

```rust
use axum::{extract::{Json, State}, response::Html, Router};
use markdown::{to_html_with_options, Options};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    md_options: Arc<Options>,
}

async fn render_with_state(
    State(state): State<AppState>,
    Json(payload): Json<MarkdownRequest>,
) -> Html<String> {
    let html = to_html_with_options(&payload.content, &state.md_options)
        .unwrap_or_else(|e| format!("<p>Error: {}</p>", e.reason));
    Html(html)
}

#[tokio::main]
async fn main() {
    let state = AppState {
        md_options: Arc::new(Options::gfm()),
    };

    let app = Router::new()
        .route("/api/render", axum::routing::post(render_with_state))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

## Document Caching

For repeated access to the same documents:

```rust
use axum::{extract::{Path, State}, response::Html, Router};
use markdown::{to_html_with_options, Options};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct CachedDoc {
    source: String,
    html: String,
}

#[derive(Clone)]
struct AppState {
    cache: Arc<RwLock<HashMap<String, CachedDoc>>>,
    options: Arc<Options>,
}

async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Html<String>, axum::http::StatusCode> {
    // Check cache first
    {
        let cache = state.cache.read().await;
        if let Some(doc) = cache.get(&id) {
            return Ok(Html(doc.html.clone()));
        }
    }

    // Load from storage (filesystem, database, etc.)
    let source = load_document(&id).await
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)?;

    // Parse and cache
    let html = to_html_with_options(&source, &state.options)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    {
        let mut cache = state.cache.write().await;
        cache.insert(id, CachedDoc { source, html: html.clone() });
    }

    Ok(Html(html))
}

async fn load_document(id: &str) -> Result<String, std::io::Error> {
    tokio::fs::read_to_string(format!("docs/{}.md", id)).await
}
```

## API Response Formats

### JSON Response with Metadata

```rust
use axum::{extract::Json, response::IntoResponse};
use markdown::{to_mdast, to_html_with_options, Options, ParseOptions, mdast::{Node, Heading, Text}};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct RenderRequest {
    content: String,
}

#[derive(Serialize)]
struct RenderResponse {
    html: String,
    title: Option<String>,
    headings: Vec<HeadingInfo>,
    word_count: usize,
}

#[derive(Serialize)]
struct HeadingInfo {
    depth: u8,
    text: String,
}

async fn render_with_metadata(
    Json(payload): Json<RenderRequest>,
) -> Json<RenderResponse> {
    let options = Options::gfm();
    let html = to_html_with_options(&payload.content, &options)
        .unwrap_or_default();

    let ast = to_mdast(&payload.content, &ParseOptions::gfm()).ok();

    let (title, headings) = ast.as_ref()
        .map(|a| extract_headings(a))
        .unwrap_or_default();

    let word_count = payload.content.split_whitespace().count();

    Json(RenderResponse {
        html,
        title,
        headings,
        word_count,
    })
}

fn extract_headings(ast: &Node) -> (Option<String>, Vec<HeadingInfo>) {
    let mut title = None;
    let mut headings = Vec::new();

    fn walk(node: &Node, title: &mut Option<String>, headings: &mut Vec<HeadingInfo>) {
        if let Node::Heading(Heading { depth, children, .. }) = node {
            let text: String = children.iter()
                .filter_map(|c| if let Node::Text(Text { value, .. }) = c { Some(value.as_str()) } else { None })
                .collect();

            if *depth == 1 && title.is_none() {
                *title = Some(text.clone());
            }
            headings.push(HeadingInfo { depth: *depth, text });
        }
        if let Some(children) = node.children() {
            for child in children {
                walk(child, title, headings);
            }
        }
    }

    walk(ast, &mut title, &mut headings);
    (title, headings)
}
```

## Middleware Approach

Process Markdown in middleware for automatic conversion:

```rust
use axum::{
    body::Body,
    extract::Request,
    http::{header, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use markdown::to_html;

async fn markdown_to_html_middleware(
    request: Request,
    next: Next,
) -> Response<Body> {
    let response = next.run(request).await;

    // Only process if content-type is text/markdown
    let is_markdown = response.headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("text/markdown"))
        .unwrap_or(false);

    if !is_markdown {
        return response;
    }

    // Extract body and convert
    let (parts, body) = response.into_parts();
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let md_str = String::from_utf8_lossy(&bytes);
    let html = to_html(&md_str);

    let mut response = Response::from_parts(parts, Body::from(html));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        "text/html; charset=utf-8".parse().unwrap(),
    );
    response
}

// Apply middleware
let app = Router::new()
    .route("/docs/*path", axum::routing::get(serve_markdown_file))
    .layer(axum::middleware::from_fn(markdown_to_html_middleware));
```

## Error Handling

```rust
use axum::{http::StatusCode, response::{IntoResponse, Response}};
use markdown::message::Message;

enum MarkdownError {
    ParseError(Message),
    NotFound,
    Internal(String),
}

impl IntoResponse for MarkdownError {
    fn into_response(self) -> Response {
        match self {
            MarkdownError::ParseError(msg) => {
                let body = format!(
                    "Markdown parse error at line {}, column {}: {}",
                    msg.point.as_ref().map(|p| p.line).unwrap_or(0),
                    msg.point.as_ref().map(|p| p.column).unwrap_or(0),
                    msg.reason
                );
                (StatusCode::BAD_REQUEST, body).into_response()
            }
            MarkdownError::NotFound => StatusCode::NOT_FOUND.into_response(),
            MarkdownError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
        }
    }
}
```

## Complete Example

```rust
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use markdown::{to_html_with_options, Options};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    options: Arc<Options>,
}

#[derive(Deserialize)]
struct RenderRequest {
    content: String,
}

#[derive(Serialize)]
struct RenderResponse {
    html: String,
}

async fn render(
    State(state): State<AppState>,
    Json(payload): Json<RenderRequest>,
) -> Result<Json<RenderResponse>, (StatusCode, String)> {
    let html = to_html_with_options(&payload.content, &state.options)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.reason.to_string()))?;

    Ok(Json(RenderResponse { html }))
}

async fn health() -> impl IntoResponse {
    "OK"
}

#[tokio::main]
async fn main() {
    let state = AppState {
        options: Arc::new(Options::gfm()),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/render", post(render))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
```

## Best Practices

1. **Cache parsed HTML** for documents that don't change frequently
2. **Use `Arc<Options>`** to share configuration without cloning
3. **Return structured errors** with position information
4. **Set request body limits** to prevent memory exhaustion
5. **Consider async file I/O** with `tokio::fs` for document loading
6. **Use content-type headers** to distinguish Markdown from other content
