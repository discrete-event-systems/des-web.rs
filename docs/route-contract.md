# DES public route contract

`des-web.rs` is the dynamic browser application owned by the
`discrete-event-systems` GitHub organization. It is mounted by
`ORESoftware/k8s-cluster` below one public namespace: `/des`.

## Canonical routes

| Public route | Purpose | Service-local handler |
|---|---|---|
| `/des/` | catalog | `/` |
| `/des/models` | cross-model index | `/models` |
| `/des/games/soccer` | soccer learning/tournaments | `/games/soccer` |
| `/des/games/soccer/planner` | rotation planner | `/games/soccer/planner` |
| `/des/games/elevator` | elevator learning/dispatch | `/games/elevator` |
| `/des/games/elevator/player` | elevator playback | `/games/elevator/player` |
| `/des/tools/routing` | VRP/TSP tool | `/tools/routing` |
| `/des/labs/factory-floor-track3t` | Track3t lab | `/labs/factory-floor-track3t` |
| `/des/runs/{run_id}` | stable cross-model run entry | `/runs/{run_id}` |
| `/des/artifacts/{artifact_id}` | generated/vendored output | `/artifacts/{artifact_id}` |
| `/des/api/v1/catalog` | machine-readable route catalog | `/api/v1/catalog` |
| `/des/api/v1/solve` | routing solver API | `/api/v1/solve` |
| `/des/api/v1/solve/{id}` | routing result API | `/api/v1/solve/{id}` |

The gateway strips `/des/` before forwarding. The server also keeps the old
service-local routes (`/soccer`, `/routing`, `/track3t`, and so on) so direct
local development and old in-cluster clients do not break. Response middleware
rewrites browser links, htmx URLs, JavaScript endpoint strings, and redirects to
the canonical public paths when the trusted gateway sends
`X-Forwarded-Prefix: /des`.

## Compatibility policy

`/des-rs/*` and `/out/*` are compatibility surfaces, not places for new links.
The Kubernetes gateway redirects safe browser reads into `/des/*`; engine-only
write/stream endpoints may continue to proxy to `dd-des-rs` until they have a
versioned `/des/api/v1` replacement.

Compatibility routes must:

1. preserve query strings;
2. use permanent redirects for GET/HEAD pages after validation;
3. never redirect a mutation across services implicitly;
4. emit access metrics so removal can be evidence-based;
5. remain covered by gateway contract tests.

## Ownership boundary

- `discrete-event-systems/des-web.rs`: pages, route taxonomy, link generation,
  browser behavior, API shape, app tests, and container publication.
- `ORESoftware/k8s-cluster`: Deployment, Service, NetworkPolicy, PDB, secrets,
  gateway proxy/redirect rules, probes, resources, and rollout tests.
- DES engine repositories: simulation behavior and reusable libraries; they do
  not own public ingress configuration.

This prevents the previous split where `/des/` pointed at `dd-des-simulator`
while newer pages lived in a separate repository and `/des/music` jumped back
to `/des-rs/music`.

## Rollout order

1. Merge and publish `des-web.rs`.
2. Record the immutable image digest in `k8s-cluster`.
3. Apply `dd-des-web` through Argo CD.
4. Verify `/healthz`, `/readyz`, the catalog, canonical pages, and compatibility
   redirects from both AWS and Hetzner entry points.
5. Remove legacy routes only after their request counters remain at zero for an
   agreed observation window.
