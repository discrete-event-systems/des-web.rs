//! des-web — MASH server (maud + axum + supabase + sea-orm + htmx) for the
//! discrete-event-systems org. Serves the sim/game pages copied from
//! ORESoftware/k8s-cluster (see readme.md for the page → upstream map) with
//! their data read from the shared pg-defs Postgres contract.
//!
//! Degraded-mode rule (house pattern): the server always boots and serves
//! every page; missing DB / Supabase / des-rs upstream only disable the
//! sections that need them, with a visible notice.

mod artifacts;
mod auth;
mod catalog;
mod config;
mod db;
mod entities;
mod planner;
mod public_paths;
mod routing;
mod views;

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{DefaultBodyLimit, Path, Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use sea_orm::DatabaseConnection;
use serde_json::json;
use tracing::{info, warn};

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<Config>,
    pub db: Option<DatabaseConnection>,
    pub http: reqwest::Client,
    pub solves: routing::SolveMap,
    pub planner_html: Arc<str>,
}

/// Liveness: the process is up. Always 200 (des-web serves pages even with no
/// DB — see the degraded-mode rule), with a status snapshot in the body.
async fn healthz(State(app): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "ok": true,
        "service": "des-web",
        "publicBasePath": public_paths::PUBLIC_BASE_PATH,
        "db": db::ping(&app.db).await,
        "supabase": app.cfg.supabase().is_some(),
        "desUpstream": app.cfg.des_upstream_url,
    }))
}

/// Readiness: ready to serve its purpose. Ready when no DB is configured
/// (intentional degraded mode) or the configured DB is reachable; 503 only when
/// a DB is configured but unreachable, so an orchestrator can gate traffic.
async fn readyz(State(app): State<AppState>) -> Response {
    let ready = app.db.is_none() || db::ping(&app.db).await;
    let code = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (code, Json(json!({ "ready": ready }))).into_response()
}

/// Baseline security headers on every response. The vendored artifact pages and
/// first-party pages are fully self-contained (inline JS/CSS, no external
/// requests), so a `'self' 'unsafe-inline'` CSP holds without breaking them
/// while still blocking external loads, framing, and base-tag hijacking.
async fn security_headers(req: Request, next: Next) -> Response {
    let mut resp = next.run(req).await;
    let h = resp.headers_mut();
    h.insert(
        "content-security-policy",
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; img-src 'self' data:; \
             connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; \
             form-action 'self'",
        ),
    );
    h.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    h.insert("x-frame-options", HeaderValue::from_static("DENY"));
    h.insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    resp
}

async fn app_css() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=300"),
        ],
        include_str!("../assets/app.css"),
    )
}

async fn htmx_js() -> impl IntoResponse {
    (
        [
            (
                header::CONTENT_TYPE,
                "application/javascript; charset=utf-8",
            ),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        include_str!("../assets/htmx.min.js"),
    )
}

async fn track3t_page() -> Response {
    artifacts::serve_by_slug("factory-floor-track3t")
}

async fn elevator_player_page() -> Response {
    artifacts::serve_by_slug("elevator")
}

async fn artifact_page(Path(slug): Path<String>) -> Response {
    artifacts::serve_by_slug(&slug)
}

fn router(state: AppState) -> Router {
    Router::new()
        // Canonical service-local route taxonomy. The k8s gateway mounts these
        // below /des, while response middleware emits browser-facing /des URLs.
        .route("/models", get(catalog::models_page))
        .route("/models/", get(catalog::models_page))
        .route("/runs/{run_id}", get(catalog::run_page))
        .route("/runs/{run_id}/", get(catalog::run_page))
        .route("/games/soccer", get(views::soccer_page))
        .route("/games/soccer/", get(views::soccer_page))
        .route("/games/soccer/planner", get(planner::planner_page))
        .route("/games/soccer/planner/", get(planner::planner_page))
        .route("/games/soccer/planner/solve", post(planner::proxy_solve))
        .route("/games/soccer/planner/stream", post(planner::proxy_stream))
        .route("/games/elevator", get(views::elevator_page))
        .route("/games/elevator/", get(views::elevator_page))
        .route("/games/elevator/player", get(elevator_player_page))
        .route("/games/elevator/player/", get(elevator_player_page))
        .route("/tools/routing", get(views::routing_page))
        .route("/tools/routing/", get(views::routing_page))
        .route("/labs/factory-floor-track3t", get(track3t_page))
        .route("/labs/factory-floor-track3t/", get(track3t_page))
        .route("/api/v1/catalog", get(catalog::api_catalog))
        .route("/api/v1/solve", post(routing::post_solve))
        .route("/api/v1/solve/{id}", get(routing::get_solve))
        // Home + shared partials. These service-local paths remain stable for
        // direct development and are published below /des by the gateway.
        .route("/", get(views::home))
        .route("/partials/db-status", get(views::partial_db_status))
        .route("/partials/sims", get(views::partial_sims))
        // Legacy service-local aliases retained for compatibility.
        .route("/soccer", get(views::soccer_page))
        .route(
            "/partials/soccer/tournaments",
            get(views::partial_soccer_tournaments),
        )
        .route(
            "/partials/soccer/matches",
            get(views::partial_soccer_matches),
        )
        .route("/partials/soccer/runs", get(views::partial_soccer_runs))
        .route("/soccer/planner", get(planner::planner_page))
        .route("/soccer/planner/solve", post(planner::proxy_solve))
        .route("/soccer/planner/stream", post(planner::proxy_stream))
        .route("/elevator", get(views::elevator_page))
        .route("/elevator/player", get(elevator_player_page))
        .route("/partials/elevator/runs", get(views::partial_elevator_runs))
        .route(
            "/partials/elevator/decisions",
            get(views::partial_elevator_decisions),
        )
        .route("/routing", get(views::routing_page))
        .route("/api/solve", post(routing::post_solve))
        .route("/api/solve/{id}", get(routing::get_solve))
        .route(
            "/partials/routing/solves",
            get(views::partial_routing_solves),
        )
        .route("/track3t", get(track3t_page))
        .route("/artifacts", get(views::artifacts_index))
        .route("/artifacts/", get(views::artifacts_index))
        .route("/artifacts/{slug}", get(artifact_page))
        // Supabase auth.
        .route("/login", get(views::login_page))
        .route("/auth/magic-link", post(auth::magic_link))
        .route("/auth/status", get(auth::status))
        // Plumbing.
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/assets/app.css", get(app_css))
        .route("/assets/htmx.min.js", get(htmx_js))
        .fallback(views::not_found)
        // Cap request bodies (the largest is a planner roster / routing spec —
        // well under this); rejects oversized posts with 413 before handling.
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(middleware::from_fn(public_paths::rewrite_public_paths))
        .layer(middleware::from_fn(security_headers))
        .with_state(state)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("shutdown signal received");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "des_web=info,info".into()),
        )
        .init();

    let cfg = Arc::new(Config::from_env());

    let db = match cfg.database_url.as_deref() {
        Some(url) => match db::connect_lazy(url).await {
            Ok(conn) => {
                info!("postgres pool ready (lazy) — pg-defs contract + des-web overlay");
                Some(conn)
            }
            Err(err) => {
                warn!(%err, "postgres pool init failed; running degraded");
                None
            }
        },
        None => {
            warn!("no database URL (DES_WEB_DATABASE_URL/DATABASE_URL/SUPABASE_DB_URL/RDS_DATABASE_URL/PG_DATABASE_URL); running degraded");
            None
        }
    };

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;

    let state = AppState {
        cfg: cfg.clone(),
        db,
        http,
        solves: Default::default(),
        planner_html: Arc::from(planner::planner_page_html()),
    };

    let addr = format!("{}:{}", cfg.host, cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(
        %addr,
        public_base_path = public_paths::PUBLIC_BASE_PATH,
        "des-web listening — canonical pages: / /models /games/soccer /games/elevator /tools/routing /labs/factory-floor-track3t /artifacts"
    );

    axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}
