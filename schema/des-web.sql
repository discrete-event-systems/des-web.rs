-- des-web overlay schema — tables owned by THIS repo, layered on top of the
-- shared pg-defs contract (libs/pg-defs/schema/schema.sql).
--
-- scripts/dpm.sh concatenates pg-defs schema.sql + this file into one
-- desired-state source and lets dpm (declarative-postgres-migrate) converge the
-- target database onto it. Do not write imperative migrations for these tables;
-- edit the desired state here and run scripts/dpm.sh diff/verify/apply.
--
-- Style note: status/kind columns are `text` with IN-list CHECKs (matching
-- des_soccer_tournaments in pg-defs) rather than varchar IN-lists, which hit a
-- known dpm deparse fixed-point bug (varchar IN-list CHECKs never converge).

-- Catalog of sim/game pages this server serves. The home page renders this
-- table; rows are seeded by schema/seed.sql and safe to extend at runtime.
create table if not exists des_web_sims (
  id bigserial primary key,
  slug text not null unique,
  title text not null,
  blurb text default '' not null,
  kind text not null,
  page_route text not null,
  source_service text not null,
  engine text default '' not null,
  tags jsonb default '[]'::jsonb not null,
  sort_order integer default 100 not null,
  is_enabled boolean default true not null,
  created_at timestamptz default now() not null,
  updated_at timestamptz default now() not null,
  constraint des_web_sims_slug_format_chk
    check (slug ~ '^[a-z0-9][a-z0-9._/-]{0,158}$'),
  constraint des_web_sims_kind_chk
    check (kind in ('game', 'sim', 'solver', 'artifact', 'dashboard')),
  constraint des_web_sims_title_size_chk
    check (octet_length(title) between 1 and 240),
  constraint des_web_sims_tags_array_chk
    check (jsonb_typeof(tags) = 'array')
);

-- Finished VRP/TSP solves from the /routing page (in-process port of
-- dd-routing-server's multi-start construction + 2-opt). Live solves stay in
-- memory; completed solves are persisted here so the page has data to display.
create table if not exists des_web_routing_solves (
  id uuid primary key,
  status text default 'running' not null,
  stop_count integer not null,
  vehicles integer not null,
  restarts_total integer not null,
  restarts_done integer default 0 not null,
  improvements integer default 0 not null,
  seed bigint not null,
  best_distance double precision,
  depot_index integer default 0 not null,
  stops jsonb not null,
  routes jsonb default '[]'::jsonb not null,
  created_at timestamptz default now() not null,
  updated_at timestamptz default now() not null,
  finished_at timestamptz,
  constraint des_web_routing_solves_status_chk
    check (status in ('running', 'completed', 'failed')),
  constraint des_web_routing_solves_counts_chk
    check (
      stop_count >= 3
      and vehicles >= 1
      and restarts_total >= 1
      and restarts_done >= 0
      and improvements >= 0
      and depot_index >= 0
    ),
  constraint des_web_routing_solves_seed_chk
    check (seed >= 0),
  constraint des_web_routing_solves_stops_array_chk
    check (jsonb_typeof(stops) = 'array'),
  constraint des_web_routing_solves_routes_array_chk
    check (jsonb_typeof(routes) = 'array')
);

create index if not exists des_web_routing_solves_created_at_idx
  on des_web_routing_solves (created_at desc);
