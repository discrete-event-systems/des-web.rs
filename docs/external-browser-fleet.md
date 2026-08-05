# External browser automation fleet

`des-web.rs` keeps unit, schema, and in-repository Playwright coverage in this repository. Two independently versioned black-box suites live in `discrete-event-systems-test`:

- `des-web-playwright-e2e` — Playwright/Chromium contracts.
- `des-web-puppeteer-e2e` — Puppeteer/Chromium contracts using Node's built-in test runner.

`.github/workflows/external-browser-fleet.yml` calls both suites by immutable merged commit SHA. The caller remains this product repository, so its least-privilege `GITHUB_TOKEN` receives `packages: read` for the private `ghcr.io/discrete-event-systems/des-web.rs` image. Each called workflow checks out its own repository at the same pinned SHA, starts the immutable deployed image in mounted-path mode, drives a real browser, and retains framework-specific evidence.

The test repositories also publish static `.github/workflows/gha-indie-worker.yml` workflows. Those are intentionally limited to pinned checkout/setup actions plus the fixed Playwright or Puppeteer command shape accepted by `gha-indie-worker`. The suites default to the in-cluster `dd-des-web.default.svc.cluster.local:8130` service in that lane.

## Evidence

- Playwright uploads traces, screenshots, video, its HTML report, and server logs on failure.
- Puppeteer uploads a full-page home screenshot and failure server logs.
- GitHub Actions artifacts are retained for 14 days.

## Control-plane status

The `playwright` and `puppeteer` profiles are compiled into the build server. `discrete-event-systems-test` must remain in `BUILD_SERVER_ALLOWED_PROFILE_REPO_PREFIXES`. Workflow planning is available after operator authentication. Live indie-worker execution remains fail-closed while `BUILD_SERVER_GHA_WORKFLOW_EXECUTION_ENABLED=false`; enabling it requires a separate capacity and security review.

## Tracking

- Product GitHub Project: https://github.com/orgs/discrete-event-systems/projects/2
- Test GitHub Project: https://github.com/orgs/discrete-event-systems-test/projects/1
- Linear project: https://linear.app/denman/project/githubcomdiscrete-event-systems-4a3086ae0c45
