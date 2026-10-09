import { expect, test, type Page } from "@playwright/test";
import { installPurchasingDouble } from "./canonical-purchasing-double";

const draft = "00000000-0000-4000-8000-000000000001";
const ordered = "00000000-0000-4000-8000-000000000002";
const cancelled = "00000000-0000-4000-8000-000000000003";
async function calls(page: Page, command: string) {
  return page.evaluate((name) => (window as unknown as { purchasingCalls: { command: string; args: Record<string, unknown> }[] }).purchasingCalls.filter((call) => call.command === name), command);
}
async function openNew(page: Page) {
  await page.goto("/purchases/new");
  await expect(page.getByLabel("Supplier", { exact: true })).toContainText("Nile Medical");
}
async function fillDraft(page: Page) {
  await openNew(page);
  await page.getByLabel("Supplier", { exact: true }).selectOption("11111111-1111-4111-8111-111111111111");
  await page.getByRole("button", { name: /Panadol 500mg/ }).click();
}

test.beforeEach(async ({ page }) => {
  await installPurchasingDouble(page);
  await page.goto("/purchases");
});

test("purchase route uses only canonical purchasing commands", async ({ page }) => {
  await expect(page.getByRole("table", { name: "Purchase orders" })).toContainText("PO-20261003");
  expect(await calls(page, "list_purchase_orders")).not.toHaveLength(0);
  expect(await calls(page, "list_purchases")).toHaveLength(0);
  expect(await calls(page, "complete_purchase")).toHaveLength(0);
});
test("loading state is visible", async ({ page }) => {
  await page.evaluate(() => ((window as unknown as { purchasingDelay: number }).purchasingDelay = 500));
  await page.getByLabel("Search purchases").fill("PO-");
  await expect(page.getByText("Loading purchases…")).toBeVisible();
});
test("empty state distinguishes no purchase history", async ({ page }) => {
  await page.evaluate(() => ((window as unknown as { purchasingEmpty: boolean }).purchasingEmpty = true));
  await page.getByLabel("Search purchases").fill("refresh");
  await page.getByLabel("Search purchases").fill("");
  await expect(page.getByRole("heading", { name: "No purchase orders yet" })).toBeVisible();
});
test("no-results state reflects active filters", async ({ page }) => {
  await page.getByLabel("Search purchases").fill("missing order");
  await expect(page.getByRole("heading", { name: "No purchases match these filters" })).toBeVisible();
});
test("list errors are visible and retryable", async ({ page }) => {
  await page.evaluate(() => ((window as unknown as { purchasingFailCommand: string }).purchasingFailCommand = "list_purchase_orders"));
  await page.getByLabel("Search purchases").fill("fail");
  await expect(page.getByText("Controlled backend rejection")).toBeVisible();
  await page.evaluate(() => ((window as unknown as { purchasingFailCommand: string }).purchasingFailCommand = ""));
  await page.getByRole("button", { name: "Retry" }).click();
  await expect(page.getByRole("heading", { name: "No purchases match these filters" })).toBeVisible();
});
test("search and status are backend filters", async ({ page }) => {
  await page.getByLabel("Search purchases").fill("Nile");
  await page.getByLabel("Purchase status").selectOption("ORDERED");
  await expect.poll(async () => (await calls(page, "list_purchase_orders")).at(-1)?.args.query).toMatchObject({ search: "Nile", status: "ORDERED", limit: 20, offset: 0 });
});
test("purchase list paginates using backend offsets", async ({ page }) => {
  await expect(page.getByText("23 total · Page 1")).toBeVisible();
  await page.getByRole("button", { name: "Next" }).click();
  await expect(page.getByText("23 total · Page 2")).toBeVisible();
  await expect.poll(async () => (await calls(page, "list_purchase_orders")).at(-1)?.args.query).toMatchObject({ offset: 20 });
});
test("supplier selector uses canonical active supplier IPC", async ({ page }) => {
  await openNew(page);
  await page.getByLabel("Search active suppliers").fill("Active");
  await expect(page.getByLabel("Supplier", { exact: true })).toContainText("Active Pharma");
  expect((await calls(page, "list_canonical_suppliers")).at(-1)?.args.query).toMatchObject({ isActive: true });
});
test("Catalog package search passes the query to the backend", async ({ page }) => {
  await openNew(page);
  await page.getByLabel("Search packages").fill("622100000002");
  await expect.poll(async () => (await calls(page, "search_purchase_packages")).at(-1)?.args.query).toMatchObject({ search: "622100000002" });
  await expect(page.getByRole("button", { name: /Vitamin C/ })).toBeVisible();
});
test("add, edit and remove a package line", async ({ page }) => {
  await fillDraft(page);
  await expect(page.getByRole("table")).toContainText("Panadol 500mg");
  await page.getByLabel("Quantity for Panadol 500mg").fill("3");
  await page.getByLabel("Unit cost for Panadol 500mg").fill("14.00");
  await expect(page.getByRole("table")).toContainText("42.00 EGP");
  await page.getByRole("button", { name: "Remove Panadol 500mg" }).click();
  await expect(page.getByRole("heading", { name: "No packages added" })).toBeVisible();
});
test("duplicate package selection is prevented", async ({ page }) => {
  await fillDraft(page);
  await expect(page.getByRole("button", { name: /^Panadol 500mg/ })).toBeDisabled();
  await expect(page.getByRole("table").locator("tbody tr")).toHaveCount(1);
});
test("frontend totals show quantity, discount and tax", async ({ page }) => {
  await fillDraft(page);
  await page.getByLabel("Quantity for Panadol 500mg").fill("2");
  await page.getByLabel("Discount (EGP)").fill("1.00");
  await page.getByLabel("Tax (EGP)").fill("2.00");
  await expect(page.getByText("26.00 EGP", { exact: true })).toBeVisible();
});
test("save draft sends UUID identity, request identity and integer money", async ({ page }) => {
  await fillDraft(page);
  await page.getByLabel("Supplier reference").fill("INV-25");
  await page.getByRole("button", { name: "Save draft" }).click();
  await expect(page.getByRole("heading", { name: "PO-20261003-0024" })).toBeVisible();
  const input = (await calls(page, "create_purchase_order_draft"))[0].args.input as Record<string, unknown>;
  expect(input.id).toMatch(/^[0-9a-f-]{36}$/); expect(input.requestId).toMatch(/^[0-9a-f-]{36}$/);
  expect(input).toMatchObject({ supplierReference: "INV-25", discountMinor: 0, taxMinor: 0 });
  expect((input.lines as unknown[])).toEqual([{ productPackageId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", orderedQuantity: 1, unitCostMinor: 1250 }]);
});
test("duplicate save is blocked while a request is pending", async ({ page }) => {
  await fillDraft(page);
  await page.evaluate(() => ((window as unknown as { purchasingDelay: number }).purchasingDelay = 600));
  const button = page.getByRole("button", { name: "Save draft" });
  await button.dblclick();
  await expect.poll(async () => (await calls(page, "create_purchase_order_draft")).length).toBe(1);
});
test("draft editor loads snapshots and sends expected revision", async ({ page }) => {
  await page.goto(`/purchases/${draft}/edit`);
  await expect(page.getByLabel("Quantity for Panadol 500mg")).toHaveValue("2");
  await page.getByLabel("Notes").fill("Changed notes");
  await page.getByRole("button", { name: "Save draft" }).click();
  expect((await calls(page, "update_purchase_order_draft"))[0].args.input).toMatchObject({ id: draft, expectedRevision: 1, notes: "Changed notes" });
});
test("stale edit conflict stays visible", async ({ page }) => {
  await page.goto(`/purchases/${draft}/edit`);
  await page.evaluate(() => ((window as unknown as { purchasingFailCommand: string }).purchasingFailCommand = "update_purchase_order_draft"));
  await page.getByRole("button", { name: "Save draft" }).click();
  await expect(page.getByText("Purchase order changed. Reload before continuing.")).toBeVisible();
});
test("purchase detail shows canonical snapshots and authoritative totals", async ({ page }) => {
  await page.goto(`/purchases/${draft}`);
  await expect(page.getByRole("heading", { name: "PO-20261003-0001" })).toBeVisible();
  await expect(page.getByText("SUP-000001 · Nile Medical")).toBeVisible();
  await expect(page.getByRole("table", { name: "Purchase order lines" })).toContainText("Panadol 500mg");
  await expect(page.getByText("26.00 EGP", { exact: true })).toBeVisible();
});
test("ordering requires confirmation and uses revision plus durable request ID", async ({ page }) => {
  await page.goto(`/purchases/${draft}`);
  await page.getByRole("button", { name: "Order purchase" }).click();
  await expect(page.getByRole("dialog")).toContainText("does not receive stock");
  await page.getByRole("button", { name: "Confirm order" }).click();
  const input = (await calls(page, "order_purchase_order"))[0].args.input as Record<string, unknown>;
  expect(input).toMatchObject({ id: draft, expectedRevision: 1 }); expect(input.requestId).toMatch(/^[0-9a-f-]{36}$/);
  await expect(page.getByText("Purchase order finalized. No stock was received.")).toBeVisible();
});
test("cancellation requires a reason and remains historical", async ({ page }) => {
  await page.goto(`/purchases/${ordered}`);
  await page.getByRole("button", { name: "Cancel purchase" }).click();
  await expect(page.getByRole("button", { name: "Confirm cancellation" })).toBeDisabled();
  await page.getByLabel("Cancellation reason").fill("Supplier unavailable");
  await page.getByRole("button", { name: "Confirm cancellation" }).click();
  expect((await calls(page, "cancel_purchase_order"))[0].args.input).toMatchObject({ id: ordered, expectedRevision: 1, reason: "Supplier unavailable" });
  await expect(page.getByText("Purchase order cancelled.")).toBeVisible();
});
test("ordered purchase is read-only and canonical receiving is enabled", async ({ page }) => {
  await page.goto(`/purchases/${ordered}`);
  await expect(page.getByRole("button", { name: "Receive stock" })).toBeEnabled();
  await expect(page.getByText(/Receipts post atomically/)).toBeVisible();
  await expect(page.getByRole("button", { name: "Edit draft" })).toHaveCount(0);
});
test("cancelled purchase is terminal in the UI", async ({ page }) => {
  await page.goto(`/purchases/${cancelled}`);
  await expect(page.getByText("No longer required")).toBeVisible();
  await expect(page.getByRole("button", { name: "Edit draft" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Order purchase" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Cancel purchase" })).toHaveCount(0);
});
test("legacy receive URL routes only to canonical receiving", async ({ page }) => {
  await page.goto(`/purchases/${ordered}/receive`);
  await expect(page.getByRole("heading", { name: /Receive PO-20261003-0002/ })).toBeVisible();
  for (const command of ["complete_purchase", "create_inventory_document", "post_inventory_document", "receive_purchase"]) expect(await calls(page, command)).toHaveLength(0);
});
test("IPC action errors remain visible", async ({ page }) => {
  await page.goto(`/purchases/${draft}`);
  await page.evaluate(() => ((window as unknown as { purchasingFailCommand: string }).purchasingFailCommand = "order_purchase_order"));
  await page.getByRole("button", { name: "Order purchase" }).click();
  await page.getByRole("button", { name: "Confirm order" }).click();
  await expect(page.getByText("Controlled backend rejection")).toBeVisible();
});
test("order submit is disabled while pending", async ({ page }) => {
  await page.goto(`/purchases/${draft}`);
  await page.evaluate(() => ((window as unknown as { purchasingDelay: number }).purchasingDelay = 700));
  await page.getByRole("button", { name: "Order purchase" }).click();
  const confirm = page.getByRole("button", { name: "Confirm order" });
  await confirm.dblclick();
  await expect.poll(async () => (await calls(page, "order_purchase_order")).length).toBe(1);
});

for (const [width, height] of [[1366, 768], [1440, 900], [1920, 1080]]) {
  test(`purchase editor fits supported viewport ${width}x${height}`, async ({ page }) => {
    await page.setViewportSize({ width, height }); await fillDraft(page);
    await expect(page.getByRole("button", { name: "Save draft" })).toBeInViewport();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  });
}
