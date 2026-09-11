import { test, expect } from "@playwright/test";

// These tests drive a real browser against a running des-web (with the seeded
// dev/CI database), exercising the maud pages, the htmx partial loads, the
// client-side routing solver, the gzip-vendored artifacts, and the hardening
// headers. They assume the seed data from schema/seed.sql is present.

test("home page renders the catalog via htmx", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveTitle(/des-web/);
  // #catalog is filled by hx-get /partials/sims on load.
  await expect(page.locator("#catalog .card").first()).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Soccer rotation planner" }),
  ).toBeVisible();
  // db-status chip loads via htmx and reports a state.
  await expect(page.locator("#db-status .chip")).toBeVisible();
});

test("responses carry the hardening security headers", async ({ page }) => {
  const resp = await page.goto("/");
  const h = resp.headers();
  expect(h["content-security-policy"]).toContain("default-src 'self'");
  expect(h["content-security-policy"]).toContain("frame-ancestors 'none'");
  expect(h["x-frame-options"]).toBe("DENY");
  expect(h["x-content-type-options"]).toBe("nosniff");
  expect(h["referrer-policy"]).toBe("strict-origin-when-cross-origin");
});

test("soccer dashboard loads pg-defs data via htmx partials", async ({
  page,
}) => {
  await page.goto("/soccer");
  await expect(page.getByRole("heading", { name: /Soccer/ })).toBeVisible();
  // Seeded knockout matches render in the matches partial.
  await expect(page.getByText("quarterfinal").first()).toBeVisible();
  // Seeded learning runs render (a runner id from schema/seed.sql).
  await expect(page.getByText("des-web-seed-runner").first()).toBeVisible();
});

test("elevator dashboard shows seeded dispatch-policy runs", async ({
  page,
}) => {
  await page.goto("/elevator");
  await expect(page.getByText("pomdp-belief").first()).toBeVisible();
});

test("routing solver runs to completion and persists", async ({ page }) => {
  await page.goto("/routing");
  await page.locator("#count").fill("60");
  await page.locator("#vehicles").fill("3");
  await page.locator("#restarts").fill("12");
  await page.getByRole("button", { name: /Generate/ }).click();

  // The dashboard JS polls /api/solve/{id} and flips status to "completed".
  await expect(page.locator("#status")).toHaveText("completed", {
    timeout: 25_000,
  });
  const distance = await page.locator("#distance").textContent();
  expect(Number.parseFloat(distance)).toBeGreaterThan(0);
  expect(await page.locator("#progress").textContent()).toBe("12/12");

  // The persisted-solves table (htmx, /partials/routing/solves) is present.
  await expect(page.locator("#history table")).toBeVisible();
});

test("track3t artifact loads (served gzip, decoded by the browser)", async ({
  page,
}) => {
  await page.goto("/track3t");
  await expect(page).toHaveTitle(/Track3t/i);
});

test("artifacts index lists artifacts and links open them", async ({
  page,
}) => {
  await page.goto("/artifacts");
  const firstCard = page.locator(".card").first();
  await expect(firstCard).toBeVisible();
  await firstCard.click();
  // Any vendored artifact page has a <title>; just assert navigation happened.
  await expect(page).not.toHaveURL(/\/artifacts$/);
});

test("login shows the magic-link form or a not-configured notice", async ({
  page,
}) => {
  await page.goto("/login");
  const form = page.locator("form.auth-form");
  const notice = page.getByText(/not configured/i);
  await expect(form.or(notice).first()).toBeVisible();
});

test("unknown path returns the 404 page", async ({ page }) => {
  const resp = await page.goto("/no-such-page-here");
  expect(resp.status()).toBe(404);
  await expect(page.getByText("404")).toBeVisible();
});

test("keyboard can tab into a focusable control on home", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("#catalog .card").first()).toBeVisible();
  await page.keyboard.press("Tab");
  const tag = await page.evaluate(() => document.activeElement?.tagName);
  expect(["A", "BUTTON", "INPUT", "SELECT"]).toContain(tag);
});

test("home markup is HTMX/Maud, not React", async ({ page }) => {
  await page.goto("/");
  const html = await page.content();
  expect(html.toLowerCase()).not.toContain("react");
  expect(html).not.toContain("JSX");
  expect(html).toContain("hx-get");
});

test("login page has no console errors", async ({ page }) => {
  const errors = [];
  page.on("console", (message) => {
    if (message.type() === "error" && !/favicon/i.test(message.text())) {
      errors.push(message.text());
    }
  });
  await page.goto("/login");
  expect(errors).toEqual([]);
});
