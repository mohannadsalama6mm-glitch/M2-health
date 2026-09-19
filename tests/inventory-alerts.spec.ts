import { test, expect, type Page } from "@playwright/test";
import { installAlertsDouble } from "./inventory-alerts-double";
test.beforeEach(async ({ page }) => {
  await installAlertsDouble(page);
});
async function query(page: Page, command: string) {
  return page.evaluate(
    (c) =>
      (
        window as unknown as {
          alertCalls: { command: string; args?: { query: unknown } }[];
        }
      ).alertCalls
        .filter((x) => x.command === c)
        .at(-1)?.args?.query,
    command,
  );
}
test("expired list reads native stock and cost", async ({ page }) => {
  await page.goto("/inventory/expiry");
  await expect(
    page.getByRole("table", { name: "Expiry batches" }),
  ).toContainText("DEV EXPIRY");
  await expect(page.getByRole("table")).toContainText("1.00 EGP");
  expect(await query(page, "list_expiry_inventory")).toMatchObject({
    expired: true,
  });
});
test("30 60 90 day filters use disjoint database windows", async ({ page }) => {
  await page.goto("/inventory/expiry");
  for (const [value, min, max] of [
    ["30", 0, 30],
    ["60", 31, 60],
    ["90", 61, 90],
  ] as const) {
    await page.getByLabel("Expiry window").selectOption(value);
    await expect
      .poll(() => query(page, "list_expiry_inventory"))
      .toMatchObject({
        expired: false,
        minimumDays: min,
        maximumDays: max,
        offset: 0,
      });
  }
});
test("configured low stock displays explicit levels", async ({ page }) => {
  await page.goto("/inventory/low-stock");
  await expect(
    page.getByRole("table", { name: "Configured low stock" }),
  ).toContainText("3 / 1 / 10");
});
test("out of stock filter displays zero and sends filter", async ({ page }) => {
  await page.goto("/inventory/low-stock");
  await page.getByLabel("Stock condition").selectOption("out");
  await expect
    .poll(() => query(page, "list_low_stock_inventory"))
    .toMatchObject({ outOfStock: true });
  await expect(
    page.getByRole("table").locator("tbody tr").first(),
  ).toContainText("0");
});
test("branch settings persist through typed save", async ({ page }) => {
  await page.goto("/inventory/alerts");
  await page.getByLabel("Warning days").fill("60");
  await page.getByLabel("Critical days").fill("15");
  await page.getByRole("button", { name: "Save alert settings" }).click();
  await expect
    .poll(() =>
      page.evaluate(
        () =>
          (
            window as unknown as {
              alertCalls: { command: string; args: { input: unknown } }[];
            }
          ).alertCalls.find(
            (c) => c.command === "update_inventory_alert_settings",
          )?.args.input,
      ),
    )
    .toMatchObject({ expiryWarningDays: 60, criticalExpiryDays: 15 });
});
test("manual evaluation preserves one rendered logical alert", async ({
  page,
}) => {
  await page.goto("/inventory/alerts");
  await expect(page.getByRole("table")).toBeVisible();
  for (let i = 0; i < 3; i++) {
    await page
      .getByRole("button", { name: "Refresh Alerts", exact: true })
      .click();
    await expect(page.getByRole("table")).toBeVisible();
  }
  await expect(page.getByRole("table").locator("tbody tr")).toHaveCount(1);
  expect(
    await page.evaluate(
      () =>
        (
          window as unknown as { alertCalls: { command: string }[] }
        ).alertCalls.filter((c) => c.command === "evaluate_inventory_alerts")
          .length,
    ),
  ).toBeGreaterThanOrEqual(4);
});
test("active resolved filters use backend status", async ({ page }) => {
  await page.goto("/inventory/alerts");
  await page.getByLabel("Alert status").selectOption("RESOLVED");
  await expect
    .poll(() => query(page, "list_inventory_alerts"))
    .toMatchObject({ status: "RESOLVED" });
  await expect(page.getByRole("table")).toContainText("RESOLVED");
});
test("type severity dates combine in database filter", async ({ page }) => {
  await page.goto("/inventory/alerts");
  await page.getByLabel("Alert type").selectOption("LOW_STOCK");
  await page.getByLabel("Severity", { exact: true }).selectOption("MEDIUM");
  await page.getByLabel("Updated from").fill("2026-09-01");
  await page.getByLabel("Updated through").fill("2026-09-30");
  await expect
    .poll(() => query(page, "list_inventory_alerts"))
    .toMatchObject({
      alertType: "LOW_STOCK",
      severity: "MEDIUM",
      fromDate: "2026-09-01",
      toDate: "2026-09-30",
    });
});
test("FEFO inspection preserves native order", async ({ page }) => {
  await page.goto("/inventory/fefo");
  await expect(
    page.getByLabel("FEFO product").locator("option"),
  ).not.toHaveCount(1);
  await page.getByLabel("FEFO product").selectOption({ index: 1 });
  await expect(
    page.getByLabel("FEFO package").locator("option"),
  ).not.toHaveCount(1);
  await page.getByLabel("FEFO package").selectOption({ index: 1 });
  const rows = page
    .getByRole("table", { name: "FEFO batches" })
    .locator("tbody tr");
  await expect(rows.nth(0)).toContainText("KNOWN LOT");
  await expect(rows.nth(1)).toContainText("Unknown (last)");
});
test("empty inventory has no fabricated rows", async ({ page }) => {
  await page.goto("/inventory/alerts");
  await page.evaluate(() => {
    (window as unknown as { alertEmpty: boolean }).alertEmpty = true;
  });
  await page
    .getByRole("button", { name: "Refresh Alerts", exact: true })
    .click();
  await expect(page.getByText("No matching inventory records.")).toBeVisible();
});
test("backend failure visible and refresh recovers", async ({ page }) => {
  await page.goto("/inventory/alerts");
  await page.evaluate(() => {
    (window as unknown as { alertFailure: string }).alertFailure =
      "DEV database unavailable";
  });
  await page
    .getByRole("button", { name: "Refresh Alerts", exact: true })
    .click();
  await expect(page.getByText("DEV database unavailable")).toBeVisible();
  await page.evaluate(() => {
    (window as unknown as { alertFailure: string }).alertFailure = "";
  });
  await page
    .getByRole("button", { name: "Refresh Alerts", exact: true })
    .click();
  await expect(page.getByRole("table")).toBeVisible();
});
test("pagination uses bounded native offset", async ({ page }) => {
  await page.goto("/inventory/alerts");
  await page.evaluate(() => {
    (window as unknown as { alertTotal: number }).alertTotal = 41;
  });
  await page
    .getByRole("button", { name: "Refresh Alerts", exact: true })
    .click();
  await page.getByRole("button", { name: "Next", exact: true }).click();
  await expect
    .poll(() => query(page, "list_inventory_alerts"))
    .toMatchObject({ limit: 20, offset: 20 });
});
test("history displayed and expiry action only navigates", async ({ page }) => {
  await page.goto("/inventory/alerts");
  await page.getByRole("button", { name: "History", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Alert history" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Open expiry write-off workflow" }),
  ).toHaveAttribute("href", /\/inventory\/documents\?package=pkg-1&batch=batch-1&new=WRITE_OFF&action=EXPIRY/);
  expect(
    await page.evaluate(() =>
      (
        window as unknown as { alertCalls: { command: string }[] }
      ).alertCalls.some((c) => c.command === "post_inventory_document"),
    ),
  ).toBe(false);
});
test("dismiss is advisory native request", async ({ page }) => {
  await page.goto("/inventory/alerts");
  await page.getByRole("button", { name: "Dismiss", exact: true }).click();
  await expect
    .poll(() =>
      page.evaluate(() =>
        (
          window as unknown as { alertCalls: { command: string }[] }
        ).alertCalls.some((c) => c.command === "dismiss_inventory_alert"),
      ),
    )
    .toBe(true);
});
