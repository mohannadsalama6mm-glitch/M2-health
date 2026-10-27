import { expect, test, type Page } from "@playwright/test";
import { installInventoryDouble } from "./inventory-double";
import { installWorkspaceDouble } from "./inventory-workspace-double";

async function prepare(page: Page) {
  await installWorkspaceDouble(page);
  await installInventoryDouble(page);
  await page.addInitScript(() => {
    localStorage.setItem(
      "m2-health.onboarding.desktop.v1",
      JSON.stringify({ dismissed: true }),
    );
    localStorage.setItem(
      "m2-health.tutorial.desktop.v1",
      JSON.stringify({ dismissed: true, completed: false }),
    );
  });
  await page.goto("/inventory", { waitUntil: "domcontentloaded" });
  await expect(page.getByRole("heading", { name: "Inventory" })).toBeVisible();
}

test.beforeEach(async ({ page }) => {
  await prepare(page);
});

test("existing product adds opening stock to the active branch through the ledger", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Add stock", exact: true }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toContainText("Main Pharmacy");
  await dialog.getByLabel("Product", { exact: true }).selectOption("p-1");
  await expect(dialog.getByLabel("Package")).toHaveValue("pkg-1");
  await dialog.getByLabel("Quantity").fill("7");
  await dialog.getByLabel("Reason").fill("Initial branch setup");
  await expect(dialog.getByLabel("Batch / lot number")).not.toHaveAttribute(
    "required",
  );
  await expect(dialog.getByLabel("Expiry date")).not.toHaveAttribute(
    "required",
  );
  await dialog.getByRole("button", { name: "Add stock", exact: true }).click();
  await expect(
    page.getByText(
      "Stock added to Main Pharmacy. The opening movement was posted to the ledger.",
    ),
  ).toBeVisible();
  const result = await page.evaluate(() => {
    const store = (
      window as unknown as {
        __inventoryStore: {
          movements: {
            branchId: string;
            productPackageId: string;
            movementType: string;
            quantityDelta: number;
            reason: string;
          }[];
        };
      }
    ).__inventoryStore;
    return store.movements.at(-1);
  });
  expect(result).toMatchObject({
    branchId: "424b25b8-3c75-4d7a-87cd-988397081194",
    productPackageId: "pkg-1",
    movementType: "opening",
    quantityDelta: 7,
    reason: "Initial branch setup",
  });
});

test("new product is created inline and immediately receives stock", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Add stock", exact: true }).click();
  const dialog = page.getByRole("dialog");
  await dialog.getByRole("button", { name: "New product" }).click();
  await dialog.getByLabel("Product name").fill("New Test Medicine");
  await dialog.getByLabel("Package label").fill("Box of 20 tablets");
  await dialog.getByLabel("Quantity").fill("12");
  await dialog.getByLabel("Reason").fill("Initial shelf count");
  await dialog
    .getByRole("button", { name: "Create product and add stock" })
    .click();
  await expect(
    page.getByText(
      "Stock added to Main Pharmacy. The opening movement was posted to the ledger.",
    ),
  ).toBeVisible();
  const result = await page.evaluate(() => {
    const catalog = (
      window as unknown as {
        __catalogStore: { products: { product: { commercialNameEn: string } }[] };
      }
    ).__catalogStore;
    const inventory = (
      window as unknown as {
        __inventoryStore: {
          movements: { movementType: string; quantityDelta: number }[];
        };
      }
    ).__inventoryStore;
    return {
      created: catalog.products.some(
        (entry) => entry.product.commercialNameEn === "New Test Medicine",
      ),
      movement: inventory.movements.at(-1),
    };
  });
  expect(result.created).toBe(true);
  expect(result.movement).toMatchObject({
    movementType: "opening",
    quantityDelta: 12,
  });
});

test("empty inventory explains product versus stock and offers both next actions", async ({
  page,
}) => {
  await page.evaluate(() => {
    (window as unknown as { inspectionTotal: number }).inspectionTotal = 0;
    history.pushState({}, "", "/catalog");
    window.dispatchEvent(new PopStateEvent("popstate"));
  });
  await page.getByRole("link", { name: "Inventory", exact: true }).click();
  const empty = page
    .getByRole("heading", { name: "No stock is available in this branch" })
    .locator("..");
  await expect(empty).toContainText("Inventory shows quantities");
  await expect(empty.getByRole("button", { name: "Add stock" })).toBeVisible();
  await expect(empty.getByRole("link", { name: "Create product" })).toBeVisible();
});

test("Arabic RTL stock entry remains localized and English LTR can be restored", async ({
  page,
}) => {
  await page.getByLabel("Language").selectOption("ar");
  await expect(page.locator("html")).toHaveAttribute("lang", "ar");
  await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
  await page.getByRole("button", { name: "إضافة مخزون", exact: true }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog.getByRole("heading", { name: "إضافة مخزون" })).toBeVisible();
  await expect(dialog.getByLabel("كيف وصل هذا المخزون؟")).toBeVisible();
  await expect(dialog.getByLabel("الكمية")).toBeVisible();
  await expect(dialog.getByText("بيانات اختيارية للتشغيلة والصلاحية والتكلفة")).toBeVisible();
  await page.keyboard.press("Escape");
  await page.getByLabel("اللغة").selectOption("en");
  await expect(page.locator("html")).toHaveAttribute("lang", "en");
  await expect(page.locator("html")).toHaveAttribute("dir", "ltr");
  await expect(page.getByRole("button", { name: "Add stock", exact: true })).toBeVisible();
});
