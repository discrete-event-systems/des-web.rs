# DES E2E orchestration

`des-web.rs` participates in the organization-wide DES interaction gate.

Until `discrete-event-systems/des-e2e` is provisioned, the executable orchestrator lives in `discrete-event-systems/.github` at `.github/workflows/des-e2e-orchestrator.yml`.

This repository remains authoritative for its native Rust, schema, PostgreSQL, and browser CI because it requires the private recursive `libs` submodule and `LIBS_DEPLOY_KEY`. The organization orchestrator therefore requires the current `main` SHA of `des-web.rs` to have a successful native `ci.yml` run before cross-repository E2E can pass.

External browser contracts are additionally exercised from:

- `discrete-event-systems-test/des-web-playwright-e2e`
- `discrete-event-systems-test/des-web-puppeteer-e2e`

When the canonical `discrete-event-systems/des-e2e` repository exists, it should call this repository's validated build/fixture contract rather than duplicating private-submodule credentials.
