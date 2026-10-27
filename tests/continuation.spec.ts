import { installAlertsDouble } from "./inventory-alerts-double";
import { test, expect } from "@playwright/test";
import { installCatalogDouble } from "./catalog-double";
import { installInventoryDouble } from "./inventory-double";
test("catalog retains multiple packages ingredients barcodes and pricing through edit", async ({
  page,
}) => {
  await installCatalogDouble(page);
  await page.goto("/catalog");
  await page
    .getByRole("searchbox", { name: "Search Products" })
    .fill("Panadol");
  await page
    .getByRole("link", { name: "Panadol 500 mg Paracetamol · Haleon" })
    .click();
  await page.getByRole("link", { name: "Edit product" }).click();
  await page.getByRole("button", { name: "Add package", exact: true }).click();
  await page
    .getByLabel("Package 2 label", { exact: true })
    .fill("Strip · 12 tablets");
  await page
    .getByLabel("Package 2 barcode 1", { exact: true })
    .fill("6229999999992");
  await page
    .getByRole("button", { name: "Add ingredient", exact: true })
    .click();
  await page
    .getByLabel("Ingredient 2", { exact: true })
    .selectOption("Demo second ingredient");
  await page
    .getByLabel("Package 2 selling price (EGP)", { exact: true })
    .fill("39.50");
  await page.getByRole("button", { name: "Save changes", exact: true }).click();
  await page.getByRole("tab", { name: "Packages & barcodes" }).click();
  await expect(page.getByText("6229999999992", { exact: true })).toBeVisible();
  await expect(
    page.getByText("Strip · 12 tablets", { exact: false }).first(),
  ).toBeVisible();
  await page.getByRole("link", { name: "Edit product" }).click();
  await expect(page.getByLabel("Package 2 label", { exact: true })).toHaveValue(
    "Strip · 12 tablets",
  );
  await expect(
    page.getByLabel("Package 2 barcode 1", { exact: true }),
  ).toHaveValue("6229999999992");
  await expect(
    page.getByLabel("Package 2 selling price (EGP)", { exact: true }),
  ).toHaveValue("39.5");
  await page
    .getByLabel("Brand / product name (English)", { exact: true })
    .fill("Panadol review edit");
  await page.getByRole("button", { name: "Save changes", exact: true }).click();
  await page
    .getByRole("navigation", { name: "Pharmacy navigation" })
    .getByRole("link", { name: "Products", exact: true })
    .click();
  await page
    .getByRole("searchbox", { name: "Search Products" })
    .fill("Panadol review edit");
  await expect(
    page.getByRole("table", { name: "Products" }).getByRole("row"),
  ).toHaveCount(2);
});
for (const [width, height] of [
  [1366, 768],
  [1440, 900],
  [1920, 1080],
])
  test(`POS checkout stays in viewport with a long cart ${width}`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height });
    await page.goto("/sales");
    const state = page.getByRole("heading", {
      name: "Connect to the desktop app to use the POS.",
    });
    await expect(state).toBeVisible();
    const box = await state.boundingBox();
    const main = await page.locator("main").boundingBox();
    expect(box!.y + box!.height).toBeLessThanOrEqual(main!.y + main!.height);
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= window.innerWidth,
      ),
    ).toBe(true);
    await page.screenshot({ path: `test-results/pos-cart-${width}.png` });
  });
test("browser POS exposes no mutable payment or void workflow", async ({
  page,
}) => {
  await page.goto("/sales");
  await expect(page.getByLabel("Barcode input")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Complete sale" })).toHaveCount(
    0,
  );
  await expect(page.getByRole("button", { name: "Cancel / void" })).toHaveCount(
    0,
  );
});
test("browser purchase editor cannot post stock", async ({ page }) => {
  await page.goto("/purchases/new");
  await expect(
    page.getByRole("heading", {
      name: "Connect to the desktop app to complete a purchase.",
    }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Complete purchase" }),
  ).toHaveCount(0);
});
test("browser partner and system routes never imply local persistence", async ({
  page,
}) => {
  await page.goto("/suppliers");
  await expect(
    page.getByRole("heading", {
      name: "Connect to the desktop app to manage suppliers.",
    }),
  ).toBeVisible();
  for (const [path, heading] of [
    ["/audit-log", "Security audit log requires the desktop app"],
    ["/settings", "Settings requires the desktop app"],
    ["/backup", "Backup and restore requires the desktop app"],
  ]) {
    await page.goto(path);
    await expect(page.getByRole("heading", { name: heading })).toBeVisible();
  }
});
for (const [width, height] of [
  [1366, 768],
  [1440, 900],
  [1920, 1080],
])
  test(`dialog drawer brand and collapsed shell ${width}`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    await installCatalogDouble(page);
    await installInventoryDouble(page);
    await installAlertsDouble(page, { catalog: false, preserveRecovery: true });
    await page.goto("/inventory/counts");
    await expect(page.locator(".sidebar .brand-superscript")).toBeVisible();
    const logo = await page.locator(".sidebar .logo svg").boundingBox();
    const two = await page.locator(".sidebar .brand-superscript").boundingBox();
    expect(two!.height).toBeGreaterThan(logo!.height * 0.2);
    await page.getByRole("button", { name: "SC-003", exact: true }).click();
    const dialog = page.getByRole("dialog");
    const box = await dialog.boundingBox();
    expect(box!.x).toBeGreaterThanOrEqual(0);
    expect(box!.y).toBeGreaterThanOrEqual(0);
    expect(box!.y + box!.height).toBeLessThanOrEqual(height);
    await page.screenshot({ path: `test-results/count-dialog-${width}.png` });
    await page.keyboard.press("Escape");
    await expect(
      page.getByRole("button", { name: "SC-003", exact: true }),
    ).toBeFocused();
    await page
      .getByRole("link", { name: "Expiry management", exact: true })
      .click();
    await page.getByLabel("Expiry window").selectOption("expired");
    await expect(
      page.getByRole("table", { name: "Expiry batches" }),
    ).toContainText("DEV EXPIRY");
    expect(
      await page
        .locator("main")
        .evaluate((el) => el.scrollWidth <= el.clientWidth),
    ).toBe(true);
    await expect(
      page.getByRole("button", { name: "Write-off", exact: true }),
    ).toHaveCount(0);
    await page.screenshot({ path: `test-results/expiry-monitor-${width}.png` });
    await page
      .getByRole("button", { name: "Collapse sidebar", exact: true })
      .click();
    await expect(page.locator(".sidebar .logo-icon")).toBeVisible();
    await expect(page.locator(".sidebar .nav-item.active")).toContainText(
      "Expiry",
    );
    const nav = page.getByRole("navigation", { name: "Pharmacy navigation" });
    expect(await nav.evaluate((el) => getComputedStyle(el).overflowY)).toBe(
      "auto",
    );
    const scroll = await nav.evaluate((el) => {
      el.scrollTop = el.scrollHeight;
      return {
        top: el.scrollTop,
        maximum: Math.max(0, el.scrollHeight - el.clientHeight),
      };
    });
    expect(scroll.top).toBe(scroll.maximum);
    const lastItem = await nav
      .getByRole("link", { name: "Settings", exact: true })
      .boundingBox();
    const navBox = await nav.boundingBox();
    expect(lastItem!.y + lastItem!.height).toBeLessThanOrEqual(
      navBox!.y + navBox!.height + 1,
    );
    await expect(nav).not.toContainText("Design system");
    await page
      .getByRole("button", { name: "Expand sidebar", exact: true })
      .click();
  });
