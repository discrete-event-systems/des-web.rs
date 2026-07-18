# des-web.rs — MASH server for discrete-event sims & games

One standalone Rust web server that serves the discrete-event sim/game pages
from [ORESoftware/k8s-cluster](https://github.com/ORESoftware/k8s-cluster)
(**copied, not ripped out** — the originals keep running in-cluster), with the
data behind them read from the shared Postgres contract.

For the public, dependency-free showcase, visit the Astro/GitHub Pages site at
[discrete-event-systems.github.io](https://discrete-event-systems.github.io/).
This repository remains the dynamic application server; the Pages site keeps
its simulation and game galleries static and browser-isolated.

**MASH** — the stack:

| letter | piece | here |
|---|---|---|
| **M** | [maud](https://maud.lambda.xyz) | compile-time HTML for all first-party pages/partials |
| **A** | [axum](https://github.com/tokio-rs/axum) 0.8 | routing, extractors, graceful shutdown |
| **S** | [Supabase](https://supabase.com) | GoTrue magic-link login (`/login`) + optional Postgres provider |
| **S** | [SeaORM](https://www.sea-ql.org/SeaORM/) 1.1 | all DB access (not sqlx directly) via the generated pg-defs entity crate |
| **H** | [htmx](https://htmx.org) 2 (vendored) | partial loading/refresh for every DB-backed section |

## Pages and where they were copied from

| route | what | copied from (k8s-cluster) | data |
|---|---|---|---|
| `/` | catalog of sims/games | new (maud+htmx) | `des_web_sims` |
| `/soccer` | tournaments, knockout matches, learning runs | new (maud+htmx) | pg-defs `des_soccer_*` |
| `/soccer/planner` | interactive rotation planner (MIP/MDP) | `soccer-sim-game-engine.rs` `planner_ui.html` (served by des-rs) | solves proxy to `DES_UPSTREAM_URL` |
| `/routing` | live VRP/TSP canvas dashboard | `routing-server-rs/src/dashboard.rs` | in-process solver; solves persist to `des_web_routing_solves` |
| `/track3t` | Track3t warehouse-floor DES animation | `discrete-event-system/out/factory-floor-track3t.html` | self-contained artifact |
| `/elevator` | FEL elevator dispatch learning (LOOK/MDP/POMDP) | new (maud+htmx) | pg-defs `des_fel_elevator_*` |
| `/elevator/player` | animated high-rise elevator playback | `discrete-event-system/out/elevator.html` | self-contained artifact |
| `/artifacts` | vendored DES engine renders (incl. soccer MIP/LP traces) | `discrete-event-system/out/*.html` | self-contained artifacts |
| `/login` | Supabase GoTrue magic-link login | pattern from `athleto-app-rs` | Supabase |

Vendored artifacts are stored **gzip-compressed** in `assets/artifacts/` and
served as-stored with `Content-Encoding: gzip` (the 31 MB track3t page is
667 KB over the wire). Every artifact is fully self-contained HTML/JS — no CDN,
no external requests, which also goes for htmx and all first-party assets.

## pg-defs — the shared schema contract

This repo is 100% connected to
[pg-defs](https://github.com/ORESoftware/k8s-libs-and-shared-defs) (the
`libs/` **git submodule**, private — you need repo access):

1. **Schema**: `libs/pg-defs/schema/schema.sql` is the canonical contract.
   `schema/des-web.sql` layers two des-web-owned tables on top
   (`des_web_sims`, `des_web_routing_solves`). Never edit generated adapters;
   never write imperative migrations.
2. **Rust**: SeaORM entities come from the generated
   `dd-pg-defs-sea-orm` crate (`libs/pg-defs/generated/rust/sea-orm`, a path
   dependency). Overlay-table entities live in `src/entities.rs`, mirroring
   `schema/des-web.sql`.

## Migrations — dpm (declarative, for AWS RDS / Supabase / local)

Migrations are declarative via
[dpm](https://github.com/declarative-migrations/declarative-postgres-migrate.rs):
the combined desired state (pg-defs `schema.sql` + `schema/des-web.sql`) is the
source of truth and the target database **converges** onto it. No migration
files are tracked — dpm introspects both sides and emits ordered, reviewable
SQL.

```sh
brew install declarative-migrations/tap/dpm

export SHADOW_DATABASE_URL=postgres://localhost:5432/postgres  # throwaway-DB server, never prod
export TARGET_DATABASE_URL=postgres://...                      # RDS / Supabase / local

scripts/dpm.sh diff      # print the migration SQL (never executes)
scripts/dpm.sh verify    # rehearse on a shadow replica, prove convergence
scripts/dpm.sh review    # diff + AI review
scripts/dpm.sh apply     # generate + execute (interactive confirm)
```

Target resolution matches the pg-defs tooling: `TARGET_DATABASE_URL` →
`AGENT_TASKS_RDS_DATABASE_URL` → `RDS_DATABASE_URL` → `DATABASE_URL` →
`PG_DATABASE_URL` → `SUPABASE_DB_URL`. Destructive statements are emitted
commented-out and refused at apply time without dpm's two explicit consent
flags. Never apply automatically; a human reviews the SQL first.

Supabase notes: point dpm and the app at the **session pooler or direct
connection** (port 5432), not the transaction pooler — DDL and prepared
statements need real sessions — and keep `SHADOW_DATABASE_URL` on a local
server (Supabase won't let dpm create/drop scratch databases).

Known upstream dpm limitation (same as pg-defs CI): varchar IN-list CHECK
constraints deparse in a form that never converges to string equality, so
drift checks are advisory for the pg-defs tables. The des-web overlay uses
`text` columns for its IN-list CHECKs and is unaffected.

## Quickstart (local)

```sh
git clone --recurse-submodules git@github.com:discrete-event-systems/des-web.rs.git
cd des-web.rs

# start Homebrew Postgres 17, create des_web_dev, dpm-apply the combined
# schema, load idempotent seed data (demo tournaments, elevator runs, catalog):
scripts/dev-db.sh

DATABASE_URL=postgres://localhost:5432/des_web_dev cargo run
open http://localhost:8130
```

`.env.example` documents every knob (Supabase auth, RDS URLs, the optional
`DES_UPSTREAM_URL` that turns the copied soccer-planner page into a live
solver by proxying to a running des-rs).

The server always boots — no DB, no Supabase, no upstream just degrade their
own sections with a visible notice.

## Layout

```
src/main.rs        router + state + startup (axum)
src/views.rs       maud pages + htmx partials
src/routing.rs     in-process VRP/TSP (multi-start NN + 2-opt) + persistence
src/planner.rs     vendored soccer-planner page + des-rs proxy
src/artifacts.rs   gzip-vendored DES artifact serving
src/auth.rs        Supabase GoTrue magic link
src/entities.rs    SeaORM entities for the overlay tables
schema/des-web.sql overlay desired-state (dpm source, with pg-defs schema.sql)
schema/seed.sql    idempotent demo data (pg-defs + overlay tables)
scripts/dpm.sh     declarative migration entrypoint
libs/              pg-defs submodule (schema contract + generated SeaORM crate)
```

Docker: `docker build .` (initialize the `libs` submodule first). CI uses the
`LIBS_DEPLOY_KEY` Actions secret, backed by a read-only deploy key scoped only
to the private pg-defs repository. Forked pull requests cannot receive that
secret and therefore cannot run the submodule-dependent jobs.
