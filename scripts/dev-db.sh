#!/usr/bin/env bash
# Local dev bootstrap: start Homebrew Postgres 17, create the dev database,
# converge it onto the declarative schema (pg-defs + overlay) via dpm, and load
# the idempotent seed data. Local-dev convenience ONLY — for RDS/Supabase run
# scripts/dpm.sh diff/verify, review the SQL, then scripts/dpm.sh apply.
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# macOS: pg_ctl fails with "postmaster became multithreaded during startup"
# when the invoking shell has no usable locale.
export LC_ALL="${LC_ALL:-en_US.UTF-8}"

PGDATA="${PGDATA:-/opt/homebrew/var/postgresql@17}"
PGBIN="${PGBIN:-/opt/homebrew/opt/postgresql@17/bin}"
DEV_DB="${DEV_DB:-des_web_dev}"
DEV_SERVER_URL="${DEV_SERVER_URL:-postgres://localhost:5432/postgres}"
DEV_DB_URL="${DEV_DB_URL:-postgres://localhost:5432/$DEV_DB}"

mkdir -p "$repo_dir/tmp"

if ! "$PGBIN/pg_isready" -q -h localhost -p 5432 2>/dev/null; then
  echo "==> starting postgres ($PGDATA)"
  "$PGBIN/pg_ctl" -D "$PGDATA" -l "$repo_dir/tmp/postgres-dev.log" start
  for _ in $(seq 1 30); do
    "$PGBIN/pg_isready" -q -h localhost -p 5432 && break
    sleep 0.5
  done
fi

if ! "$PGBIN/psql" "$DEV_SERVER_URL" -Atc "select 1 from pg_database where datname = '$DEV_DB'" | grep -q 1; then
  echo "==> creating database $DEV_DB"
  "$PGBIN/createdb" -h localhost -p 5432 "$DEV_DB"
fi

echo "==> converging $DEV_DB_URL onto pg-defs + des-web schema (dpm apply)"
SHADOW_DATABASE_URL="$DEV_SERVER_URL" TARGET_DATABASE_URL="$DEV_DB_URL" \
  "$repo_dir/scripts/dpm.sh" apply --yes "$@"

echo "==> loading seed data"
"$PGBIN/psql" "$DEV_DB_URL" -v ON_ERROR_STOP=1 -q -f "$repo_dir/schema/seed.sql"

echo "==> done. run the server with:"
echo "    DATABASE_URL=$DEV_DB_URL cargo run"
