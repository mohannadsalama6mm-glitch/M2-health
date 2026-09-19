import { test, expect } from "@playwright/test";
import { installCatalogDouble } from "./catalog-double";

test("recovered runtime shows canonical stock and blocks every prototype route", async ({ page }) => {
  await installCatalogDouble(page);
  await page.addInitScript(() => {
    const w = window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> }; recoveryCalls: string[] };
    const original = w.__TAURI_INTERNALS__.invoke;
    w.recoveryCalls = [];
    w.__TAURI_INTERNALS__.invoke = async (command, args) => {
      w.recoveryCalls.push(command);
      if (command === "recovery_status") return { laterWorkflowsEnabled: false };
      if (command === "list_branch_inventory") return [{ branchId: "branch", productPackageId: "verified-package", batchId: null, quantity: 8 }];
      if (command === "list_stock_movements") return [{ id: "movement", productPackageId: "verified-package", batchId: "batch", movementType: "OPENING", quantityDelta: 10, reason: "Verified opening", createdAt: "2026-09-13" }];
      return original(command, args);
    };
  });
  await page.goto("/inventory");
  await expect(page.getByRole("table", { name: "Verified inventory balances" })).toContainText("8");
  await page.goto("/inventory/movements");
  await expect(page.getByRole("table", { name: "Verified stock movements" })).toContainText("Verified opening");
  for (const route of ["/sales", "/sales/123", "/purchases/new", "/purchases/123/receive", "/customers", "/suppliers", "/inventory/counts", "/inventory/expiry"]) {
    await page.goto(route);
    await expect(page.getByText("Posting is disabled")).toBeVisible();
    const calls = await page.evaluate(() => (window as unknown as { recoveryCalls: string[] }).recoveryCalls);
    expect(calls).not.toContain("complete_sale");
    expect(calls).not.toContain("complete_purchase");
    expect(calls).not.toContain("list_stock_counts");
    expect(calls).not.toContain("pos_search");
  }
  await page.goto("/catalog");
  await expect(page.getByText("1 results · local SQLite")).toBeVisible();
});

test("capability failure never enables posting", async ({ page }) => {
  await installCatalogDouble(page);
  await page.addInitScript(() => {
    const w = window as unknown as { __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> } };
    const original = w.__TAURI_INTERNALS__.invoke;
    w.__TAURI_INTERNALS__.invoke = async (command, args) => {
      if (command === "recovery_status") throw { code: "internal", message: "Unavailable" };
      if (command === "pos_search") throw new Error("Unsafe screen mounted");
      return original(command, args);
    };
  });
  await page.goto("/sales");
  await expect(page.getByText("Workflow availability could not be verified")).toBeVisible();
  await expect(page.getByRole("button", { name: /complete sale/i })).toHaveCount(0);
});
