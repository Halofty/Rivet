#![cfg(feature = "server")]

use dioxus::server::DioxusRouterExt;
use dioxus::server::axum::{
    Extension, Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use rivet::api::BackendStatus;
use rivet::server::AppState;
use serde_json::{Value, json};
use tower::ServiceExt;

// Exercise the generated endpoints without requiring DX's bundled public assets.
// SSR and the production router require a separate DX/browser smoke check.
fn api_router() -> Router {
    Router::new()
        .register_server_functions()
        .layer(Extension(AppState {
            application: "Rivet test",
        }))
        .with_state(dioxus::server::FullstackState::headless())
}

#[tokio::test]
async fn health_endpoint_receives_server_context() {
    let response = api_router()
        .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 4096).await.unwrap();
    let status: BackendStatus = serde_json::from_slice(&body).unwrap();
    assert_eq!(status.application, "Rivet test");
}

#[tokio::test]
async fn echo_endpoint_round_trips_unicode() {
    let response = api_router()
        .oneshot(
            Request::post("/api/echo")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"message": "  안녕, Rivet  "}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 4096).await.unwrap();
    let reply: String = serde_json::from_slice(&body).unwrap();
    assert_eq!(reply, "Rivet test received: 안녕, Rivet");
}

#[tokio::test]
async fn invalid_echo_returns_a_structured_bad_request() {
    let response = api_router()
        .oneshot(
            Request::post("/api/echo")
                .header("content-type", "application/json")
                .body(Body::from(json!({"message": "   "}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), 4096).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(error["code"], 400);
    assert_eq!(
        error["data"]["Validation"],
        "Enter a message between 1 and 200 characters."
    );
}
