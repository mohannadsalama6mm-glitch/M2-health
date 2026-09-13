import { test, expect } from "@playwright/test";
import { installCatalogDouble } from "./catalog-double";
import { installInventoryDouble } from "./inventory-double";

async function seed(page: import("@playwright/test").Page) {
  await installCatalogDouble(page);
  await installInventoryDouble(page);
}

test("overview lists packages with branch ledgers and summary", async ({
  page,
}) => {
  await seed(page);
  await page.goto("/inventory");
  await expect(
    page.getByRole("heading", { name: "Inventory" }),
  ).toBeVisible();
  const grid = page.getByRole("table", { name: "Inventory" });
  const panadol = grid
    .getByRole("row")
    .filter({ hasText: "Panadol 500 mg" });
  await expect(panadol).toContainText("46");
  await expect(panadol).toContainText("LOT-A");
  await expect(panadol).toContainText("Expiring");
  const amox = grid.getByRole("row").filter({ hasText: "Amoxicillin 500 mg" });
  await expect(amox).toContainText("5");
  await expect(amox).toContainText("Low stock");
  await expect(page.getByText("Low stock", { exact: true }).last()).toBeVisible();
  await expect(
    page.locator(".summary-strip").getByText("In stock", { exact: true }),
  ).toBeVisible();
  await expect(
    page.locator(".summary-strip").getByText("Low stock", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Desktop app required" }),
  ).toHaveCount(0);
});

test("adjust stock posts a movement and updates the overview", async ({
  page,
}) => {
  await seed(page);
  await page.goto("/inventory");
  await page
    .getByRole("button", { name: "Adjust stock", exact: true })
    .click();
  await page
    .getByLabel("Product / package")
    .selectOption({ value: "pkg-1" });
  await page.getByLabel("New stock quantity").fill("50");
  await page.getByLabel("Reason / reference").fill("Count review");
  await page.getByRole("button", { name: "Apply", exact: true }).click();
  await expect(
    page.getByText("Stock adjusted. The movement was posted."),
  ).toBeVisible();
  await expect(
    page
      .getByRole("table", { name: "Inventory" })
      .getByRole("row")
      .filter({ hasText: "Panadol" }),
  ).toContainText("50");
  await page
    .getByRole("link", { name: "Movements", exact: true })
    .click();
  const ledger = page.getByRole("table", { name: "Stock movements" });
  const adjustment = ledger
    .getByRole("row")
    .filter({ hasText: "Panadol 500 mg" })
    .filter({ hasText: "Adjustment" });
  await expect(adjustment.first()).toBeVisible();
});

test("transfer moves stock between branches", async ({ page }) => {
  await seed(page);
  await page.goto("/inventory");
  await page.getByRole("button", { name: "Transfer", exact: true }).click();
  await page
    .getByLabel("Product / package")
    .selectOption({ value: "pkg-1" });
  await page.getByLabel("Transfer quantity").fill("3");
  await page
    .getByLabel("Destination branch")
    .selectOption({ label: "Maadi Branch" });
  await page.getByRole("button", { name: "Apply", exact: true }).click();
  await expect(
    page.getByText("Stock transferred between branches."),
  ).toBeVisible();
  await expect(
    page
      .getByRole("table", { name: "Inventory" })
      .getByRole("row")
      .filter({ hasText: "Panadol" }),
  ).toContainText("43");
});

test("reorder level can be edited per package", async ({ page }) => {
  await seed(page);
  await page.goto("/inventory");
  await page
    .getByRole("button", {
      name: "Set reorder level for Panadol 500 mg",
    })
    .click();
  await expect(page.getByRole("heading", { name: "Reorder level" })).toBeVisible();
  await page
    .getByRole("spinbutton", { name: "Reorder level" })
    .fill("20");
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await expect(page.getByText("Reorder level updated.")).toBeVisible();
  await expect(
    page
      .getByRole("table", { name: "Inventory" })
      .getByRole("row")
      .filter({ hasText: "Panadol" }),
  ).toContainText("20");
});

test("expiry write-off removes an expired batch from the ledger", async ({
  page,
}) => {
  await seed(page);
  await page.goto("/inventory/expiry");
  await expect(
    page.getByRole("heading", { name: "Expiry management" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Expired", exact: true }).click();
  const expired = page
    .getByRole("table", { name: "Expiry batches" })
    .getByRole("row")
    .filter({ hasText: "LOT-D" });
  await expect(expired).toContainText("Expired");
  await page
    .getByRole("button", { name: "Write-off", exact: true })
    .first()
    .click();
  await expect(
    page.getByRole("heading", { name: "Write off batch" }),
  ).toBeVisible();
  await page
    .getByLabel("Reason", { exact: true })
    .fill("Expired stock review");
  await page
    .getByRole("button", { name: "Write off", exact: true })
    .click();
  await expect(
    page.getByText("Stock written off and posted to the ledger."),
  ).toBeVisible();
  await expect(
    page.getByRole("table", { name: "Expiry batches" }).getByRole("row"),
  ).toHaveCount(1);
});

test("count workflow creates saves completes and read-only review", async ({
  page,
}) => {
  await seed(page);
  await page.goto("/inventory/counts");
  await page
    .getByRole("button", { name: "New count session", exact: true })
    .click();
  await page
    .getByLabel("Scope / note")
    .fill("Year-end shelf audit");
  await page
    .getByRole("button", { name: "Create session", exact: true })
    .click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await expect(
    dialog.getByText("Editing session", { exact: true }),
  ).toBeVisible();
  const countTable = dialog.getByRole("table", { name: "Count quantities" });
  const rows = countTable.getByRole("row");
  await expect(rows).toHaveCount(6);
  const firstInput = rows.nth(1).locator("input");
  await firstInput.fill("25");
  await dialog.getByRole("button", { name: "Save counts" }).click();
  await expect(page.getByText("Counted quantities saved.")).toBeVisible();
  await dialog.getByRole("button", { name: "Complete count" }).click();
  await page.getByRole("button", { name: "Confirm", exact: true }).click();
  await expect(dialog.getByText("Count completed")).toBeVisible();
  await expect(dialog.getByText("read-only")).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(
    page
      .getByRole("table", { name: "Count sessions" })
      .getByRole("row")
      .nth(1),
  ).toContainText("Completed");
});

test("browser mode explains the desktop requirement on inventory screens", async ({
  page,
}) => {
  await page.goto("/inventory");
  await expect(
    page.getByRole("heading", {
      name: "Inventory requires the desktop app",
    }),
  ).toBeVisible();
  await page.goto("/inventory/movements");
  await expect(
    page.getByRole("heading", {
      name: "Movements require the desktop app",
    }),
  ).toBeVisible();
  await page.goto("/inventory/counts");
  await expect(
    page.getByRole("heading", {
      name: "Stock counts require the desktop app",
    }),
  ).toBeVisible();
  await page.goto("/inventory/expiry");
  await expect(
    page.getByRole("heading", {
      name: "Expiry management requires the desktop app",
    }),
  ).toBeVisible();
});