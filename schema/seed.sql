-- Idempotent demo/seed data for des-web.
--
-- Run AFTER the schema has converged (scripts/dpm.sh apply, or scripts/dev-db.sh
-- which does both):  psql "$TARGET_DATABASE_URL" -f schema/seed.sql
--
-- Every insert is `on conflict do nothing` with fixed ids, so re-running is a
-- no-op. Seeds cover both the pg-defs contract tables the pages display
-- (des_soccer_*, des_fel_elevator_*) and the des-web overlay tables
-- (des_web_sims catalog, des_web_routing_solves).

begin;

-- ---------------------------------------------------------------------------
-- pg-defs: soccer learning + tournaments (displayed at /soccer)
-- ---------------------------------------------------------------------------

insert into des_soccer_learning_experiments (id, slug, display_name, description, status, config, labels)
values (
  'a11ce000-0000-4000-8000-00000000de50',
  'des-web/seed-experiment',
  'des-web demo experiment',
  'Seeded by des-web.rs (discrete-event-systems) so the copied soccer pages have data to display. Mirrors the shape written by the akrion soccer learning stack.',
  'active',
  '{"source": "des-web seed", "learningMode": "q-table"}'::jsonb,
  '["des-web", "seed"]'::jsonb
)
on conflict (id) do nothing;

insert into des_soccer_tournaments
  (id, experiment_id, tournament_date, seed, learning_mode, format, team_count,
   match_count, matches_played, champion_team_id, runner_up_team_id,
   third_place_team_id, wall_time_seconds, status, finished_at)
values
  (900001, 'a11ce000-0000-4000-8000-00000000de50', '2026-07-10', 4242, 'q-table',
   '{"type": "single-elimination", "rounds": 3, "thirdPlaceMatch": true}'::jsonb,
   8, 8, 8, 3, 6, 1, 512.7, 'completed', now() - interval '6 days'),
  (900002, 'a11ce000-0000-4000-8000-00000000de50', '2026-07-16', 7331, 'neural-td',
   '{"type": "single-elimination", "rounds": 3, "thirdPlaceMatch": true}'::jsonb,
   8, 8, 5, null, null, null, null, 'running', null)
on conflict (id) do nothing;

-- Keep the sequence ahead of the explicit ids above.
select setval(
  pg_get_serial_sequence('des_soccer_tournaments', 'id'),
  greatest(900100, (select max(id) from des_soccer_tournaments))
);

insert into des_soccer_tournament_matches
  (tournament_id, match_index, stage, home_team_id, away_team_id, home_goals,
   away_goals, shootout_winner_team_id, home_training_steps, away_training_steps)
values
  (900001, 0, 'quarterfinal', 1, 8, 2, 1, null, 120000, 118500),
  (900001, 1, 'quarterfinal', 2, 7, 0, 0, 7,    121400, 119900),
  (900001, 2, 'quarterfinal', 3, 6, 3, 1, null, 122800, 117200),
  (900001, 3, 'quarterfinal', 4, 5, 1, 2, null, 118000, 123600),
  (900001, 4, 'semifinal',    1, 7, 1, 3, null, 131000, 130400),
  (900001, 5, 'semifinal',    3, 5, 2, 0, null, 133900, 129700),
  (900001, 6, 'third-place',  1, 5, 4, 2, null, 140100, 138800),
  (900001, 7, 'final',        7, 3, 1, 2, null, 152600, 151900)
on conflict (tournament_id, match_index) do nothing;

insert into des_soccer_learning_runs
  (id, experiment_id, runner_id, seed, episode_index, status, score_home,
   score_away, home_goal_diff, away_goal_diff, home_outcome, away_outcome,
   fitness_micros, duration_ticks, elapsed_millis, transitions, summary)
values
  ('b0000000-0000-4000-8000-000000000001', 'a11ce000-0000-4000-8000-00000000de50',
   'des-web-seed-runner', 1001, 0, 'completed', 3, 1, 2, -2, 'win', 'loss',
   1730000, 540000, 8215, 412, '{"phase": "group", "note": "seeded"}'::jsonb),
  ('b0000000-0000-4000-8000-000000000002', 'a11ce000-0000-4000-8000-00000000de50',
   'des-web-seed-runner', 1002, 1, 'completed', 1, 1, 0, 0, 'draw', 'draw',
   1245000, 540000, 7980, 388, '{"phase": "group", "note": "seeded"}'::jsonb),
  ('b0000000-0000-4000-8000-000000000003', 'a11ce000-0000-4000-8000-00000000de50',
   'des-web-seed-runner', 1003, 2, 'completed', 0, 2, -2, 2, 'loss', 'win',
   918000, 540000, 8402, 405, '{"phase": "group", "note": "seeded"}'::jsonb),
  ('b0000000-0000-4000-8000-000000000004', 'a11ce000-0000-4000-8000-00000000de50',
   'des-web-seed-runner', 1004, 3, 'completed', 2, 0, 2, -2, 'win', 'loss',
   1812000, 540000, 8110, 421, '{"phase": "knockout", "note": "seeded"}'::jsonb),
  ('b0000000-0000-4000-8000-000000000005', 'a11ce000-0000-4000-8000-00000000de50',
   'des-web-seed-runner', 1005, 4, 'failed', 0, 0, 0, 0, 'draw', 'draw',
   0, 12000, 302, 9, '{"error": "seeded failure example"}'::jsonb),
  ('b0000000-0000-4000-8000-000000000006', 'a11ce000-0000-4000-8000-00000000de50',
   'des-web-seed-runner', 1006, 5, 'completed', 4, 3, 1, -1, 'win', 'loss',
   1954000, 540000, 8544, 447, '{"phase": "knockout", "note": "seeded"}'::jsonb)
on conflict (id) do nothing;

-- ---------------------------------------------------------------------------
-- pg-defs: elevator FEL learning (displayed at /elevator)
-- ---------------------------------------------------------------------------

insert into des_fel_elevator_learning_runs
  (id, run_label, scenario_slug, status, dispatch_policy, seed, floors, shafts,
   capacity, travel_seconds_micros, dwell_seconds_micros, arrival_rate_micros,
   horizon_seconds_micros, events, arrivals, boarded, served, mean_wait_micros,
   dispatch_decisions, pomdp_belief_updates, online_learning_updates,
   config, metrics, artifact)
values
  ('c0000000-0000-4000-8000-000000000001', 'seed: LOOK baseline', 'des-web/highrise-demo',
   'imported', 'look', 4242, 24, 4, 12, 1800000, 9000000, 350000, 28800000000,
   184232, 10081, 9964, 9871, 31400000, 2210, 0, 0,
   '{"source": "des-web seed"}'::jsonb,
   '{"p95WaitSeconds": 71.2, "meanWaitSeconds": 31.4}'::jsonb,
   '{"html": "/elevator/player", "source": "discrete-event-system/out/elevator.html"}'::jsonb),
  ('c0000000-0000-4000-8000-000000000002', 'seed: MDP table policy', 'des-web/highrise-demo',
   'imported', 'mdp-table', 4242, 24, 4, 12, 1800000, 9000000, 350000, 28800000000,
   184232, 10081, 10012, 9958, 24700000, 2210, 0, 41800,
   '{"source": "des-web seed"}'::jsonb,
   '{"p95WaitSeconds": 55.9, "meanWaitSeconds": 24.7}'::jsonb,
   '{"html": "/elevator/player", "source": "discrete-event-system/out/elevator.html"}'::jsonb),
  ('c0000000-0000-4000-8000-000000000003', 'seed: POMDP belief dispatch', 'des-web/highrise-demo',
   'imported', 'pomdp-belief', 4242, 24, 4, 12, 1800000, 9000000, 350000, 28800000000,
   184232, 10081, 10034, 9990, 21900000, 2210, 18345, 52600,
   '{"source": "des-web seed"}'::jsonb,
   '{"p95WaitSeconds": 49.1, "meanWaitSeconds": 21.9}'::jsonb,
   '{"html": "/elevator/player", "source": "discrete-event-system/out/elevator.html"}'::jsonb)
on conflict (id) do nothing;

insert into des_fel_elevator_dispatch_decisions
  (id, run_id, decision_index, sim_time_micros, call_floor, car_index, policy_kind, meta_data)
values
  ('d0000000-0000-4000-8000-000000000001', 'c0000000-0000-4000-8000-000000000001',
   0, 12400000, 7, 1, 'look', '{"direction": "up"}'::jsonb),
  ('d0000000-0000-4000-8000-000000000002', 'c0000000-0000-4000-8000-000000000001',
   1, 15100000, 18, 3, 'look', '{"direction": "down"}'::jsonb),
  ('d0000000-0000-4000-8000-000000000003', 'c0000000-0000-4000-8000-000000000002',
   0, 12400000, 7, 2, 'mdp-table', '{"qValue": 0.412}'::jsonb),
  ('d0000000-0000-4000-8000-000000000004', 'c0000000-0000-4000-8000-000000000002',
   1, 15100000, 18, 0, 'mdp-table', '{"qValue": 0.377}'::jsonb),
  ('d0000000-0000-4000-8000-000000000005', 'c0000000-0000-4000-8000-000000000003',
   0, 12400000, 7, 2, 'pomdp-belief', '{"beliefEntropy": 1.24}'::jsonb),
  ('d0000000-0000-4000-8000-000000000006', 'c0000000-0000-4000-8000-000000000003',
   1, 15100000, 18, 1, 'pomdp-belief', '{"beliefEntropy": 0.98}'::jsonb)
on conflict (id) do nothing;

-- ---------------------------------------------------------------------------
-- des-web overlay: sims catalog (rendered on the home page)
-- ---------------------------------------------------------------------------

insert into des_web_sims
  (slug, title, blurb, kind, page_route, source_service, engine, tags, sort_order)
values
  ('soccer/planner', 'Soccer rotation planner',
   'Interactive MIP/MDP rotation planner for 11-a-side squads. Page copied from soccer-sim-game-engine.rs (planner_ui.html); live solves proxy to des-rs when DES_UPSTREAM_URL is set.',
   'game', '/soccer/planner', 'des-rs + soccer-sim-game-engine.rs',
   'soccer_engine soccer_planner (MIP + MDP fallback)',
   '["soccer", "game", "planner", "mip"]'::jsonb, 10),
  ('soccer/data', 'Soccer learning & tournaments',
   'Tournaments, knockout matches and Q-learning runs from the akrion soccer stack, read from the shared pg-defs des_soccer_* tables via SeaORM.',
   'dashboard', '/soccer', 'pg-defs (des_soccer_*)', 'sea-orm + maud + htmx',
   '["soccer", "learning", "pg-defs"]'::jsonb, 20),
  ('routing/live', 'Optimal routing — live VRP/TSP',
   'Multi-start construction + 2-opt vehicle routing over a generated map, drawn live on canvas. Page copied from dd-routing-server; the solver runs in-process here and finished solves persist to Postgres.',
   'solver', '/routing', 'routing-server-rs', 'in-process multi-start NN + 2-opt',
   '["routing", "vrp", "tsp", "maps"]'::jsonb, 30),
  ('track3t/factory-floor', 'Track3t factory floor',
   'Warehouse floor Track3t comparison — the full discrete-event animation artifact rendered by the DES engine (31 MB page served gzip, ~667 KB over the wire).',
   'artifact', '/track3t', 'discrete-event-system (out/factory-floor-track3t.html)',
   'des-engine track3t warehouse model',
   '["track3t", "warehouse", "des"]'::jsonb, 40),
  ('elevator/data', 'Elevator dispatch learning',
   'FEL elevator learning runs (LOOK vs MDP vs POMDP dispatch) from the shared pg-defs des_fel_elevator_* tables, with per-decision drill-down.',
   'dashboard', '/elevator', 'pg-defs (des_fel_elevator_*)', 'sea-orm + maud + htmx',
   '["elevator", "fel", "pomdp", "pg-defs"]'::jsonb, 50),
  ('elevator/player', 'Elevator high-rise player',
   'Animated high-rise elevator DES artifact (event-by-event playback) rendered by the DES engine.',
   'artifact', '/elevator/player', 'discrete-event-system (out/elevator.html)',
   'des-engine elevator FEL model', '["elevator", "des", "animation"]'::jsonb, 60),
  ('soccer/mip-artifacts', 'Soccer MIP/LP solver traces',
   'Rendered solver-trace artifacts for the soccer lineup IP: MIP feasible search and LP relaxation, straight from the DES engine output.',
   'artifact', '/artifacts', 'discrete-event-system (out/soccer-IP-*.html)',
   'des-engine IP/LP solver', '["soccer", "mip", "lp"]'::jsonb, 70)
on conflict (slug) do nothing;

-- ---------------------------------------------------------------------------
-- des-web overlay: one completed example routing solve
-- ---------------------------------------------------------------------------

insert into des_web_routing_solves
  (id, status, stop_count, vehicles, restarts_total, restarts_done, improvements,
   seed, best_distance, depot_index, stops, routes, finished_at)
values (
  'e0000000-0000-4000-8000-000000000001', 'completed', 12, 2, 24, 24, 7, 42,
  312.6, 0,
  '[{"x": 50.0, "y": 50.0}, {"x": 12.1, "y": 74.3}, {"x": 25.7, "y": 31.2},
    {"x": 81.4, "y": 22.9}, {"x": 66.0, "y": 88.1}, {"x": 40.2, "y": 9.6},
    {"x": 92.5, "y": 55.8}, {"x": 8.9, "y": 44.0}, {"x": 71.3, "y": 68.7},
    {"x": 33.8, "y": 92.4}, {"x": 58.6, "y": 15.2}, {"x": 19.4, "y": 60.1}]'::jsonb,
  '[[0, 2, 5, 10, 3, 6, 8], [0, 7, 11, 1, 9, 4]]'::jsonb,
  now() - interval '2 days'
)
on conflict (id) do nothing;

commit;
