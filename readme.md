# des-web.rs — canonical DES web application

`des-web.rs` is the dynamic MASH application for the
[`discrete-event-systems`](https://github.com/discrete-event-systems)
organization. `ORESoftware/k8s-cluster` deploys it as `dd-des-web` and mounts it
below the single public namespace **`/des`**.

The ownership boundary is deliberate:

- this repository owns pages, browser behavior, routes, APIs, tests, and the
  container image;
- `ORESoftware/k8s-cluster` owns Deployment/Service/NetworkPolicy/PDB,
  observability wiring, gateway rules, compatibility redirects, and rollout;
- engine repositories own simulation behavior and reusable libraries, not
  public ingress configuration.

This replaces the previous split where `/des/` pointed at
`dd-des-simulator`, newer pages lived here, and selected links jumped back to
`/des-rs/*`.

## Canonical route surface

| public route | purpose | service-local route |
|---|---|---|
| `/des/` | DES catalog | `/` |
| `/des/models` | cross-model index | `/models` |
| `/des/games/soccer` | tournaments and learning | `/games/soccer` |
| `/des/games/soccer/planner` | rotation planner | `/games/soccer/planner` |
| `/des/games/elevator` | FEL dispatch learning | `/games/elevator` |
| `/des/games/elevator/player` | elevator playback | `/games/elevator/player` |
| `/des/tools/routing` | live VRP/TSP solver | `/tools/routing` |
| `/des/labs/factory-floor-track3t` | Track3t factory-floor lab | `/labs/factory-floor-track3t` |
| `/des/runs/{run_id}` | stable cross-model run entry | `/runs/{run_id}` |
| `/des/artifacts/{artifact_id}` | rendered/vendored output | `/artifacts/{artifact_id}` |
| `/des/api/v1/catalog` | machine-readable route catalog | `/api/v1/catalog` |
| `/des/api/v1/solve` | routing solve API | `/api/v1/solve` |
| `/des/api/v1/solve/{id}` | routing solve result | `/api/v1/solve/{id}` |

The gateway strips `/des/` and sends `X-Forwarded-Prefix: /des`. In that
mounted mode, response middleware rewrites first-party HTML links, htmx URLs,
JavaScript endpoint strings, and redirects to the public taxonomy. Without that
header the historical root routes continue to work, preserving local
quickstarts and old in-cluster callers.

`/des-rs/*` and `/out/*` are compatibility surfaces only. New links must use
`/des/*`. See [`docs/route-contract.md`](docs/route-contract.md) for the full
contract, migration rules, and rollout order.

## Application pages

| service-local route | what | source/data |
|---|---|---|
| `/` | catalog of sims/games | `des_web_sims` with built-in fallback |
| `/soccer` | tournaments, knockout matches, learning runs | pg-defs `des_soccer_*` |
| `/soccer/planner` | interactive rotation planner | vendored planner; solve proxy to `DES_UPSTREAM_URL` |
| `/routing` | live VRP/TSP dashboard | in-process solver + `des_web_routing_solves` |
| `/track3t` | warehouse-floor animation | vendored DES artifact |
| `/elevator` | FEL dispatch learning | pg-defs `des_fel_elevator_*` |
| `/elevator/player` | high-rise playback | vendored DES artifact |
| `/artifacts` | generated DES renders | gzip-vendored self-contained HTML |
| `/login` | Supabase magic-link login | optional GoTrue configuration |

Vendored artifacts are stored gzip-compressed in `assets/artifacts/` and served
as stored with `Content-Encoding: gzip`. They are fully self-contained and are
not rewritten by the `/des` response middleware.

## MASH stack

| letter | component | role |
|---|---|---|
| M | maud | compile-time HTML pages and htmx fragments |
| A | axum 0.8 | routing, middleware, probes, graceful shutdown |
| S | Supabase | optional GoTrue magic-link auth and Postgres provider |
| S | SeaORM 1.1 | typed access through the generated pg-defs entity crate |
| H | htmx 2 | independently loading DB-backed sections |

The process always boots. Missing Postgres, Supabase, or DES engine upstreams
degrade only the features that need them and leave the catalog available.

## Shared schema contract

The private `libs/` git submodule points at
`ORESoftware/k8s-libs-and-shared-defs`:

1. `libs/pg-defs/schema/schema.sql` is the shared desired state.
2. `schema/des-web.sql` adds the des-web-owned `des_web_sims` and
   `des_web_routing_solves` tables.
3. `dd-pg-defs-sea-orm` is consumed as a generated path dependency.
4. `schema/seed.sql` provides idempotent local/CI demo data.

Declarative migrations use `dpm` 0.3.2 or newer:

```sh
export SHADOW_DATABASE_URL=postgres://localhost:5432/postgres
export TARGET_DATABASE_URL=postgres://localhost:5432/des_web_dev
scripts/dpm.sh diff
scripts/dpm.sh verify
scripts/dpm.sh review
scripts/dpm.sh apply
```

Destructive statements remain review-gated; production applies are never an
automatic application-start side effect.

## Local development

```sh
git clone --recurse-submodules git@github.com:discrete-event-systems/des-web.rs.git
cd des-web.rs
scripts/dev-db.sh
DATABASE_URL=postgres://localhost:5432/des_web_dev cargo run
open http://localhost:8130
```

Local development intentionally uses the service-local routes. To inspect the
public-path rewrite without Kubernetes, send the same trusted proxy header the
gateway sends:

```sh
curl -H 'X-Forwarded-Prefix: /des' http://localhost:8130/
```

`.env.example` documents database URL resolution, Supabase, and
`DES_UPSTREAM_URL` for live soccer planner solves.

## Tests and publication

CI runs formatting, Clippy, Rust tests, declarative schema convergence, seeded
Postgres assertions, and Playwright browser tests. Route tests cover both the
service-local compatibility surface and the `/des` public rewrite contract.

On `main`, `publish-image.yml` builds the initialized-submodule Docker context
and publishes immutable SHA and `main` tags to
`ghcr.io/discrete-event-systems/des-web.rs`. GitOps must promote the resulting
**digest**, not a mutable tag, into `ORESoftware/k8s-cluster`.

## Layout

```text
src/main.rs          axum router, state, startup
src/public_paths.rs  /des mounting and compatibility rewrite
src/catalog.rs       model/run pages + route catalog API
src/views.rs         maud pages and htmx fragments
src/routing.rs       VRP/TSP solver and persistence
src/planner.rs       planner page and DES-engine proxy
src/artifacts.rs     gzip artifact serving
schema/              declarative desired state + seed
scripts/             dpm and local database helpers
e2e/                 Playwright browser contract tests
```
