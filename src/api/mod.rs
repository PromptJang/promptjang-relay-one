mod error;
mod handlers;
mod state;

pub use state::AppState;

use axum::body::Body;
use axum::http::{HeaderValue, Request, StatusCode, Uri};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post};
use axum::{Router, middleware};
use tower_http::set_header::SetResponseHeaderLayer;
use rust_embed::RustEmbed;

use crate::api::handlers::{
    endpoints::{
        create_endpoint, delete_endpoint, finish_rotation, list_endpoints, rotate_secret,
        test_destination, update_endpoint,
    },
    events::{get_event, list_events, replay_event},
    health::{health, ready, system},
    ingest::ingest,
    keys::{create_key, delete_key, list_keys, reveal_key},
    mail::{acknowledge, claim, delete_mailbox, list_mailboxes, list_messages, nack, push},
};

#[derive(RustEmbed)]
#[folder = "web/dist/"]
struct Assets;

async fn embedded_asset(uri: Uri) -> Response {
    let requested = uri.path().trim_start_matches('/');
    let name = if requested.is_empty() { "index.html" } else { requested };
    let asset = Assets::get(name).or_else(|| Assets::get("index.html"));
    match asset {
        Some(asset) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", mime_guess::from_path(name).first_or_octet_stream().as_ref())
            .body(Body::from(asset.data.into_owned()))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn deprecated(request: Request<axum::body::Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("Deprecation", HeaderValue::from_static("true"));
    response.headers_mut().insert(
        "Link",
        HeaderValue::from_static("</api/v1>; rel=\"successor-version\""),
    );
    response
}

async fn request_id(mut request: Request<axum::body::Body>, next: Next) -> Response {
    let value = request
        .headers()
        .get("X-Request-ID")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty() && value.len() <= 128)
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
    let legacy = Router::new()
        .route("/api/endpoints", get(list_endpoints).post(create_endpoint))
        .route(
            "/api/endpoints/{id}",
            patch(update_endpoint).delete(delete_endpoint),
        )
        .route("/api/keys", get(list_keys).post(create_key))
        .route("/api/keys/{id}", delete(delete_key))
        .route("/api/events", get(list_events))
        .route("/api/events/{id}", get(get_event))
        .route("/api/events/{id}/replay", post(replay_event))
        .route("/e/{endpoint_id}", post(ingest))
        .layer(middleware::from_fn(deprecated));
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/v1/destinations", get(list_endpoints).post(create_endpoint))
        .route("/api/v1/destinations/{id}", patch(update_endpoint).delete(delete_endpoint))
        .route("/api/v1/destinations/{id}/signing-secret/rotate", post(rotate_secret))
        .route("/api/v1/destinations/{id}/signing-secret/previous", delete(finish_rotation))
        .route("/api/v1/destinations/{id}/test", post(test_destination))
        .route("/api/v1/keys", get(list_keys).post(create_key))
        .route("/api/v1/keys/{id}", delete(delete_key))
        .route("/api/v1/keys/{id}/secret", get(reveal_key))
        .route("/api/v1/events", get(list_events))
        .route("/api/v1/events/{id}", get(get_event))
        .route("/api/v1/events/{id}/replay", post(replay_event))
        .route("/api/v1/system", get(system))
        .route("/docs", get(handlers::docs::index))
        .route("/docs/", get(handlers::docs::index))
        .route("/docs/{name}", get(handlers::docs::article))
        .route("/v1/destinations/{endpoint_id}/events", post(ingest))
        .route("/v1/mail/{name}/messages", post(push))
        .route("/v1/mail/{name}/claim", post(claim))
        .route("/v1/mail/{name}/messages/{id}/ack", post(acknowledge))
        .route("/v1/mail/{name}/messages/{id}/nack", post(nack))
        .route("/api/v1/mail", get(list_mailboxes))
        .route("/api/v1/mail/{name}", delete(delete_mailbox))
        .route("/api/v1/mail/{name}/messages", get(list_messages))
        .merge(legacy)
        .fallback(embedded_asset)
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'; connect-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"),
        ))
        .layer(middleware::from_fn(request_id))
        .with_state(state)
}
