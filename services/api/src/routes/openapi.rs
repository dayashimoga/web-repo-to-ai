//! OpenAPI 3.1 specification and interactive documentation routes.

use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};

/// Embedded OpenAPI 3.1.0 specification in JSON format.
pub const OPENAPI_JSON: &str = include_str!("../../../../docs/openapi.json");

/// Embedded OpenAPI 3.1.0 specification in YAML format.
pub const OPENAPI_YAML: &str = include_str!("../../../../docs/openapi.yaml");

/// Handler returning the OpenAPI 3.1 specification as JSON.
pub async fn openapi_json_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/json; charset=utf-8")],
        OPENAPI_JSON,
    )
}

/// Handler returning the OpenAPI 3.1 specification as YAML.
pub async fn openapi_yaml_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/yaml; charset=utf-8")],
        OPENAPI_YAML,
    )
}

/// Interactive API reference documentation HTML viewer.
pub async fn openapi_docs_handler() -> impl IntoResponse {
    let html = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>web-repo-to-ai REST API Reference</title>
    <meta name="description" content="Universal URL & GitHub Repository to AI-Ready Content Converter API reference documentation" />
    <link rel="icon" type="image/svg+xml" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><text y='.9em' font-size='90'>⚡</text></svg>" />
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@scalar/api-reference@1.25.11/dist/browser/standalone.css" />
    <style>
      :root {
        --theme-font: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
      }
      body {
        margin: 0;
        padding: 0;
        background: #0f172a;
        color: #f8fafc;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      }
    </style>
  </head>
  <body>
    <script
      id="api-reference"
      data-url="/api/v1/openapi.json"
      data-configuration='{"theme":"deepSpace","layout":"modern","showSidebar":true}'
      src="https://cdn.jsdelivr.net/npm/@scalar/api-reference@1.25.11/dist/browser/standalone.js"
    ></script>
    <noscript>
      <div style="padding: 2rem; max-width: 800px; margin: 0 auto; line-height: 1.6;">
        <h1>web-repo-to-ai REST API</h1>
        <p>JavaScript is disabled. You can view the raw OpenAPI 3.1 specifications below:</p>
        <ul>
          <li><a href="/api/v1/openapi.json" style="color: #38bdf8;">OpenAPI 3.1 Specification (JSON)</a></li>
          <li><a href="/api/v1/openapi.yaml" style="color: #38bdf8;">OpenAPI 3.1 Specification (YAML)</a></li>
        </ul>
      </div>
    </noscript>
  </body>
</html>"#;

    Html(html)
}
