//! The soccer rotation planner page, copied from
//! soccer-sim-game-engine.rs/src/des/soccer_planner/planner_ui.html (the page
//! des-rs serves at /soccer/planner). The page is fully self-contained; only
//! its solve/stream endpoints need the engine. When DES_UPSTREAM_URL points at
//! a running des-rs instance those endpoints proxy through, so the copied page
//! does live solves against the real engine; otherwise they return 503 with a
//! clear hint and the page still renders (editable config, docs, defaults).

use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use serde_json::json;

use crate::AppState;

/// Faithful port of the engine's `default_planner_request()` (18 players,
/// 4-4-2 + GK, plausible best-position scores) so the vendored page's config
/// editor starts from the same squad as upstream.
fn default_planner_config() -> serde_json::Value {
    let num_positions = 11usize;
    let players: Vec<serde_json::Value> = (0..18usize)
        .map(|p| {
            let mut scores = vec![0.35f64; num_positions];
            let best = p % num_positions;
            scores[best] = 0.88;
            if best + 1 < num_positions {
                scores[best + 1] = 0.62;
            }
            if best > 0 {
                scores[best - 1] = 0.58;
            }
            if p == 0 {
                scores.iter_mut().for_each(|s| *s = 0.2);
                scores[0] = 0.95;
            }
            json!({
                "id": p,
                "name": format!("Player {}", p + 1),
                "status": "available",
                "positionScores": scores,
                "bannedPositions": [],
                "fixedPosition": null,
            })
        })
        .collect();

    json!({
        "outfieldFormation": [4, 4, 2],
        "numPeriods": 2,
        "minutesPerPeriod": 45,
        "maxSubsPerGame": 119,
        "minSubsPerGame": 0,
        "defaultMinContiguousBlocks": 1,
        "defaultMaxContiguousBlocks": 4,
        "defaultMaxBenchBlocks": 3,
        "players": players,
        "synergies": [],
        "seed": 4242,
        "solverTimeLimitMs": 120000.0,
        "solverMaxNodes": 20000,
        "solverMaxTicks": 200000,
        "solverLpMaxIters": 6000,
        "solverHeuristicPasses": 120,
        "fallbackToMdp": true,
    })
}

/// Assemble the page once at startup (same placeholder contract as the
/// engine's `planner_page_html()`).
pub fn planner_page_html() -> String {
    let default_json = serde_json::to_string_pretty(&default_planner_config())
        .unwrap_or_else(|_| "{}".to_string());
    let default_escaped = default_json.replace("</script", "<\\/script");

    include_str!("../assets/soccer-planner.html")
        .replace("__DEFAULT_CONFIG__", &default_escaped)
        .replace("__PLANNER_VERSION__", "des-web (vendored)")
        .replace("__PLANNER_GIT_COMMIT_SHORT__", "vendored")
        .replace(
            "__PLANNER_GIT_COMMIT__",
            "vendored from soccer-sim-game-engine.rs",
        )
}

pub async fn planner_page(State(app): State<AppState>) -> Html<String> {
    Html(app.planner_html.to_string())
}

pub async fn proxy_solve(State(app): State<AppState>, body: Bytes) -> Response {
    proxy(app, "/soccer/planner/solve", body).await
}

pub async fn proxy_stream(State(app): State<AppState>, body: Bytes) -> Response {
    proxy(app, "/soccer/planner/stream", body).await
}

async fn proxy(app: AppState, path: &str, body: Bytes) -> Response {
    let Some(base) = app.cfg.des_upstream_url.as_deref() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "live solves are not enabled on this copy",
                "hint": "set DES_UPSTREAM_URL to a running des-rs instance (k8s-cluster) to proxy solves to the real engine",
            })),
        )
            .into_response();
    };

    let upstream = format!("{base}{path}");
    match app
        .http
        .post(&upstream)
        .header(header::CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .await
    {
        Ok(resp) => {
            let status =
                StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            let content_type = resp
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/json")
                .to_string();
            let mut response = Response::builder()
                .status(status)
                .header(header::CONTENT_TYPE, content_type)
                .body(Body::from_stream(resp.bytes_stream()))
                .unwrap_or_else(|_| StatusCode::BAD_GATEWAY.into_response());
            response
                .headers_mut()
                .insert("x-des-web-proxied-to", upstream.parse().unwrap());
            response
        }
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "error": format!("proxy to des-rs failed: {err}"),
                "upstream": upstream,
            })),
        )
            .into_response(),
    }
}
