# DES public route contract

`des-web.rs` is the dynamic browser application owned by the
`discrete-event-systems` GitHub organization. It is mounted by
`ORESoftware/k8s-cluster` below one public namespace: `/des`.

Implementation tracking: [Linear DEN-1936](https://linear.app/denman/issue/DEN-1936/des-webrsk8s-cluster-consolidate-public-des-pages-under-des)  
Operational rollout: [Linear DEN-2280](https://linear.app/denman/issue/DEN-2280/k8s-clusterdes-webrs-verify-the-canonical-des-rollout-in-aws-and)  
DES tracker: [des-web.rs#11](https://github.com/discrete-event-systems/des-web.rs/issues/11)  
GitOps tracker: [k8s-cluster#991](https://github.com/ORESoftware/k8s-cluster/issues/991)

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
local development and old in-cluster clients do not break. In Kubernetes,
`DES_PUBLIC_PATH_MODE=mounted` rewrites browser links, htmx URLs, forms,
JavaScript endpoint strings, and redirects to the canonical public paths. A
trusted future gateway may alternatively send `X-Forwarded-Prefix: /des`.

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
  gateway proxy/redirect rules, probes, resources, Argo CD, and rollout tests.
- DES engine repositories: simulation behavior and reusable libraries; they do
  not own public ingress configuration.

This prevents the previous split where `/des/` pointed at `dd-des-simulator`
while newer pages lived in a separate repository and `/des/music` jumped back
to `/des-rs/music`.

## Delivery status

The application and GitOps implementation were merged on August 5, 2026.

- Application PR: [des-web.rs#10](https://github.com/discrete-event-systems/des-web.rs/pull/10)
  - final source head: `77741ec8b5331617f71416748ef5f06846e43a5d`
  - merge commit: `e7d8b284dd796826bc09120bbd10295b0bf2783f`
  - application CI and immutable image publication passed
- GitOps PR: [k8s-cluster#872](https://github.com/ORESoftware/k8s-cluster/pull/872)
  - final source head: `16b9ecbad319a5433f5a58dec6e386ea48605f05`
  - merge commit: `7b77b48dcb347a0c474da1831e09f27338db43c1`
  - the focused DES route contract and full kustomize render passed
- Post-merge GitOps documentation: [k8s-cluster#996](https://github.com/ORESoftware/k8s-cluster/pull/996)
  - merge commit: `24e40c65b19d3673c7f5512aa76f9e82e082c430`
  - the branch was semantically reapplied on current `main` before merge

The promoted image is immutable and tied to the exact application source:

```text
ghcr.io/discrete-event-systems/des-web.rs:sha-77741ec8b5331617f71416748ef5f06846e43a5d@sha256:c3b32a5ef767bcdba515c8199fce363871ba2916e4c824609a09a37b3adc02e5
```

The older GitOps PR merge-reference catalog job reported drift. Current
`k8s-cluster/main` was regenerated through its locked Nix toolchain; the
generator wrote 86 application records from 121 tracked documents and produced
no diff (`catalog/applications.json is already current`). No generated catalog
commit is required.

Implementation work is complete in [DEN-1936](https://linear.app/denman/issue/DEN-1936/des-webrsk8s-cluster-consolidate-public-des-pages-under-des). Live deployment and evidence remain intentionally open in [DEN-2280](https://linear.app/denman/issue/DEN-2280/k8s-clusterdes-webrs-verify-the-canonical-des-rollout-in-aws-and), [des-web.rs#11](https://github.com/discrete-event-systems/des-web.rs/issues/11), and [k8s-cluster#991](https://github.com/ORESoftware/k8s-cluster/issues/991).

## Operational rollout

Completed delivery gates:

- [x] Merge and publish `des-web.rs` from the successful source revision.
- [x] Record the immutable image SHA and digest in `k8s-cluster`.
- [x] Merge the GitOps objects and route contract after the application PR.
- [x] Validate the focused route contract and full `dd-next-runtime` render.
- [x] Regenerate the current Argo application catalog with the locked toolchain and verify that it is already current.

Remaining operational gates:

1. sync `dd-next-runtime` through both AWS and Hetzner Argo CD control planes;
2. verify `/des/`, `/des/models`, canonical games/tools/labs pages,
   `/des/api/v1/catalog`, `/des/healthz`, and `/des/readyz` through both public
   entry points;
3. verify planner/solve delegation to `dd-des-rs`, optional persisted fragments,
   and degraded operation without a database URL;
4. observe `/des-rs/*`, `/out/*`, and `/des/music` traffic before removing any
   compatibility route;
5. repair the independent private-submodule deploy-key configuration in
   `k8s-cluster` so repository-wide private-source checks can initialize
   `remote/libs`.

Legacy routes may be retired only after their request counters remain at zero
for an agreed observation window and the evidence is attached to the rollout
trackers.
