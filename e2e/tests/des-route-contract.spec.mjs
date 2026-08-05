import { test, expect } from "@playwright/test";

test("canonical service-local aliases resolve", async ({ request }) => {
  for (const path of [
    "/models",
    "/games/soccer",
    "/games/elevator",
    "/tools/routing",
    "/labs/factory-floor-track3t",
    "/runs/test-run-42",
  ]) {
    const response = await request.get(path);
    expect(response.ok(), `${path} should resolve`).toBeTruthy();
  }
});

test("route catalog publishes the /des contract", async ({ request }) => {
  const response = await request.get("/api/v1/catalog");
  expect(response.ok()).toBeTruthy();
  const catalog = await response.json();
  expect(catalog.schema).toBe("des.route-catalog.v1");
  expect(catalog.basePath).toBe("/des");
  expect(catalog.pages.soccer).toBe("/des/games/soccer");
  expect(catalog.api.solve).toBe("/des/api/v1/solve");
  expect(catalog.ownership.application).toBe(
    "discrete-event-systems/des-web.rs",
  );
});

test("trusted /des mount header rewrites browser links and htmx endpoints", async ({
  request,
}) => {
  const response = await request.get("/", {
    headers: { "X-Forwarded-Prefix": "/des" },
  });
  expect(response.ok()).toBeTruthy();
  const html = await response.text();
  expect(html).toContain('href="/des/games/soccer"');
  expect(html).toContain('href="/des/games/soccer/planner"');
  expect(html).toContain('href="/des/tools/routing"');
  expect(html).toContain('href="/des/labs/factory-floor-track3t"');
  expect(html).toContain('hx-get="/des/partials/sims"');
  expect(html).toContain('href="/des/assets/app.css"');
});

test("without the mount header local root links remain compatible", async ({
  request,
}) => {
  const response = await request.get("/");
  const html = await response.text();
  expect(html).toContain('href="/soccer"');
  expect(html).toContain('hx-get="/partials/sims"');
  expect(html).not.toContain('href="/des/games/soccer"');
});
