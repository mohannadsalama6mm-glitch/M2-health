import { expect, test } from "@playwright/test";
import { installCatalogDouble } from "./catalog-double";

test.beforeEach(async ({ page }) => {
  await installCatalogDouble(page);
  await page.goto("/sync");
});

test("sync remains fail-closed while Phase 8C rollout gates are disabled", async ({ page }) => {
  await expect(page.getByRole("heading", { name: "Sync / cloud status" })).toBeVisible();
  await expect(page.getByText("Local-only mode")).toBeVisible();
  await expect(page.getByText("Not enrolled", { exact: true })).toBeVisible();
  await expect(page.getByText("Disabled", { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Synchronize now" })).toBeDisabled();
});

test("cloud sign-in clears the password and never persists credentials", async ({ page }) => {
  await page.getByLabel("Cloud email").fill("operator@example.test");
  await page.getByLabel("Cloud password").fill("transient-password");
  await page.getByRole("button", { name: "Sign in to cloud" }).click();

  await expect(page.getByText("Authenticated cloud subject:")).toBeVisible();
  await expect(page.getByText("cloud-user-test")).toBeVisible();
  await expect(page.getByRole("button", { name: "Synchronize now" })).toBeDisabled();
  expect(await page.evaluate(() => ({
    local: Object.values(localStorage),
    session: Object.values(sessionStorage),
    html: document.documentElement.innerHTML,
  }))).toEqual({ local: [], session: [], html: expect.not.stringContaining("transient-password") });

  await page.getByRole("button", { name: "Sign out" }).click();
  await expect(page.getByRole("button", { name: "Sign in to cloud" })).toBeVisible();
});

test("an authorized operator can dismiss a conflict with an audit note", async ({ page }) => {
  await page.evaluate(() => {
    (window as unknown as { cloudConflicts: Array<Record<string, unknown>> }).cloudConflicts = [{
      id: "conflict-1", eventId: "event-1", entityType: "inventory_movement", entityId: "movement-1",
      reasonCode: "REVISION_CONFLICT", detail: "Remote stock would become negative.", status: "OPEN",
      createdAt: "2026-10-10T00:00:00Z", resolvedAt: null, resolutionAction: null,
      resolvedByUserId: null, resolutionNote: null,
    }];
  });
  await page.evaluate(() => { history.pushState({}, "", "/dashboard"); dispatchEvent(new PopStateEvent("popstate")); });
  await page.evaluate(() => { history.pushState({}, "", "/sync"); dispatchEvent(new PopStateEvent("popstate")); });
  await expect(page.getByText("Remote stock would become negative.")).toBeVisible();
  await page.getByLabel("Resolution note").fill("Reviewed against the physical ledger");
  await page.getByRole("button", { name: "Dismiss" }).click();
  await expect(page.getByText("Conflict dismissed with an audit note")).toBeVisible();
});

for (const viewport of [
  { width: 1366, height: 768 },
  { width: 1440, height: 900 },
  { width: 1920, height: 1080 },
]) {
  test(`sync readiness fits ${viewport.width}x${viewport.height}`, async ({ page }) => {
    await page.setViewportSize(viewport);
    await page.reload();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
    await expect(page.getByRole("button", { name: "Synchronize now" })).toBeVisible();
  });
}
