//! Canonical catalog/model/run landing pages and the machine-readable route
//! contract. These pages intentionally stay lightweight: detailed data remains
//! in the existing soccer, elevator, routing, and artifact handlers.

use axum::extract::Path;
use axum::Json;
use maud::{html, Markup};
use serde_json::{json, Value};

use crate::views;

pub async fn models_page() -> Markup {
    views::layout(
        "Models",
        "/models",
        html! {
            h1 { "DES models" }
            p class="lede" {
                "One catalog for the model families served by the discrete-event-systems organization. "
                "Interactive pages, solver tools, and generated artifacts share the canonical "
                code { "/des" } " route namespace."
            }
            div class="cards" {
                a class="card" href="/des/games/soccer" {
                    h3 { "Soccer" }
                    p { "Tournament learning, rotation planning, MIP/MDP solves, and solver traces." }
                }
                a class="card" href="/des/games/elevator" {
                    h3 { "Elevator" }
                    p { "Future-event-list dispatch runs using LOOK, MDP-table, and POMDP-belief policies." }
                }
                a class="card" href="/des/tools/routing" {
                    h3 { "Routing" }
                    p { "VRP/TSP solver runs and persisted route solutions." }
                }
                a class="card" href="/des/labs/factory-floor-track3t" {
                    h3 { "Factory floor" }
                    p { "Track3t warehouse-floor discrete-event animation." }
                }
            }
        },
    )
}

pub async fn run_page(Path(run_id): Path<String>) -> Markup {
    views::layout(
        "Run",
        "/runs",
        html! {
            h1 { "DES run " code { (run_id) } }
            p class="lede" {
                "This stable route is the cross-model entry point for a run identifier. "
                "Model-specific dashboards remain the source of detailed state while the shared run "
                "index is populated incrementally from Postgres and NATS/JetStream metadata."
            }
            div class="cards" {
                a class="card" href="/des/games/soccer" { h3 { "Soccer runs" } }
                a class="card" href="/des/games/elevator" { h3 { "Elevator runs" } }
                a class="card" href="/des/tools/routing" { h3 { "Routing solves" } }
                a class="card" href="/des/artifacts" { h3 { "Generated artifacts" } }
            }
        },
    )
}

pub async fn api_catalog() -> Json<Value> {
    Json(json!({
        "schema": "des.route-catalog.v1",
        "basePath": "/des",
        "pages": {
            "catalog": "/des/",
            "models": "/des/models",
            "runs": "/des/runs/{run_id}",
            "artifacts": "/des/artifacts/{artifact_id}",
            "soccer": "/des/games/soccer",
            "soccerPlanner": "/des/games/soccer/planner",
            "elevator": "/des/games/elevator",
            "routing": "/des/tools/routing",
            "factoryFloorTrack3t": "/des/labs/factory-floor-track3t"
        },
        "api": {
            "catalog": "/des/api/v1/catalog",
            "solve": "/des/api/v1/solve",
            "solveById": "/des/api/v1/solve/{id}"
        },
        "compatibility": ["/des-rs/*", "/out/*"],
        "ownership": {
            "application": "discrete-event-systems/des-web.rs",
            "gitops": "ORESoftware/k8s-cluster"
        }
    }))
}
