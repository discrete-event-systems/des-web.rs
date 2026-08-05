//! Public-route compatibility for mounting this service below `/des`.
//!
//! The application keeps its historical service-local routes so direct local
//! development and old in-cluster callers continue to work. Browser-facing
//! HTML and redirects are rewritten to the canonical public route taxonomy.

use axum::body::{to_bytes, Body};
use axum::extract::Request;
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

pub const PUBLIC_BASE_PATH: &str = "/des";
const MAX_HTML_BYTES: usize = 8 * 1024 * 1024;

/// Ordered longest/specific-first because several service-local and legacy
/// paths overlap. Canonical service-local aliases come first; historical routes
/// remain so direct callers can migrate independently.
const ROUTE_REWRITES: &[(&str, &str)] = &[
    ("/games/soccer/planner", "/des/games/soccer/planner"),
    ("/games/elevator/player", "/des/games/elevator/player"),
    (
        "/labs/factory-floor-track3t",
        "/des/labs/factory-floor-track3t",
    ),
    ("/games/soccer", "/des/games/soccer"),
    ("/games/elevator", "/des/games/elevator"),
    ("/tools/routing", "/des/tools/routing"),
    ("/api/v1", "/des/api/v1"),
    ("/models", "/des/models"),
    ("/runs", "/des/runs"),
    ("/soccer/planner", "/des/games/soccer/planner"),
    ("/elevator/player", "/des/games/elevator/player"),
    ("/api/solve", "/des/api/v1/solve"),
    ("/soccer", "/des/games/soccer"),
    ("/routing", "/des/tools/routing"),
    ("/track3t", "/des/labs/factory-floor-track3t"),
    ("/elevator", "/des/games/elevator"),
    ("/artifacts", "/des/artifacts"),
    ("/login", "/des/login"),
    ("/auth", "/des/auth"),
    ("/partials", "/des/partials"),
    ("/assets", "/des/assets"),
];

fn has_path_prefix(value: &str, prefix: &str) -> bool {
    value == prefix
        || value.strip_prefix(prefix).is_some_and(|rest| {
            rest.starts_with('/') || rest.starts_with('?') || rest.starts_with('#')
        })
}

fn is_mounted_below_des(forwarded_prefix: Option<&str>, configured_mode: Option<&str>) -> bool {
    forwarded_prefix.is_some_and(|value| value.trim_end_matches('/') == PUBLIC_BASE_PATH)
        || configured_mode.is_some_and(|value| value.trim().eq_ignore_ascii_case("mounted"))
}

/// Convert a service-local/legacy path to the public `/des` route contract.
/// Absolute external URLs and already-canonical paths are intentionally left
/// alone. The returned value includes any query string or fragment unchanged.
pub fn canonical_public_path(value: &str) -> Option<String> {
    if !value.starts_with('/') || has_path_prefix(value, PUBLIC_BASE_PATH) {
        return None;
    }
    if value == "/" {
        return Some(format!("{PUBLIC_BASE_PATH}/"));
    }
    for (service_local, canonical) in ROUTE_REWRITES {
        if has_path_prefix(value, service_local) {
            return Some(format!("{canonical}{}", &value[service_local.len()..]));
        }
    }
    None
}

fn replace_quoted_path(mut html: String, service_local: &str, canonical: &str) -> String {
    for quote in ['"', '\'', '`'] {
        let from = format!("{quote}{service_local}");
        let to = format!("{quote}{canonical}");
        html = html.replace(&from, &to);
    }
    html
}

/// Rewrite links, htmx attributes, forms, and JavaScript string literals in
/// first-party HTML. Vendored gzip artifacts are skipped by the middleware.
fn rewrite_html(input: &str) -> String {
    let mut html = input.to_owned();
    for (service_local, canonical) in ROUTE_REWRITES {
        html = replace_quoted_path(html, service_local, canonical);
    }

    // The catalog brand/home links are the only exact root links. Restrict the
    // replacement to common URL-bearing forms so ordinary prose is untouched.
    for attribute in ["href", "src", "action", "hx-get", "hx-post"] {
        html = html.replace(
            &format!(r#"{attribute}="/""#),
            &format!(r#"{attribute}="{PUBLIC_BASE_PATH}/""#),
        );
        html = html.replace(
            &format!(r#"{attribute}='/'"#),
            &format!(r#"{attribute}='{PUBLIC_BASE_PATH}/'"#),
        );
    }
    html
}

/// Response middleware that keeps direct service-local routes stable while
/// publishing one canonical browser namespace below `/des`.
///
/// A trusted reverse proxy may signal the mount with `X-Forwarded-Prefix`.
/// Kubernetes sets `DES_PUBLIC_PATH_MODE=mounted` as an explicit fallback so
/// link generation does not depend on one proxy implementation detail.
pub async fn rewrite_public_paths(request: Request, next: Next) -> Response {
    let forwarded_prefix = request
        .headers()
        .get("x-forwarded-prefix")
        .and_then(|value| value.to_str().ok());
    let configured_mode = std::env::var("DES_PUBLIC_PATH_MODE").ok();
    let mounted_below_des = is_mounted_below_des(forwarded_prefix, configured_mode.as_deref());

    let mut response = next.run(request).await;
    if !mounted_below_des {
        return response;
    }

    let location = response
        .headers()
        .get(header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    if let Some(location) = location {
        if let Some(canonical) = canonical_public_path(&location) {
            if let Ok(value) = HeaderValue::from_str(&canonical) {
                response.headers_mut().insert(header::LOCATION, value);
            }
        }
    }

    let is_html = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("text/html"));
    let is_encoded = response.headers().contains_key(header::CONTENT_ENCODING);
    if !is_html || is_encoded {
        return response;
    }

    let (mut parts, body) = response.into_parts();
    let bytes = match to_bytes(body, MAX_HTML_BYTES).await {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::error!(%error, "failed to buffer des-web HTML for public-path rewriting");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to render DES page",
            )
                .into_response();
        }
    };

    let text = match String::from_utf8(bytes.to_vec()) {
        Ok(text) => text,
        Err(error) => {
            tracing::error!(%error, "des-web emitted non-UTF-8 HTML");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to render DES page",
            )
                .into_response();
        }
    };

    parts.headers.remove(header::CONTENT_LENGTH);
    Response::from_parts(parts, Body::from(rewrite_html(&text)))
}

#[cfg(test)]
mod tests {
    use super::{canonical_public_path, is_mounted_below_des, rewrite_html};

    #[test]
    fn maps_service_local_and_legacy_routes_to_the_canonical_taxonomy() {
        assert_eq!(
            canonical_public_path("/games/soccer/planner?seed=42").as_deref(),
            Some("/des/games/soccer/planner?seed=42")
        );
        assert_eq!(
            canonical_public_path("/routing#latest").as_deref(),
            Some("/des/tools/routing#latest")
        );
        assert_eq!(
            canonical_public_path("/api/v1/solve/abc").as_deref(),
            Some("/des/api/v1/solve/abc")
        );
        assert_eq!(
            canonical_public_path("/api/solve/abc").as_deref(),
            Some("/des/api/v1/solve/abc")
        );
    }

    #[test]
    fn leaves_external_and_canonical_paths_unchanged() {
        assert_eq!(canonical_public_path("/des/games/soccer"), None);
        assert_eq!(canonical_public_path("https://example.com/soccer"), None);
    }

    #[test]
    fn mount_detection_accepts_proxy_header_or_explicit_deployment_mode() {
        assert!(is_mounted_below_des(Some("/des"), None));
        assert!(is_mounted_below_des(Some("/des/"), None));
        assert!(is_mounted_below_des(None, Some("mounted")));
        assert!(is_mounted_below_des(None, Some("MOUNTED")));
        assert!(!is_mounted_below_des(None, None));
        assert!(!is_mounted_below_des(Some("/other"), Some("local")));
    }

    #[test]
    fn rewrites_html_attributes_and_javascript_literals() {
        let input = r#"<a href="/games/soccer">game</a><script>fetch('/api/v1/solve')</script><link href="/assets/app.css"><a href="/">home</a>"#;
        let output = rewrite_html(input);
        assert!(output.contains("/des/games/soccer"));
        assert!(output.contains("/des/api/v1/solve"));
        assert!(output.contains("/des/assets/app.css"));
        assert!(output.contains("href=\"/des/\""));
    }
}
