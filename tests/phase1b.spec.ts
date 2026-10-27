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
  await expect(
    page.getByRole("heading", { name: "Add product" }),
  ).toBeVisible();
  await expect(page.getByText("Desktop app required")).toBeVisible();
  await page.goto("/catalog/p1");
  await expect(
    page.getByRole("heading", { name: "Product details" }),
  ).toBeVisible();
  await page.goto("/catalog/p1/edit");
  await expect(
    page.getByRole("heading", { name: "Edit product" }),
  ).toBeVisible();
  await expect(page.getByText("Desktop app required").first()).toBeVisible();
});
test("browser POS is honest and never simulates a persisted sale", async ({
  page,
}) => {
  await page.goto("/sales");
  await expect(
    page.getByRole("heading", {
      name: "Connect to the desktop app to use the POS.",
    }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "Complete sale" })).toHaveCount(
    0,
  );
});
test("catalog create and edit flows stay read-only until the desktop app", async ({
  page,
}) => {
  await page.goto("/catalog");
  await page.getByRole("link", { name: "Add product", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Add product" }),
  ).toBeVisible();
  await expect(page.getByText("Desktop app required").first()).toBeVisible();
});
test("browser purchase receiving and notifications require the desktop app", async ({
  page,
}) => {
  await page.goto("/purchases/PO-1048/receive");
  await expect(
    page.getByRole("heading", {
      name: "Connect to the desktop app to receive a purchase.",
    }),
  ).toBeVisible();
  await page.goto("/notifications");
  await expect(
    page.getByRole("heading", {
      name: "Notifications requires the desktop app",
    }),
  ).toBeVisible();
});
test("browser stock counts and backup never simulate local writes", async ({
  page,
}) => {
  await page.goto("/inventory/counts");
  await expect(
    page.getByRole("heading", {
      name: "Stock counts require the desktop app",
    }),
  ).toBeVisible();
  await page.goto("/backup");
  await expect(
    page.getByRole("heading", {
      name: "Backup and restore requires the desktop app",
    }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Restore", exact: true }),
  ).toHaveCount(0);
});
