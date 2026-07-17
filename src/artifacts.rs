//! Vendored DES artifact pages, copied from the k8s-cluster
//! `remote/submodules/discrete-event-system/out/` render output and stored
//! gzip-compressed in this repo (the 31 MB track3t page is ~667 KB gzipped).
//! They are served with `Content-Encoding: gzip` exactly as stored — every
//! page is fully self-contained HTML/JS with zero external requests.

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

pub struct Artifact {
    pub slug: &'static str,
    pub title: &'static str,
    pub blurb: &'static str,
    pub source: &'static str,
    pub plain_size: &'static str,
    pub gz: &'static [u8],
}

pub const ARTIFACTS: &[Artifact] = &[
    Artifact {
        slug: "factory-floor-track3t",
        title: "Track3t factory floor — warehouse comparison",
        blurb: "Full track3t discrete-event animation: competing warehouse floor layouts replayed event by event.",
        source: "discrete-event-system/out/factory-floor-track3t.html",
        plain_size: "31 MB (667 KB over the wire)",
        gz: include_bytes!("../assets/artifacts/factory-floor-track3t.html.gz"),
    },
    Artifact {
        slug: "elevator",
        title: "Elevator high-rise — FEL playback",
        blurb: "High-rise elevator future-event-list simulation with animated car dispatch.",
        source: "discrete-event-system/out/elevator.html",
        plain_size: "2.9 MB (61 KB over the wire)",
        gz: include_bytes!("../assets/artifacts/elevator.html.gz"),
    },
    Artifact {
        slug: "soccer-IP-MIP-feasible-solver",
        title: "Soccer lineup IP — MIP feasible solver trace",
        blurb: "Solver-eye view of the soccer lineup integer program finding feasible rotations.",
        source: "discrete-event-system/out/soccer-IP-MIP-feasible-solver.html",
        plain_size: "87 KB",
        gz: include_bytes!("../assets/artifacts/soccer-IP-MIP-feasible-solver.html.gz"),
    },
    Artifact {
        slug: "soccer-IP-MIP-feasible",
        title: "Soccer lineup IP — MIP feasible search",
        blurb: "Branch-and-bound search animation for the soccer lineup integer program.",
        source: "discrete-event-system/out/soccer-IP-MIP-feasible.html",
        plain_size: "441 KB",
        gz: include_bytes!("../assets/artifacts/soccer-IP-MIP-feasible.html.gz"),
    },
];

pub fn find(slug: &str) -> Option<&'static Artifact> {
    let slug = slug.trim_end_matches(".html");
    ARTIFACTS.iter().find(|a| a.slug.eq_ignore_ascii_case(slug))
}

pub fn serve(artifact: &'static Artifact) -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CONTENT_ENCODING, "gzip"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
            (header::VARY, "Accept-Encoding"),
        ],
        artifact.gz,
    )
        .into_response()
}

pub fn serve_by_slug(slug: &str) -> Response {
    match find(slug) {
        Some(a) => serve(a),
        None => (StatusCode::NOT_FOUND, format!("unknown artifact: {slug}")).into_response(),
    }
}
