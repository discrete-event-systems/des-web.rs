//! Maud views + htmx partials. Server-rendered pages (the M and H in MASH):
//! full pages carry the shared layout; `/partials/*` handlers return fragments
//! that htmx swaps in (`hx-get` + `hx-trigger="load"`), so every DB-backed
//! section loads independently and degrades to a notice when Postgres is off.

use axum::extract::State;
use axum::response::Html;
use maud::{html, Markup, DOCTYPE};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

use dd_pg_defs_sea_orm::{
    des_fel_elevator_dispatch_decisions as elevator_decisions,
    des_fel_elevator_learning_runs as elevator_runs, des_soccer_learning_runs as soccer_runs,
    des_soccer_tournament_matches as soccer_matches, des_soccer_tournaments as soccer_tournaments,
};

use crate::artifacts;
use crate::entities::{des_web_routing_solves as routing_solves, des_web_sims as sims};
use crate::AppState;

const NAV: &[(&str, &str)] = &[
    ("/", "Home"),
    ("/soccer", "Soccer"),
    ("/soccer/planner", "Planner"),
    ("/routing", "Routing"),
    ("/track3t", "Track3t"),
    ("/elevator", "Elevator"),
    ("/artifacts", "Artifacts"),
    ("/login", "Login"),
];

pub fn layout(title: &str, active: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " · des-web" }
                link rel="stylesheet" href="/assets/app.css";
                script src="/assets/htmx.min.js" {}
            }
            body {
                header {
                    a class="brand" href="/" { "des-web" span class="brand-sub" { "discrete-event-systems" } }
                    nav {
                        @for (href, label) in NAV {
                            a href=(href) class=[(**href == *active).then_some("active")] { (label) }
                        }
                    }
                }
                main { (body) }
                footer {
                    span { "MASH — maud · axum · supabase · sea-orm · htmx" }
                    span { " · schema contract: pg-defs (k8s-libs-and-shared-defs) · migrations: dpm" }
                }
            }
        }
    }
}

fn table_note(msg: &str) -> Markup {
    html! { p class="note" { (msg) } }
}

fn db_offline() -> Markup {
    table_note("Postgres is not reachable — set DATABASE_URL (local, Supabase, or RDS) and run scripts/dev-db.sh to converge + seed the schema.")
}

/// A failed DB query. Log the real error server-side (for operators) and show
/// the client a generic notice — never echo raw `DbErr` text, which leaks
/// schema/table names and query internals to the browser.
fn db_error(context: &str, err: &sea_orm::DbErr) -> Markup {
    tracing::warn!(context, error = %err, "des-web db query failed");
    html! {
        (db_offline())
        p class="note note-dim" { "(query failed — see server logs)" }
    }
}

fn status_chip(status: &str) -> Markup {
    html! { span class={ "chip chip-" (status) } { (status) } }
}

fn micros_secs(m: i64) -> String {
    format!("{:.1}s", m as f64 / 1_000_000.0)
}

// ---------------------------------------------------------------------------
// home
// ---------------------------------------------------------------------------

pub async fn home() -> Markup {
    layout(
        "Sims & games",
        "/",
        html! {
            section class="hero" {
                h1 { "Discrete-event sims & games" }
                p {
                    "The sim/game pages from " code { "ORESoftware/k8s-cluster" } " — soccer, track3t, "
                    "optimal routing, elevators — copied onto one standalone MASH server "
                    "(maud + axum + supabase + sea-orm + htmx), with the data behind them served "
                    "from the shared " code { "pg-defs" } " Postgres contract."
                }
                div class="chips" {
                    span id="db-status" hx-get="/partials/db-status" hx-trigger="load, every 30s" {
                        span class="chip" { "db: checking…" }
                    }
                }
            }
            section {
                h2 { "Catalog" }
                div id="catalog" hx-get="/partials/sims" hx-trigger="load" {
                    p class="note" { "Loading catalog…" }
                }
            }
        },
    )
}

pub async fn partial_db_status(State(app): State<AppState>) -> Markup {
    let ok = crate::db::ping(&app.db).await;
    html! {
        @if ok {
            span class="chip chip-completed" { "db: connected" }
        } @else {
            span class="chip chip-failed" { "db: offline (degraded mode)" }
        }
    }
}

/// Static fallback mirroring schema/seed.sql, so the catalog renders even with
/// no database at all.
const FALLBACK_SIMS: &[(&str, &str, &str, &str)] = &[
    (
        "Soccer rotation planner",
        "game",
        "/soccer/planner",
        "des-rs + soccer-sim-game-engine.rs",
    ),
    (
        "Soccer learning & tournaments",
        "dashboard",
        "/soccer",
        "pg-defs (des_soccer_*)",
    ),
    (
        "Optimal routing — live VRP/TSP",
        "solver",
        "/routing",
        "routing-server-rs",
    ),
    (
        "Track3t factory floor",
        "artifact",
        "/track3t",
        "discrete-event-system out/",
    ),
    (
        "Elevator dispatch learning",
        "dashboard",
        "/elevator",
        "pg-defs (des_fel_elevator_*)",
    ),
    (
        "Elevator high-rise player",
        "artifact",
        "/elevator/player",
        "discrete-event-system out/",
    ),
    (
        "Soccer MIP/LP solver traces",
        "artifact",
        "/artifacts",
        "discrete-event-system out/",
    ),
];

pub async fn partial_sims(State(app): State<AppState>) -> Markup {
    let rows = match &app.db {
        Some(db) => {
            sims::Entity::find()
                .filter(sims::Column::IsEnabled.eq(true))
                .order_by_asc(sims::Column::SortOrder)
                .all(db)
                .await
        }
        None => Err(sea_orm::DbErr::Custom("no database configured".into())),
    };

    match rows {
        Ok(rows) if !rows.is_empty() => html! {
            div class="cards" {
                @for s in &rows {
                    a class="card" href=(s.page_route) {
                        div class="card-head" {
                            h3 { (s.title) }
                            span class={ "chip chip-kind-" (s.kind) } { (s.kind) }
                        }
                        p { (s.blurb) }
                        p class="meta" { "from " (s.source_service) }
                        @if !s.engine.is_empty() {
                            p class="meta" { "engine: " (s.engine) }
                        }
                    }
                }
            }
        },
        Ok(_) => html! {
            (table_note("des_web_sims is empty — run schema/seed.sql (or scripts/dev-db.sh) to load the catalog."))
            (fallback_cards())
        },
        Err(err) => html! {
            (db_offline())
            p class="note note-dim" { "(" (err.to_string()) ")" }
            (fallback_cards())
        },
    }
}

fn fallback_cards() -> Markup {
    html! {
        p class="note note-dim" { "Showing the built-in catalog instead:" }
        div class="cards" {
            @for (title, kind, route, source) in FALLBACK_SIMS {
                a class="card" href=(route) {
                    div class="card-head" {
                        h3 { (title) }
                        span class={ "chip chip-kind-" (kind) } { (kind) }
                    }
                    p class="meta" { "from " (source) }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// soccer
// ---------------------------------------------------------------------------

pub async fn soccer_page() -> Markup {
    layout(
        "Soccer",
        "/soccer",
        html! {
            h1 { "Soccer — learning & tournaments" }
            p class="lede" {
                "Data straight from the shared " code { "pg-defs" } " contract tables the akrion "
                "soccer stack writes (" code { "des_soccer_*" } "), read with SeaORM. "
                "Play the interactive " a href="/soccer/planner" { "rotation planner" } ", or watch the "
                a href="/artifacts/soccer-IP-MIP-feasible-solver" { "MIP solver trace" } "."
            }
            section {
                h2 { "Tournaments" }
                div hx-get="/partials/soccer/tournaments" hx-trigger="load" { p class="note" { "Loading…" } }
            }
            section {
                h2 { "Knockout matches" }
                div hx-get="/partials/soccer/matches" hx-trigger="load" { p class="note" { "Loading…" } }
            }
            section {
                h2 { "Recent learning runs" }
                div hx-get="/partials/soccer/runs" hx-trigger="load, every 20s" { p class="note" { "Loading…" } }
            }
        },
    )
}

pub async fn partial_soccer_tournaments(State(app): State<AppState>) -> Markup {
    let Some(db) = &app.db else {
        return db_offline();
    };
    match soccer_tournaments::Entity::find()
        .order_by_desc(soccer_tournaments::Column::Id)
        .limit(12)
        .all(db)
        .await
    {
        Err(err) => db_error("partial", &err),
        Ok(rows) if rows.is_empty() => {
            table_note("No tournaments yet — run schema/seed.sql for demo data.")
        }
        Ok(rows) => html! {
            table {
                thead { tr {
                    th { "id" } th { "date" } th { "mode" } th { "teams" }
                    th { "played" } th { "champion" } th { "wall time" } th { "status" }
                } }
                tbody {
                    @for t in &rows {
                        tr {
                            td { (t.id) }
                            td { (t.tournament_date) }
                            td { (t.learning_mode) }
                            td { (t.team_count) }
                            td { (t.matches_played) "/" (t.match_count) }
                            td { @match t.champion_team_id {
                                Some(c) => { "team " (c) },
                                None => { "—" },
                            } }
                            td { @match t.wall_time_seconds {
                                Some(w) => { (format!("{w:.0}s")) },
                                None => { "—" },
                            } }
                            td { (status_chip(&t.status)) }
                        }
                    }
                }
            }
        },
    }
}

pub async fn partial_soccer_matches(State(app): State<AppState>) -> Markup {
    let Some(db) = &app.db else {
        return db_offline();
    };
    // Grouped by tournament (newest first), then in bracket order. The
    // tournament_id FK column is available again since the pg-defs wrapped-FK
    // parser fix (k8s-libs-and-shared-defs ee57b76).
    match soccer_matches::Entity::find()
        .order_by_desc(soccer_matches::Column::TournamentId)
        .order_by_asc(soccer_matches::Column::MatchIndex)
        .limit(16)
        .all(db)
        .await
    {
        Err(err) => db_error("partial", &err),
        Ok(rows) if rows.is_empty() => table_note("No matches recorded yet."),
        Ok(rows) => html! {
            table {
                thead { tr {
                    th { "tournament" } th { "#" } th { "stage" } th { "fixture" }
                    th { "score" } th { "training steps (h/a)" } th { "recorded" }
                } }
                tbody {
                    @for m in &rows {
                        tr {
                            td { (m.tournament_id) }
                            td { (m.match_index) }
                            td { (m.stage) }
                            td { "team " (m.home_team_id) " vs team " (m.away_team_id) }
                            td {
                                b { (m.home_goals) "–" (m.away_goals) }
                                @if let Some(w) = m.shootout_winner_team_id {
                                    span class="meta" { " (pens: team " (w) ")" }
                                }
                            }
                            td { (m.home_training_steps) " / " (m.away_training_steps) }
                            td { (m.recorded_at.format("%Y-%m-%d %H:%M").to_string()) }
                        }
                    }
                }
            }
        },
    }
}

pub async fn partial_soccer_runs(State(app): State<AppState>) -> Markup {
    let Some(db) = &app.db else {
        return db_offline();
    };
    match soccer_runs::Entity::find()
        .order_by_desc(soccer_runs::Column::CreatedAt)
        .limit(12)
        .all(db)
        .await
    {
        Err(err) => db_error("partial", &err),
        Ok(rows) if rows.is_empty() => {
            table_note("No learning runs yet — run schema/seed.sql for demo data.")
        }
        Ok(rows) => html! {
            table {
                thead { tr {
                    th { "episode" } th { "runner" } th { "score" } th { "outcome (h/a)" }
                    th { "fitness" } th { "elapsed" } th { "status" }
                } }
                tbody {
                    @for r in &rows {
                        tr {
                            td { (r.episode_index) }
                            td { (r.runner_id) }
                            td { b { (r.score_home) "–" (r.score_away) } }
                            td { (r.home_outcome) " / " (r.away_outcome) }
                            td { (format!("{:.3}", r.fitness_micros as f64 / 1_000_000.0)) }
                            td { (r.elapsed_millis) "ms" }
                            td { (status_chip(&r.status)) }
                        }
                    }
                }
            }
        },
    }
}

// ---------------------------------------------------------------------------
// elevator
// ---------------------------------------------------------------------------

pub async fn elevator_page() -> Markup {
    layout(
        "Elevator",
        "/elevator",
        html! {
            h1 { "Elevator — FEL dispatch learning" }
            p class="lede" {
                "Future-event-list elevator runs comparing LOOK, MDP-table and POMDP-belief "
                "dispatch, from the shared " code { "des_fel_elevator_*" } " pg-defs tables. "
                "Watch the animated " a href="/elevator/player" { "high-rise playback" } "."
            }
            section {
                h2 { "Learning runs" }
                div hx-get="/partials/elevator/runs" hx-trigger="load" { p class="note" { "Loading…" } }
            }
            section {
                h2 { "Sample dispatch decisions" }
                div hx-get="/partials/elevator/decisions" hx-trigger="load" { p class="note" { "Loading…" } }
            }
        },
    )
}

pub async fn partial_elevator_runs(State(app): State<AppState>) -> Markup {
    let Some(db) = &app.db else {
        return db_offline();
    };
    match elevator_runs::Entity::find()
        .order_by_desc(elevator_runs::Column::CreatedAt)
        .limit(12)
        .all(db)
        .await
    {
        Err(err) => db_error("partial", &err),
        Ok(rows) if rows.is_empty() => {
            table_note("No elevator runs yet — run schema/seed.sql for demo data.")
        }
        Ok(rows) => html! {
            table {
                thead { tr {
                    th { "run" } th { "policy" } th { "building" } th { "served" }
                    th { "mean wait" } th { "belief updates" } th { "status" }
                } }
                tbody {
                    @for r in &rows {
                        tr {
                            td { (r.run_label) }
                            td { code { (r.dispatch_policy) } }
                            td { (r.floors) "F × " (r.shafts) " cars × cap " (r.capacity) }
                            td { (r.served) "/" (r.arrivals) }
                            td { b { (micros_secs(r.mean_wait_micros)) } }
                            td { (r.pomdp_belief_updates) }
                            td { (status_chip(&r.status)) }
                        }
                    }
                }
            }
        },
    }
}

pub async fn partial_elevator_decisions(State(app): State<AppState>) -> Markup {
    let Some(db) = &app.db else {
        return db_offline();
    };
    match elevator_decisions::Entity::find()
        .order_by_asc(elevator_decisions::Column::SimTimeMicros)
        .limit(20)
        .all(db)
        .await
    {
        Err(err) => db_error("partial", &err),
        Ok(rows) if rows.is_empty() => table_note("No dispatch decisions recorded."),
        Ok(rows) => html! {
            table {
                thead { tr {
                    th { "t (sim)" } th { "#" } th { "call floor" } th { "car" } th { "policy" }
                } }
                tbody {
                    @for d in &rows {
                        tr {
                            td { (micros_secs(d.sim_time_micros)) }
                            td { (d.decision_index) }
                            td { "floor " (d.call_floor) }
                            td { "car " (d.car_index) }
                            td { code { (d.policy_kind) } }
                        }
                    }
                }
            }
        },
    }
}

// ---------------------------------------------------------------------------
// routing
// ---------------------------------------------------------------------------

pub async fn routing_page() -> Html<&'static str> {
    Html(include_str!("../assets/routing-dashboard.html"))
}

pub async fn partial_routing_solves(State(app): State<AppState>) -> Markup {
    let Some(db) = &app.db else {
        return db_offline();
    };
    match routing_solves::Entity::find()
        .order_by_desc(routing_solves::Column::CreatedAt)
        .limit(10)
        .all(db)
        .await
    {
        Err(err) => db_error("partial", &err),
        Ok(rows) if rows.is_empty() => {
            table_note("No persisted solves yet — run one above, or load schema/seed.sql.")
        }
        Ok(rows) => html! {
            table {
                thead { tr {
                    th { "solve" } th { "stops" } th { "vehicles" } th { "restarts" }
                    th { "best distance" } th { "improvements" } th { "when" } th { "status" }
                } }
                tbody {
                    @for s in &rows {
                        tr {
                            td { code { (s.id.to_string()[..8].to_string()) } }
                            td { (s.stop_count) }
                            td { (s.vehicles) }
                            td { (s.restarts_done) "/" (s.restarts_total) }
                            td { @match s.best_distance {
                                Some(d) => { b { (format!("{d:.1}")) } },
                                None => { "—" },
                            } }
                            td { (s.improvements) }
                            td { (s.created_at.format("%Y-%m-%d %H:%M").to_string()) }
                            td { (status_chip(&s.status)) }
                        }
                    }
                }
            }
        },
    }
}

// ---------------------------------------------------------------------------
// artifacts + login + 404
// ---------------------------------------------------------------------------

pub async fn artifacts_index() -> Markup {
    layout(
        "Artifacts",
        "/artifacts",
        html! {
            h1 { "Rendered DES artifacts" }
            p class="lede" {
                "Self-contained HTML animations rendered by the DES engine, vendored from "
                code { "discrete-event-system/out/" } " and served gzip-compressed as stored."
            }
            div class="cards" {
                @for a in artifacts::ARTIFACTS {
                    a class="card" href={ "/artifacts/" (a.slug) } {
                        div class="card-head" {
                            h3 { (a.title) }
                            span class="chip chip-kind-artifact" { "artifact" }
                        }
                        p { (a.blurb) }
                        p class="meta" { (a.source) " · " (a.plain_size) }
                    }
                }
            }
        },
    )
}

pub async fn login_page(State(app): State<AppState>) -> Markup {
    let configured = app.cfg.supabase().is_some();
    layout(
        "Login",
        "/login",
        html! {
            h1 { "Login" }
            p class="lede" { "Passwordless magic-link sign-in via Supabase GoTrue (the S in MASH)." }
            @if configured {
                form hx-post="/auth/magic-link" hx-target="#auth-result" hx-swap="innerHTML" class="auth-form" {
                    label { "Email"
                        input type="email" name="email" placeholder="you@example.com" required;
                    }
                    button type="submit" { "Send magic link" }
                }
            } @else {
                p class="note" {
                    "Supabase is not configured. Set " code { "SUPABASE_URL" } " and "
                    code { "SUPABASE_ANON_KEY" } " to enable magic-link login. "
                    "(Supabase can also be the app's Postgres: point " code { "DATABASE_URL" }
                    " at the project's connection string.)"
                }
            }
            div id="auth-result" {}
        },
    )
}

pub async fn not_found() -> (axum::http::StatusCode, Markup) {
    (
        axum::http::StatusCode::NOT_FOUND,
        layout(
            "Not found",
            "",
            html! {
                h1 { "404" }
                p class="lede" { "No such page. " a href="/" { "Back to the catalog." } }
            },
        ),
    )
}
