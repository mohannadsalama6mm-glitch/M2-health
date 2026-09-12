import { test, expect } from "@playwright/test";
import { installCatalogDouble } from "./catalog-double";
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
    for (const name of [
      "Panadol 500 mg",
      "Amoxicillin 500 mg",
      "Vitamin C 1000 mg",
      "Cetirizine 10 mg",
    ])
      await page
        .getByRole("button", { name: `Add ${name} to cart`, exact: true })
        .click();
    await page
      .locator(".cart-lines")
      .evaluate((e) => (e.scrollTop = e.scrollHeight));
    const button = await page
      .getByRole("button", { name: "Complete sale · demo" })
      .boundingBox();
    const main = await page.locator("main").boundingBox();
    expect(button!.y + button!.height).toBeLessThanOrEqual(
      main!.y + main!.height,
    );
    expect(button!.y).toBeGreaterThan(main!.y);
    await expect(page.getByText("Grand total", { exact: true })).toBeVisible();
    expect(
      await page
        .locator(".cart-lines")
        .evaluate((e) => e.scrollHeight > e.clientHeight),
    ).toBe(true);
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= window.innerWidth,
      ),
    ).toBe(true);
    await page.screenshot({ path: `test-results/pos-cart-${width}.png` });
  });
test("POS quantity discount customer card and void confirmation", async ({
  page,
}) => {
  await page.goto("/sales");
  await page.getByLabel("Barcode input").fill("6221001000011");
  await page.getByLabel("Barcode input").press("Enter");
  await page.getByRole("button", { name: "Increase quantity" }).click();
  await page.getByLabel("Sale discount (EGP)").fill("4");
  await page.getByLabel("Sale customer").selectOption("Sara Mohamed");
  await page.getByLabel("Payment method", { exact: true }).selectOption("Card");
  await expect(page.getByLabel("Cash received (EGP)")).toBeDisabled();
  await page.getByRole("button", { name: "Complete sale · demo" }).click();
  await expect(
    page.getByRole("dialog").getByText("Sara Mohamed", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("dialog").getByText("60.00", { exact: false }),
  ).toBeVisible();
  await page.keyboard.press("Escape");
  await page.getByRole("button", { name: "Cancel / void" }).click();
  await page.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(
    page.getByRole("spinbutton", { name: "Quantity", exact: true }),
  ).toHaveValue("2");
  await page.getByRole("button", { name: "Cancel / void" }).click();
  await page.getByRole("button", { name: "Confirm", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Ready for your next sale" }),
  ).toBeVisible();
});
test("inventory adjustment transfer and expiry review", async ({ page }) => {
  await page.goto("/inventory");
  await page.getByRole("button", { name: "Adjust stock", exact: true }).click();
  await page.getByLabel("New stock quantity").fill("125");
  await page.getByLabel("Reason / reference").fill("Count review");
  await page.getByRole("button", { name: "Apply demo change" }).click();
  await expect(
    page
      .getByRole("table", { name: "Inventory" })
      .getByRole("row")
      .filter({ hasText: "Panadol" }),
  ).toContainText("125");
  await page.getByRole("button", { name: "Transfer", exact: true }).click();
  await page.getByLabel("Transfer quantity").fill("2");
  await page.getByLabel("Reason / reference").fill("Branch review");
  await page.getByRole("button", { name: "Apply demo change" }).click();
  await expect(
    page
      .getByRole("table", { name: "Inventory" })
      .getByRole("row")
      .filter({ hasText: "Panadol" }),
  ).toContainText("125");
  await page
    .getByRole("link", { name: "Expiry management", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Review", exact: true })
    .first()
    .click();
  await page
    .getByRole("button", { name: "Return to supplier", exact: true })
    .click();
  await page.getByLabel("Reason", { exact: true }).fill("Near expiry");
  await page.getByRole("button", { name: "Apply demo change" }).click();
  await expect(page.getByRole("dialog")).not.toBeVisible();
});
test("created purchase keeps its selected lines through receiving", async ({
  page,
}) => {
  await page.goto("/purchases/new");
  await page
    .getByLabel("Purchase supplier")
    .selectOption("Nile Medical Supplies");
  await page.getByLabel("Purchase product 1").selectOption("p3");
  await page.getByLabel("Order quantity 1").fill("4");
  await page.getByRole("button", { name: "Save demo draft" }).click();
  await expect(
    page.getByRole("table", { name: "Purchase order lines" }),
  ).toContainText("Vitamin C 1000 mg");
  await expect(
    page.getByRole("table", { name: "Purchase order lines" }).getByRole("row"),
  ).toHaveCount(2);
  await page.getByRole("button", { name: "Mark ordered" }).click();
  await page
    .getByRole("link", { name: "Receive purchase", exact: true })
    .click();
  await expect(page.getByLabel("Received p3")).toHaveValue("4");
  await page.getByLabel("Batch p3", { exact: true }).fill("QA-VC");
  await page.getByLabel("Expiry p3", { exact: true }).fill("2028-01-01");
  await page.getByRole("button", { name: "Review receiving" }).click();
  await page.getByRole("button", { name: "Confirm", exact: true }).click();
  await expect(
    page.getByRole("table", { name: "Purchase order lines" }),
  ).toContainText("Vitamin C 1000 mg");
  await expect(
    page.getByText("Received", { exact: true }).first(),
  ).toBeVisible();
});
test("contacts management reports finance and system controls", async ({
  page,
}) => {
  test.setTimeout(90000);
  await page.goto("/suppliers");
  await page
    .getByRole("link", { name: "United Pharma Distribution Karim Adel" })
    .click();
  await page.getByRole("tab", { name: "Products supplied" }).click();
  await expect(
    page.getByRole("table", { name: "Supplier products" }),
  ).toBeVisible();
  await page.goto("/customers/c1");
  await page.getByRole("button", { name: "Edit contact" }).click();
  await page.getByLabel("Name", { exact: true }).fill("Ahmed UI review");
  await page.getByRole("button", { name: "Apply demo change" }).click();
  await expect(
    page.getByRole("heading", { name: "Ahmed UI review" }),
  ).toBeVisible();
  await page.goto("/roles");
  await page.getByRole("button", { name: "Cashier", exact: true }).click();
  const permission = page
    .getByRole("switch", { name: "View", exact: true })
    .first();
  await permission.click();
  await expect(permission).toHaveAttribute("aria-checked", "false");
  await page.goto("/branches/b1");
  await page.getByRole("tab", { name: "Devices", exact: true }).click();
  await expect(
    page.getByRole("table", { name: "Branch devices" }),
  ).toBeVisible();
  await page.goto("/audit-log");
  await page
    .getByRole("button", { name: "Inspect", exact: true })
    .first()
    .click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await page.keyboard.press("Escape");
  await page.goto("/sync");
  await page.getByRole("button", { name: "View queue", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Pending synchronization queue" }),
  ).toBeVisible();
  await page.keyboard.press("Escape");
  await page.goto("/settings");
  await page.getByRole("button", { name: "Receipt", exact: true }).click();
  await page.getByLabel("Footer", { exact: true }).fill("Thank you");
  await page.getByRole("button", { name: "Save settings preview" }).click();
  await expect(page.getByLabel("Footer", { exact: true })).toHaveValue(
    "Thank you",
  );
  await page.goto("/reports");
  await page.getByRole("button", { name: "Inventory", exact: true }).click();
  await expect(
    page.getByRole("table", { name: "Inventory report" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Export", exact: true }).click();
  await expect(
    page
      .getByRole("dialog")
      .getByRole("button", { name: "Export (not connected)" }),
  ).toBeDisabled();
  await page.keyboard.press("Escape");
  await page.goto("/finance");
  await page.getByRole("button", { name: "Expenses", exact: true }).click();
  await page
    .getByRole("button", { name: "Record expense", exact: true })
    .click();
  await page.getByLabel("Amount (EGP)").fill("50");
  await page.getByLabel("Reference", { exact: true }).fill("UI");
  await page.getByLabel("Description", { exact: true }).fill("Preview");
  await page.getByRole("button", { name: "Review preview" }).click();
  await expect(page.getByRole("dialog")).not.toBeVisible();
});
for (const [width, height] of [
  [1366, 768],
  [1440, 900],
  [1920, 1080],
])
  test(`dialog drawer brand and collapsed shell ${width}`, async ({ page }) => {
    await page.setViewportSize({ width, height });
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
    await page
      .getByRole("button", { name: "Review", exact: true })
      .first()
      .click();
    await expect(page.getByRole("dialog")).toBeVisible();
    expect(
      await page
        .getByRole("dialog")
        .evaluate((el) => el.scrollWidth <= el.clientWidth),
    ).toBe(true);
    await page.screenshot({ path: `test-results/expiry-drawer-${width}.png` });
    await page.keyboard.press("Escape");
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
