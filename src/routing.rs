//! In-process port of dd-routing-server's multi-start VRP/TSP solver.
//!
//! The upstream service (k8s-cluster/remote/deployments/routing-server-rs)
//! fans restarts out over NATS JetStream workers; this copy runs the same
//! algorithm shape — sweep/nearest-neighbor construction + 2-opt improvement,
//! racing restarts against a shared incumbent — in a single tokio task so the
//! copied canvas dashboard works standalone. Live solves are tracked in
//! memory; finished solves persist to `des_web_routing_solves` (SeaORM) so the
//! page has history to display.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::entities::des_web_routing_solves;
use crate::AppState;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Stop {
    pub x: f64,
    pub y: f64,
}

/// Wire format matches the copied dashboard's JS exactly
/// (bestDistance/restartsDone/restartsTotal/improvements/status/stops/routes/depotIndex).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolveState {
    pub solve_id: Uuid,
    pub status: String,
    pub stops: Vec<Stop>,
    pub routes: Vec<Vec<usize>>,
    pub best_distance: f64,
    pub restarts_done: u32,
    pub restarts_total: u32,
    pub improvements: u32,
    pub depot_index: usize,
    pub seed: u64,
    pub vehicles: u32,
}

pub type SolveMap = Arc<Mutex<HashMap<Uuid, SolveState>>>;

#[derive(Debug, Deserialize)]
pub struct SolveRequest {
    pub generate: GenerateSpec,
    #[serde(default = "default_restarts")]
    pub restarts: u32,
}

#[derive(Debug, Deserialize)]
pub struct GenerateSpec {
    pub count: u32,
    #[serde(default = "default_vehicles")]
    pub vehicles: u32,
    #[serde(default)]
    pub seed: u64,
}

fn default_restarts() -> u32 {
    24
}
fn default_vehicles() -> u32 {
    4
}

/// SplitMix64 — deterministic per seed, no rand dependency.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(1))
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next_f64() * n as f64) as usize % n.max(1)
    }
}

fn generate_stops(count: usize, seed: u64) -> Vec<Stop> {
    let mut rng = Rng::new(seed);
    let mut stops = Vec::with_capacity(count);
    stops.push(Stop { x: 50.0, y: 50.0 }); // depot
    for _ in 1..count {
        stops.push(Stop {
            x: 2.0 + rng.next_f64() * 96.0,
            y: 2.0 + rng.next_f64() * 96.0,
        });
    }
    stops
}

fn dist(a: Stop, b: Stop) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

/// Closed-tour length of one route (indices into `stops`, depot included).
fn route_len(route: &[usize], stops: &[Stop]) -> f64 {
    if route.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0;
    for i in 0..route.len() {
        total += dist(stops[route[i]], stops[route[(i + 1) % route.len()]]);
    }
    total
}

fn total_len(routes: &[Vec<usize>], stops: &[Stop]) -> f64 {
    routes.iter().map(|r| route_len(r, stops)).sum()
}

/// Sweep construction: partition non-depot stops into contiguous angular arcs
/// around the depot (random rotation per restart), then order each arc by
/// nearest neighbor from the depot.
fn construct(stops: &[Stop], vehicles: usize, rng: &mut Rng) -> Vec<Vec<usize>> {
    let depot = stops[0];
    let rotation = rng.next_f64() * std::f64::consts::TAU;
    let mut order: Vec<usize> = (1..stops.len()).collect();
    order.sort_by(|&a, &b| {
        let ang = |i: usize| {
            let s = stops[i];
            ((s.y - depot.y).atan2(s.x - depot.x) + rotation).rem_euclid(std::f64::consts::TAU)
        };
        ang(a)
            .partial_cmp(&ang(b))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    // A few random swaps so restarts explore different partitions.
    for _ in 0..(order.len() / 6).max(1) {
        let (i, j) = (rng.below(order.len()), rng.below(order.len()));
        order.swap(i, j);
    }

    let per = order.len().div_ceil(vehicles);
    let mut routes = Vec::with_capacity(vehicles);
    for chunk in order.chunks(per.max(1)) {
        let mut remaining: Vec<usize> = chunk.to_vec();
        let mut route = vec![0usize];
        let mut current = depot;
        while !remaining.is_empty() {
            let (best_pos, _) = remaining
                .iter()
                .enumerate()
                .map(|(pos, &idx)| (pos, dist(current, stops[idx])))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap();
            let idx = remaining.swap_remove(best_pos);
            current = stops[idx];
            route.push(idx);
        }
        routes.push(route);
    }
    while routes.len() < vehicles {
        routes.push(vec![0]);
    }
    routes
}

/// Classic 2-opt on one closed route until no improvement (bounded passes).
fn two_opt(route: &mut [usize], stops: &[Stop]) {
    let n = route.len();
    if n < 4 {
        return;
    }
    for _pass in 0..40 {
        let mut improved = false;
        for i in 1..n - 1 {
            for j in i + 1..n {
                let a = route[i - 1];
                let b = route[i];
                let c = route[j];
                let d = route[(j + 1) % n];
                if a == c || b == d {
                    continue;
                }
                let delta = dist(stops[a], stops[c]) + dist(stops[b], stops[d])
                    - dist(stops[a], stops[b])
                    - dist(stops[c], stops[d]);
                if delta < -1e-9 {
                    route[i..=j].reverse();
                    improved = true;
                }
            }
        }
        if !improved {
            break;
        }
    }
}

async fn persist(db: &DatabaseConnection, s: &SolveState) {
    let now = chrono::Utc::now().fixed_offset();
    let row = des_web_routing_solves::ActiveModel {
        id: Set(s.solve_id),
        status: Set(s.status.clone()),
        stop_count: Set(s.stops.len() as i32),
        vehicles: Set(s.vehicles as i32),
        restarts_total: Set(s.restarts_total as i32),
        restarts_done: Set(s.restarts_done as i32),
        improvements: Set(s.improvements as i32),
        seed: Set(s.seed as i64),
        best_distance: Set(Some(s.best_distance)),
        depot_index: Set(s.depot_index as i32),
        stops: Set(serde_json::to_value(&s.stops).unwrap_or_else(|_| json!([]))),
        routes: Set(serde_json::to_value(&s.routes).unwrap_or_else(|_| json!([]))),
        created_at: Set(now),
        updated_at: Set(now),
        finished_at: Set(Some(now)),
    };
    if let Err(err) = row.insert(db).await {
        tracing::warn!(%err, solve_id = %s.solve_id, "failed to persist routing solve");
    }
}

async fn run_solve(solves: SolveMap, db: Option<DatabaseConnection>, id: Uuid) {
    let (stops, vehicles, restarts, seed) = {
        let map = solves.lock().await;
        let Some(s) = map.get(&id) else { return };
        (
            s.stops.clone(),
            s.vehicles as usize,
            s.restarts_total,
            s.seed,
        )
    };

    for r in 0..restarts {
        let mut rng = Rng::new(seed.wrapping_add(r as u64).wrapping_mul(0x100000001B3));
        let mut routes = construct(&stops, vehicles, &mut rng);
        for route in &mut routes {
            two_opt(route, &stops);
        }
        let candidate = total_len(&routes, &stops);

        {
            let mut map = solves.lock().await;
            let Some(s) = map.get_mut(&id) else { return };
            s.restarts_done = r + 1;
            if s.best_distance <= 0.0 || candidate < s.best_distance {
                s.best_distance = candidate;
                s.routes = routes;
                s.improvements += 1;
            }
        }
        // Pace the loop a little so the dashboard's 400ms poll visibly races
        // the incumbent, mirroring the distributed original.
        tokio::time::sleep(std::time::Duration::from_millis(12)).await;
    }

    let done = {
        let mut map = solves.lock().await;
        let Some(s) = map.get_mut(&id) else { return };
        s.status = "completed".to_string();
        s.clone()
    };
    if let Some(db) = db {
        persist(&db, &done).await;
    }
}

pub async fn post_solve(State(app): State<AppState>, Json(req): Json<SolveRequest>) -> Response {
    let count = req.generate.count.clamp(3, 1000) as usize;
    let vehicles = req.generate.vehicles.clamp(1, 64).min(count as u32 - 1);
    let restarts = req.restarts.clamp(1, 512);
    let seed = req.generate.seed;

    let id = Uuid::new_v4();
    let state = SolveState {
        solve_id: id,
        status: "running".to_string(),
        stops: generate_stops(count, seed),
        routes: Vec::new(),
        best_distance: 0.0,
        restarts_done: 0,
        restarts_total: restarts,
        improvements: 0,
        depot_index: 0,
        seed,
        vehicles,
    };
    app.solves.lock().await.insert(id, state);

    let solves = app.solves.clone();
    let db = app.db.clone();
    tokio::spawn(async move { run_solve(solves, db, id).await });

    Json(json!({ "solveId": id })).into_response()
}

pub async fn get_solve(State(app): State<AppState>, Path(id): Path<Uuid>) -> Response {
    if let Some(s) = app.solves.lock().await.get(&id) {
        return Json(s.clone()).into_response();
    }
    // Fall back to a persisted solve so history links keep working.
    if let Some(db) = &app.db {
        if let Ok(Some(row)) = des_web_routing_solves::Entity::find_by_id(id).one(db).await {
            let stops: Vec<Stop> = serde_json::from_value(row.stops).unwrap_or_default();
            let routes: Vec<Vec<usize>> = serde_json::from_value(row.routes).unwrap_or_default();
            let s = SolveState {
                solve_id: row.id,
                status: row.status,
                stops,
                routes,
                best_distance: row.best_distance.unwrap_or(0.0),
                restarts_done: row.restarts_done as u32,
                restarts_total: row.restarts_total as u32,
                improvements: row.improvements as u32,
                depot_index: row.depot_index as usize,
                seed: row.seed as u64,
                vehicles: row.vehicles as u32,
            };
            return Json(s).into_response();
        }
    }
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": format!("unknown solve id {id}") })),
    )
        .into_response()
}
