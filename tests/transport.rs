#![cfg(feature = "server")]

use dioxus::server::DioxusRouterExt;
use dioxus::server::axum::{
    Router,
    body::{Body, to_bytes},
    http::{HeaderMap, Method, Request, StatusCode, header},
};
use rivet::server::{AppState, auth::Auth, config::ServerConfig, db};
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::ServiceExt;

const PASSWORD: &str = "correct horse battery";

/// Exercises the generated endpoints and production layers without DX's bundled assets.
/// SSR and the full production router require a separate DX/browser smoke check.
struct TestServer {
    router: Router,
    _dir: TempDir,
}

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: Value,
}

impl TestServer {
    async fn new(config: ServerConfig) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let pool = db::open(&dir.path().join("rivet.sqlite3")).await.unwrap();
        // Cheap Argon2 parameters keep tests fast; production uses the defaults.
        let auth = Auth::with_params(argon2::Params::new(8, 1, 1, None).unwrap());
        let api = Router::new()
            .register_server_functions()
            .with_state(dioxus::server::FullstackState::headless());
        let router = rivet::server::with_layers(api, AppState::new(pool, config, auth));
        Self { router, _dir: dir }
    }

    async fn local() -> Self {
        Self::new(ServerConfig::local()).await
    }

    async fn send(&self, request: Request<Body>) -> Reply {
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
        let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        Reply {
            status,
            headers,
            body,
        }
    }

    async fn call(&self, method: Method, path: &str, body: Value, extra: &[(&str, &str)]) -> Reply {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::HOST, "127.0.0.1:8080")
            .header(header::CONTENT_TYPE, "application/json");
        for (name, value) in extra {
            request = request.header(*name, *value);
        }
        let body = if body.is_null() {
            Body::empty()
        } else {
            Body::from(body.to_string())
        };
        self.send(request.body(body).unwrap()).await
    }

    async fn post(&self, path: &str, body: Value, extra: &[(&str, &str)]) -> Reply {
        self.call(Method::POST, path, body, extra).await
    }

    async fn get(&self, path: &str, extra: &[(&str, &str)]) -> Reply {
        self.call(Method::GET, path, Value::Null, extra).await
    }

    async fn register(&self, email: &str, client: &str) -> Reply {
        self.post(
            "/api/auth/register",
            credentials(email, PASSWORD, client),
            &[],
        )
        .await
    }
}

fn credentials(email: &str, password: &str, client: &str) -> Value {
    json!({ "credentials": { "email": email, "password": password, "client": client } })
}

/// `name=value` from a `Set-Cookie` header, ready to send back as `Cookie`.
fn session_cookie(reply: &Reply) -> String {
    let set_cookie = reply.headers[header::SET_COOKIE].to_str().unwrap();
    set_cookie.split(';').next().unwrap().to_owned()
}

#[tokio::test]
async fn health_is_public_and_responses_carry_security_headers() {
    let server = TestServer::local().await;
    let reply = server.get("/api/health", &[]).await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["application"], "Rivet");
    assert_eq!(reply.headers["x-content-type-options"], "nosniff");
    assert_eq!(reply.headers["x-frame-options"], "DENY");
    assert_eq!(reply.headers["referrer-policy"], "same-origin");
}

#[tokio::test]
async fn unknown_host_names_are_rejected() {
    let server = TestServer::local().await;
    let request = Request::get("/api/health")
        .header(header::HOST, "attacker.example")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        server.send(request).await.status,
        StatusCode::MISDIRECTED_REQUEST
    );

    let missing_host = Request::get("/api/health").body(Body::empty()).unwrap();
    assert_eq!(
        server.send(missing_host).await.status,
        StatusCode::MISDIRECTED_REQUEST
    );
}

#[tokio::test]
async fn cross_origin_browser_writes_are_blocked() {
    let server = TestServer::local().await;
    server.register("me@example.com", "web").await;
    let body = credentials("me@example.com", PASSWORD, "web");

    let foreign = server
        .post(
            "/api/auth/login",
            body.clone(),
            &[("origin", "https://attacker.example")],
        )
        .await;
    assert_eq!(foreign.status, StatusCode::FORBIDDEN);
    assert!(!foreign.headers.contains_key(header::SET_COOKIE));

    let same_site = server
        .post(
            "/api/auth/login",
            body,
            &[("origin", "http://127.0.0.1:8080")],
        )
        .await;
    assert_eq!(same_site.status, StatusCode::OK);
}

#[tokio::test]
async fn protected_endpoints_require_a_session() {
    let server = TestServer::local().await;
    let reply = server
        .post("/api/echo", json!({ "message": "hello" }), &[])
        .await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert_eq!(reply.body["data"], "Unauthorized");

    let forged = server
        .post(
            "/api/echo",
            json!({ "message": "hello" }),
            &[("cookie", &format!("rivet_session={}", "ab".repeat(32)))],
        )
        .await;
    assert_eq!(forged.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn first_web_account_receives_an_http_only_cookie_session() {
    let server = TestServer::local().await;
    let status = server.get("/api/auth/status", &[]).await;
    assert_eq!(
        status.body,
        json!({ "user": null, "registration_open": true })
    );

    let reply = server.register("  Me@Example.com ", "web").await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["user"]["email"], "me@example.com");
    assert!(
        reply.body["token"].is_null(),
        "web tokens stay in the cookie"
    );
    let set_cookie = reply.headers[header::SET_COOKIE].to_str().unwrap();
    assert!(set_cookie.contains("HttpOnly") && set_cookie.contains("SameSite=Lax"));
    let cookie = session_cookie(&reply);

    let status = server.get("/api/auth/status", &[("cookie", &cookie)]).await;
    assert_eq!(status.body["user"]["email"], "me@example.com");
    assert_eq!(status.body["registration_open"], false);

    let echo = server
        .post(
            "/api/echo",
            json!({ "message": "  안녕, Rivet  " }),
            &[("cookie", &cookie)],
        )
        .await;
    assert_eq!(echo.status, StatusCode::OK);
    assert_eq!(echo.body, "Rivet received: 안녕, Rivet");

    let invalid = server
        .post(
            "/api/echo",
            json!({ "message": "   " }),
            &[("cookie", &cookie)],
        )
        .await;
    assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
    assert_eq!(invalid.body["code"], 400);
    assert_eq!(
        invalid.body["data"]["Validation"],
        json!({ "field": "input", "message": "Enter a message between 1 and 200 characters." })
    );

    let logout = server
        .post("/api/auth/logout", Value::Null, &[("cookie", &cookie)])
        .await;
    assert_eq!(logout.status, StatusCode::OK);
    assert!(
        logout.headers[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .contains("Max-Age=0")
    );
    let after = server
        .post(
            "/api/echo",
            json!({ "message": "hi" }),
            &[("cookie", &cookie)],
        )
        .await;
    assert_eq!(after.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn weak_passwords_and_bad_emails_are_rejected_before_creating_an_account() {
    let server = TestServer::local().await;
    let weak = server
        .post(
            "/api/auth/register",
            credentials("me@example.com", "short", "web"),
            &[],
        )
        .await;
    assert_eq!(weak.status, StatusCode::BAD_REQUEST);
    assert_eq!(weak.body["data"]["Validation"]["field"], "password");

    let email = server.register("not-an-email", "web").await;
    assert_eq!(email.body["data"]["Validation"]["field"], "email");

    let status = server.get("/api/auth/status", &[]).await;
    assert_eq!(status.body["registration_open"], true);
}

#[tokio::test]
async fn registration_closes_after_setup_and_is_never_open_on_the_public_host() {
    let config = ServerConfig::local()
        .with_public_origin("https://rivet.example.com")
        .unwrap();
    let server = TestServer::new(config).await;

    let public = Request::post("/api/auth/register")
        .header(header::HOST, "rivet.example.com")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            credentials("attacker@example.com", PASSWORD, "web").to_string(),
        ))
        .unwrap();
    let public = server.send(public).await;
    assert_eq!(public.status, StatusCode::FORBIDDEN);
    assert_eq!(public.body["data"], "RegistrationClosed");

    // A proxy that rewrites Host to the loopback upstream must not reopen setup.
    let proxied = server
        .post(
            "/api/auth/register",
            credentials("attacker@example.com", PASSWORD, "web"),
            &[("x-forwarded-for", "203.0.113.7")],
        )
        .await;
    assert_eq!(proxied.status, StatusCode::FORBIDDEN);
    let status = server
        .get("/api/auth/status", &[("x-forwarded-for", "203.0.113.7")])
        .await;
    assert_eq!(status.body["registration_open"], false);

    let setup = server.register("me@example.com", "web").await;
    assert_eq!(setup.status, StatusCode::OK);
    assert!(
        setup.headers[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .contains("Secure"),
        "HTTPS deployments mark the cookie Secure"
    );

    let second = server.register("other@example.com", "web").await;
    assert_eq!(second.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn desktop_sign_in_returns_a_bearer_token_that_logout_revokes() {
    let server = TestServer::local().await;
    server.register("me@example.com", "web").await;

    let login = server
        .post(
            "/api/auth/login",
            credentials("ME@example.com", PASSWORD, "desktop"),
            &[],
        )
        .await;
    assert_eq!(login.status, StatusCode::OK);
    assert!(!login.headers.contains_key(header::SET_COOKIE));
    let bearer = format!("Bearer {}", login.body["token"].as_str().unwrap());

    let status = server
        .get("/api/auth/status", &[("authorization", &bearer)])
        .await;
    assert_eq!(status.body["user"]["email"], "me@example.com");

    let logout = server
        .post(
            "/api/auth/logout",
            Value::Null,
            &[("authorization", &bearer)],
        )
        .await;
    assert_eq!(logout.status, StatusCode::OK);
    let revoked = server
        .post(
            "/api/echo",
            json!({ "message": "hi" }),
            &[("authorization", &bearer)],
        )
        .await;
    assert_eq!(revoked.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn repeated_failed_sign_ins_are_throttled_per_account() {
    let server = TestServer::local().await;
    server.register("me@example.com", "web").await;

    let unknown = server
        .post(
            "/api/auth/login",
            credentials("nobody@example.com", PASSWORD, "web"),
            &[],
        )
        .await;
    assert_eq!(unknown.status, StatusCode::UNAUTHORIZED);
    assert_eq!(unknown.body["data"], "InvalidCredentials");

    for _ in 0..5 {
        let wrong = server
            .post(
                "/api/auth/login",
                credentials("me@example.com", "wrong password!", "web"),
                &[],
            )
            .await;
        assert_eq!(wrong.body["data"], "InvalidCredentials");
    }
    let limited = server
        .post(
            "/api/auth/login",
            credentials("me@example.com", PASSWORD, "web"),
            &[],
        )
        .await;
    assert_eq!(limited.status, StatusCode::TOO_MANY_REQUESTS);
}
