mod error;
mod handlers;
mod state;

pub use state::AppState;

use axum::body::Body;
use axum::http::{HeaderValue, Request, StatusCode, Uri};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Router, middleware};
use rust_embed::RustEmbed;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::api::handlers::{
    health::{health, ready, system},
    integrations::{diagnose_mcp, install_mcp, mcp_status},
    keys::{create_key, delete_key, list_keys, reveal_key},
    mail::{
        acknowledge, claim, delete_mailbox, list_mailboxes, list_mailboxes_for_agent,
        list_messages, nack, push,
    },
    updates::check_update,
};

#[derive(RustEmbed)]
#[folder = "web/dist/"]
struct Assets;

async fn embedded_asset(uri: Uri) -> Response {
    let requested = uri.path().trim_start_matches('/');
    let name = if requested.is_empty() {
        "index.html"
    } else {
        requested
    };
    match Assets::get(name).or_else(|| Assets::get("index.html")) {
        Some(asset) => Response::builder()
            .status(StatusCode::OK)
            .header(
                "Content-Type",
                mime_guess::from_path(name).first_or_octet_stream().as_ref(),
            )
            .body(Body::from(asset.data.into_owned()))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn request_id(mut request: Request<Body>, next: Next) -> Response {
    let value = request
        .headers()
        .get("X-Request-ID")
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.is_empty() && v.len() <= 128)
        .map(str::to_string)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    if let Ok(header) = HeaderValue::from_str(&value) {
        request.headers_mut().insert("X-Request-ID", header);
    }
    let mut response = next.run(request).await;
    if let Ok(header) = HeaderValue::from_str(&value) {
        response.headers_mut().insert("X-Request-ID", header);
    }
    response
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/v1/keys", get(list_keys).post(create_key))
        .route("/api/v1/keys/{id}", delete(delete_key))
        .route("/api/v1/keys/{id}/secret", get(reveal_key))
        .route("/api/v1/system", get(system))
        .route("/api/v1/integrations/mcp", get(mcp_status).post(install_mcp))
        .route("/api/v1/integrations/mcp/diagnose", post(diagnose_mcp))
        .route("/api/v1/update", get(check_update))
        .route("/docs", get(handlers::docs::index))
        .route("/docs/", get(handlers::docs::index))
        .route("/docs/{name}", get(handlers::docs::article))
        .route("/v1/mail/{name}/messages", post(push))
        .route("/v1/mailboxes", get(list_mailboxes_for_agent))
        .route("/v1/mail/{name}/claim", post(claim))
        .route("/v1/mail/{name}/messages/{id}/ack", post(acknowledge))
        .route("/v1/mail/{name}/messages/{id}/nack", post(nack))
        .route("/api/v1/mail", get(list_mailboxes))
        .route("/api/v1/mail/{name}", delete(delete_mailbox))
        .route("/api/v1/mail/{name}/messages", get(list_messages))
        .fallback(embedded_asset)
        .layer(SetResponseHeaderLayer::if_not_present(axum::http::header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")))
        .layer(SetResponseHeaderLayer::if_not_present(axum::http::header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY")))
        .layer(SetResponseHeaderLayer::if_not_present(axum::http::header::REFERRER_POLICY, HeaderValue::from_static("no-referrer")))
        .layer(SetResponseHeaderLayer::if_not_present(axum::http::header::CONTENT_SECURITY_POLICY, HeaderValue::from_static("default-src 'self'; connect-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'")))
        .layer(middleware::from_fn(request_id))
        .with_state(state)
}
