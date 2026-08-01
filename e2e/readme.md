# des-web browser e2e (Playwright)

Headless-Chromium end-to-end tests that drive a **running** des-web against the
seeded dev/CI database — the maud pages, the htmx partial loads, the client-side
routing solver, the gzip-vendored artifacts, and the hardening headers.

The server is started outside Playwright (it needs Postgres + schema + seed
first). Point the tests at it with `E2E_BASE_URL` (default
`http://localhost:8130`).

## Local

```sh
# from the repo root: start Postgres, converge + seed, run the server
scripts/dev-db.sh
DATABASE_URL=postgres://localhost:5432/des_web_dev cargo run &

cd e2e
npm install
npm run install-browsers      # playwright install --with-deps chromium
npm test                      # or: E2E_BASE_URL=http://localhost:8130 npm test
```

## CI

The `e2e` job in `.github/workflows/ci.yml` spins up Postgres 17, applies
`pg-defs/schema.sql` + `schema/des-web.sql`, loads `schema/seed.sql`, builds and
starts the release binary, waits for `/healthz`, then runs this suite. The HTML
report is uploaded as an artifact on failure.
