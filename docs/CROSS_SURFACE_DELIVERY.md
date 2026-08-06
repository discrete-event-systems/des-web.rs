# Cross-surface delivery

Verified **2026-08-06**.

## Surfaces

- Rust web application: `discrete-event-systems/des-web.rs`
- Flutter Android/iOS, Flutter Web, and Flutter desktop: `discrete-event-systems/des-flutter` — proposed/planned
- Rust desktop workbench: `discrete-event-systems/des-desktop.rs` — proposed/planned
- Shared contracts: DES interfaces, generated clients, model/event schemas, deterministic seeds, route catalog, result bundles, and conformance fixtures

Repository names are allocation targets until their remotes and builds are verified.

## Judgment-based propagation

Evaluate mobile, Flutter Web, Flutter desktop, Rust desktop, and shared contracts for every user-visible or contract-changing web change. Public catalog presentation, SEO, and vendored browser artifacts may remain web-only. Local datasets, files, large batch runs, offline replay, and workbench rendering may be native-specific. Model/run semantics, solver behavior, deterministic replay, result interpretation, permissions, errors, notifications, and navigation normally propagate or require an explicit rationale and parity issue.

## Deep links

```text
https://<verified-des-owned-host>/open/<route>?<bounded-query>
```

The host must be verified and the existing `/des` public route taxonomy must remain canonical. A custom-scheme fallback requires a reviewed ADR and must not be guessed. All surfaces share versioned route types and golden fixtures and support cold start, already-running delivery, authentication resume, replay/expiry rejection, browser fallback, and confirmation before imports, execution, export, or destructive actions.

Never put private datasets, result payloads, credentials, tokens, absolute local paths, or sensitive experiment inputs in URLs. Use bounded identifiers or short-lived, single-use, audience-bound codes and validate model/run/artifact IDs, route version, action, authorization, limits, and user intent.

## Review checklist

- [ ] Flutter Android/iOS impact evaluated.
- [ ] Flutter Web/mobile-web impact evaluated.
- [ ] Flutter desktop impact evaluated.
- [ ] Rust desktop workbench impact evaluated.
- [ ] Shared model/client/route/fixture impact evaluated.
- [ ] `/des` route and deep-link compatibility tested where relevant.
- [ ] Omitted surfaces have a rationale and follow-up when needed.

## Routing

- GitHub Project: [`discrete-event-systems-project` — Project 1](https://github.com/orgs/discrete-event-systems/projects/1)
- Linear project: [`github.com/discrete-event-systems`](https://linear.app/denman/project/githubcomdiscrete-event-systems-4a3086ae0c45)
- Central policy: [`cross-surface-delivery.md`](https://github.com/ORESoftware/project-registry/blob/main/docs/cross-surface-delivery.md)
- Desktop registry: [`desktop-applications.json`](https://github.com/ORESoftware/project-registry/blob/main/registry/desktop-applications.json)
