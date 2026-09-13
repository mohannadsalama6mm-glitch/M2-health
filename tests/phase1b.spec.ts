import { test, expect } from "@playwright/test";
import { installCatalogDouble } from "./catalog-double";
import { installInventoryDouble } from "./inventory-double";
import { installSalesDouble } from "./sales-double";
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
      await expect(page.locator("main h1, main h3").first()).toBeVisible();
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
test("catalog grid in the browser explains the desktop requirement", async ({
  page,
}) => {
  await page.goto("/catalog");
  await expect(page.getByRole("heading", { name: "Products" })).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Catalog requires the desktop app" }),
  ).toBeVisible();
  await expect(
    page.getByRole("searchbox", { name: "Search Products" }),
  ).toHaveCount(0);
});
test("catalog add edit and detail routes require the desktop app", async ({
  page,
}) => {
  await page.goto("/catalog/new");
  await expect(page.getByRole("heading", { name: "Add product" })).toBeVisible();
  await expect(page.getByText("Desktop app required")).toBeVisible();
  await page.goto("/catalog/p1");
  await expect(
    page.getByRole("heading", { name: "Product details" }),
  ).toBeVisible();
  await page.goto("/catalog/p1/edit");
  await expect(page.getByRole("heading", { name: "Edit product" })).toBeVisible();
  await expect(
    page.getByText("Desktop app required").first(),
  ).toBeVisible();
});
test("POS keyboard search hold resume complete and receipt", async ({
  page,
}) => {
  await installCatalogDouble(page);
  await installInventoryDouble(page);
  await installSalesDouble(page);
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
  await page.getByRole("button", { name: "Complete sale" }).click();
  await page.getByRole("button", { name: "Confirm sale" }).click();
  await expect(
    page.getByRole("heading", { name: "Receipt preview" }),
  ).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(
    page.getByRole("heading", { name: "Ready for your next sale" }),
  ).toBeVisible();
});
test("catalog create and edit flows stay read-only until the desktop app", async ({
  page,
}) => {
  await page.goto("/catalog");
  await page.getByRole("link", { name: "Add product", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Add product" })).toBeVisible();
  await expect(page.getByText("Desktop app required").first()).toBeVisible();
});
test("purchase receiving and notification state", async ({ page }) => {
  await installCatalogDouble(page);
  await installInventoryDouble(page);
  await installSalesDouble(page);
  await page.goto("/purchases/PO-1048/receive");
  await expect(page.getByText("No separate receiving step")).toBeVisible();
  await page.getByRole("link", { name: "View purchase", exact: true }).click();
  await expect(page.getByRole("heading", { name: "00001" })).toBeVisible();
  await page
    .getByRole("navigation", { name: "Pharmacy navigation" })
    .getByRole("link", { name: "Notifications", exact: true })
    .click();
  await page.getByRole("button", { name: "Mark all read" }).click();
  await expect(page.getByText("0 unread")).toBeVisible();
});
test("stock counts and backup destructive preview", async ({ page }) => {
  await page.goto("/inventory/counts");
  await expect(
    page.getByRole("heading", {
      name: "Stock counts require the desktop app",
    }),
  ).toBeVisible();
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
