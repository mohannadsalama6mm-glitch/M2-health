import { test, expect } from "@playwright/test";
const routes = [
  "/dashboard",
  "/sales",
  "/sales/1042",
  "/catalog",
  "/catalog/new",
  "/catalog/p1",
  "/catalog/p1/edit",
  "/inventory",
  "/inventory/movements",
  "/inventory/counts",
  "/inventory/expiry",
  "/purchases",
  "/purchases/new",
  "/purchases/PO-1048",
  "/purchases/PO-1048/receive",
  "/suppliers",
  "/suppliers/s1",
  "/customers",
  "/customers/c1",
  "/reports",
  "/finance",
  "/employees",
  "/employees/e1",
  "/roles",
  "/branches",
  "/branches/b1",
  "/settings",
  "/backup",
  "/sync",
  "/notifications",
  "/audit-log",
];
for (const [width, height] of [
  [1366, 768],
  [1440, 900],
  [1920, 1080],
])
  test(`all product routes and viewport architecture ${width}`, async ({
    page,
  }) => {
    test.setTimeout(180000);
    const errors: string[] = [];
    page.on("pageerror", (e) => errors.push(e.message));
    page.on("console", (m) => {
      if (m.type() === "error") errors.push(m.text());
    });
    await page.setViewportSize({ width, height });
    for (const route of routes) {
      await page.goto(route);
      await expect(page.locator("main h1")).toBeVisible();
      expect(
        await page
          .locator("main")
          .evaluate((el) => el.scrollWidth <= el.clientWidth),
        route,
      ).toBe(true);
      expect(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= window.innerWidth,
        ),
        route,
      ).toBe(true);
      expect(
        await page.evaluate(
          () => document.body.scrollHeight <= window.innerHeight,
        ),
        route,
      ).toBe(true);
      const before = await page.locator(".sidebar").boundingBox();
      const topbar = await page.locator(".topbar").boundingBox();
      await page.locator("main").evaluate((el) => {
        el.scrollTop = el.scrollHeight;
      });
      expect(await page.locator(".sidebar").boundingBox(), route).toEqual(
        before,
      );
      expect(await page.locator(".topbar").boundingBox(), route).toEqual(
        topbar,
      );
      expect(await page.evaluate(() => window.scrollY), route).toBe(0);
      if (
        [
          "/sales",
          "/catalog",
          "/purchases/PO-1048/receive",
          "/settings",
        ].includes(route)
      ) {
        await page.locator("main").evaluate((el) => (el.scrollTop = 0));
        await page.screenshot({
          path: `test-results/${route.replaceAll("/", "-")}-${width}.png`,
        });
      }
    }
    expect(errors).toEqual([]);
  });
test("module loading empty error and no results", async ({ page }) => {
  await page.goto("/catalog");
  const state = page.getByLabel("Preview state");
  await state.selectOption("loading");
  await expect(page.getByText("Loading products…")).toBeVisible();
  await state.selectOption("empty");
  await expect(
    page.getByRole("heading", { name: "No products yet" }),
  ).toBeVisible();
  await state.selectOption("error");
  await expect(
    page.getByText("This workspace could not be loaded"),
  ).toBeVisible();
  await page.getByRole("button", { name: "Retry preview" }).click();
  await page
    .getByRole("searchbox", { name: "Search Products" })
    .fill("unmatched medicine");
  await expect(page.getByText("No matching results")).toBeVisible();
});
test("POS keyboard search hold resume complete and receipt", async ({
  page,
}) => {
  await page.goto("/sales");
  await page
    .getByRole("searchbox", { name: "POS product search" })
    .fill("Panadol");
  await page
    .getByRole("searchbox", { name: "POS product search" })
    .press("Enter");
  await expect(
    page.getByRole("spinbutton", { name: "Quantity", exact: true }),
  ).toHaveValue("1");
  await page.getByRole("button", { name: "Hold sale", exact: true }).click();
  await page.getByRole("button", { name: "Held sales (1)" }).click();
  await page.getByRole("button", { name: "Resume", exact: true }).click();
  await page.getByLabel("Cash received (EGP)").fill("100");
  await page.getByRole("button", { name: "Complete sale · demo" }).click();
  await page.getByRole("button", { name: "Confirm demo sale" }).click();
  await expect(
    page.getByRole("heading", { name: "Receipt preview" }),
  ).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(
    page.getByRole("heading", { name: "Ready for your next sale" }),
  ).toBeVisible();
});
test("catalog add edit and in-memory reset", async ({ page }) => {
  await page.goto("/catalog/new");
  await page.getByLabel("Brand / product name").fill("Demo product QA");
  await page.getByLabel("Scientific name").fill("Demo ingredient");
  await page.getByLabel("Manufacturer", { exact: true }).fill("Demo maker");
  await page.getByLabel("Strength", { exact: true }).fill("10 mg");
  await page
    .getByLabel("Ingredients 1", { exact: true })
    .fill("Demo ingredient");
  await page.getByLabel("Packages 1", { exact: true }).fill("Box · 10 tablets");
  await page.getByLabel("Barcodes 1", { exact: true }).fill("6229999999991");
  await page.getByRole("button", { name: "Save demo product" }).click();
  await expect(
    page.getByRole("heading", { name: "Demo product QA" }),
  ).toBeVisible();
  await page.getByRole("link", { name: "Edit product" }).click();
  await page.getByLabel("Brand / product name").fill("Updated QA product");
  await page.getByRole("button", { name: "Save demo product" }).click();
  await expect(
    page.getByRole("heading", { name: "Updated QA product" }),
  ).toBeVisible();
  await page.reload();
  await expect(
    page.getByRole("heading", { name: "Product not found" }),
  ).toBeVisible();
});
test("purchase receiving and notification state", async ({ page }) => {
  await page.goto("/purchases/PO-1048/receive");
  for (const id of ["p1", "p2", "p7"])
    await page.getByLabel(`Batch ${id}`, { exact: true }).fill(`QA-${id}`);
  await page.getByRole("button", { name: "Review receiving" }).click();
  await page.getByRole("button", { name: "Confirm", exact: true }).click();
  await expect(
    page.getByText("Received", { exact: true }).first(),
  ).toBeVisible();
  await page
    .getByRole("navigation", { name: "Pharmacy navigation" })
    .getByRole("link", { name: "Notifications", exact: true })
    .click();
  await page.getByRole("button", { name: "Mark all read" }).click();
  await expect(page.getByText("0 unread")).toBeVisible();
});
test("stock count review completion and backup destructive preview", async ({
  page,
}) => {
  await page.goto("/inventory/counts");
  await page.getByRole("button", { name: "SC-003", exact: true }).click();
  await page.getByLabel("Count Panadol 500 mg").fill("118");
  await page.getByRole("button", { name: "Review differences" }).click();
  await page
    .getByRole("button", { name: "Complete count", exact: true })
    .click();
  await page.getByRole("button", { name: "Confirm", exact: true }).click();
  await expect(page.getByText("Count completed in this preview")).toBeVisible();
  await page.keyboard.press("Escape");
  await page.goto("/backup");
  await page
    .getByRole("button", { name: "Restore", exact: true })
    .first()
    .click();
  await expect(page.getByText("Review this action carefully")).toBeVisible();
  await page
    .getByLabel("Type RESTORE to review this destructive workflow")
    .fill("RESTORE");
  await page.getByLabel("Restore reason").fill("UI verification");
  await page
    .getByRole("button", { name: "Review restore", exact: true })
    .click();
  await expect(page.getByText("Restore preview reviewed")).toBeVisible();
});
