//! Supabase GoTrue auth — the S in MASH. Passwordless magic-link login via the
//! project's `/auth/v1/otp` endpoint (same integration shape as athleto-app-rs
//! in k8s-cluster, kept minimal here: request the link, report the outcome as
//! an htmx fragment). Configured with SUPABASE_URL + SUPABASE_ANON_KEY; the
//! login page degrades to a notice when they're unset.
//!
//! Supabase also serves as a Postgres provider for the whole app: point
//! DATABASE_URL (or SUPABASE_DB_URL) at the project's connection string and
//! SeaORM + dpm run against it exactly like RDS. See readme.md.

use axum::extract::State;
use axum::Form;
use axum::Json;
use maud::{html, Markup};
use serde::Deserialize;
use serde_json::json;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct MagicLinkForm {
    pub email: String,
}

pub async fn status(State(app): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "supabaseConfigured": app.cfg.supabase().is_some(),
        "supabaseUrl": app.cfg.supabase_url,
    }))
}

/// POST /auth/magic-link (htmx form target) → result fragment.
pub async fn magic_link(State(app): State<AppState>, Form(form): Form<MagicLinkForm>) -> Markup {
    let email = form.email.trim().to_string();
    if !email.contains('@') || email.len() > 320 {
        return fragment("err", "That doesn't look like an email address.");
    }
    let Some((base, key)) = app.cfg.supabase() else {
        return fragment(
            "err",
            "Supabase is not configured (set SUPABASE_URL and SUPABASE_ANON_KEY).",
        );
    };

    let result = app
        .http
        .post(format!("{base}/auth/v1/otp"))
        .header("apikey", key)
        .bearer_auth(key)
        .json(&json!({ "email": email, "create_user": true }))
        .send()
        .await;

    match result {
        Ok(resp) if resp.status().is_success() => fragment(
            "ok",
            &format!("Magic link sent to {email} — check your inbox."),
        ),
        Ok(resp) => {
            let status = resp.status();
            let detail = resp
                .json::<serde_json::Value>()
                .await
                .ok()
                .and_then(|v| {
                    v.get("msg")
                        .or_else(|| v.get("message"))
                        .or_else(|| v.get("error_description"))
                        .and_then(|m| m.as_str())
                        .map(str::to_string)
                })
                .unwrap_or_else(|| "no detail".to_string());
            fragment("err", &format!("Supabase said {status}: {detail}"))
        }
        Err(err) => fragment("err", &format!("Could not reach Supabase: {err}")),
    }
}

fn fragment(kind: &str, message: &str) -> Markup {
    html! {
        p class={ "auth-result auth-result-" (kind) } { (message) }
    }
}
