import { test, expect, type Page } from "@playwright/test";
import { installDocumentsDouble } from "./inventory-documents-double";

test.beforeEach(async ({ page }) => { await installDocumentsDouble(page); await page.goto("/inventory/documents"); });
async function line(page: Page, kind = "Receive Stock", action?: string) {
  await page.getByRole("button", { name: kind, exact: true }).click();
  if (kind === "Transfer Stock") await page.getByLabel("Destination branch").selectOption("dev-branch");
  await page.getByLabel("Search packages").fill("Panadol");
  await expect(page.getByLabel("Product", { exact: true }).locator("option")).toHaveCount(2);
  await page.getByLabel("Product", { exact: true }).selectOption("p-1");
  await page.getByLabel("Package", { exact: true }).selectOption("pkg-1");
  if (kind !== "Receive Stock") await page.getByLabel("Batch", { exact: true }).selectOption("batch-1");
  if (action) await page.getByLabel(kind === "Adjust Stock" ? "Adjustment direction" : "Write-off type").selectOption(action);
  await page.getByLabel("Quantity", { exact: true }).fill("2");
  await page.getByLabel("Line reason").fill("DEV test reason");
  await page.getByRole("button", { name: "Add line", exact: true }).click();
  await page.getByRole("button", { name: "Save draft", exact: true }).click();
  await expect(page.getByRole("button", { name: "Mark ready" })).toBeEnabled();
}
async function post(page: Page) {
  await page.getByRole("button", { name: "Mark ready" }).click();
  await page.getByRole("button", { name: "Post document", exact: true }).click();
  await expect(page.getByRole("dialog")).toContainText("permanent stock movements");
  await page.getByRole("button", { name: "Confirm", exact: true }).click();
  await expect(page.getByText("Posted document is read-only")).toBeVisible();
}
test("receipt draft persists lines and nullable metadata without posting", async ({ page }) => {
  await line(page);
  await expect(page.getByRole("table", { name: "Document lines" })).toContainText("0 → 2");
  const calls = await page.evaluate(() => (window as unknown as { documentCalls: { command: string }[] }).documentCalls);
  expect(calls.some(c => c.command === "post_inventory_document")).toBe(false);
});
test("ready and confirmed receipt post shows canonical IDs and read-only fields", async ({ page }) => {
  await line(page); await post(page);
  await expect(page.getByRole("table", { name: "Document movement links" })).toContainText("PURCHASE_RECEIPT");
  await expect(page.getByLabel("Notes", { exact: true })).toBeDisabled();
  await expect(page.getByRole("button", { name: "Save draft" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Cancel document" })).toHaveCount(0);
});
for (const action of ["IN", "OUT"]) test(`adjustment ${action} previews backend balance and posts`, async ({ page }) => {
  await line(page, "Adjust Stock", action);
  await expect(page.getByRole("table", { name: "Document lines" })).toContainText(action === "IN" ? "8 → 10" : "8 → 6"); await post(page);
});
test("damage write-off posts selected batch", async ({ page }) => { await line(page, "Damage / Expiry Write-Off", "DAMAGE"); await post(page); });
test("expiry backend eligibility rejection keeps draft", async ({ page }) => {
  await line(page, "Damage / Expiry Write-Off", "EXPIRY"); await page.getByRole("button", { name: "Mark ready" }).click();
  await expect(page.getByText("EXPIRY requires a known expired date.")).toBeVisible(); await expect(page.getByRole("button", { name: "Post document", exact: true })).toHaveCount(0);
});
test("transfer posts source and destination movement references", async ({ page }) => {
  await line(page, "Transfer Stock"); await post(page);
  const links = page.getByRole("table", { name: "Document movement links" }); await expect(links).toContainText("TRANSFER_OUT"); await expect(links).toContainText("TRANSFER_IN"); await expect(links).toContainText("DEV destination");
});
test("cancel confirmation makes draft read-only", async ({ page }) => {
  await line(page); await page.getByRole("button", { name: "Cancel document" }).click(); await page.getByRole("button", { name: "Confirm", exact: true }).click();
  await expect(page.getByText("Cancelled document", { exact: true })).toBeVisible(); await expect(page.getByRole("button", { name: "Mark ready" })).toHaveCount(0);
});
test("stale revision error preserves edits", async ({ page }) => {
  await line(page); await page.getByLabel("Notes", { exact: true }).fill("unsaved correction");
  await page.evaluate(() => { (window as unknown as { documentFailure: string }).documentFailure = "Document revision is stale. Reload before retrying."; });
  await page.getByRole("button", { name: "Save draft" }).click(); await expect(page.getByText("Document revision is stale. Reload before retrying.")).toBeVisible(); await expect(page.getByRole("textbox", { name: "Notes", exact: true })).toHaveValue("unsaved correction");
});
test("duplicate submit is locked while post is pending", async ({ page }) => {
  await line(page); await page.getByRole("button", { name: "Mark ready" }).click();
  await page.evaluate(() => { (window as unknown as { postDelay: number }).postDelay = 1000; });
  await page.getByRole("button", { name: "Post document", exact: true }).click(); await page.getByRole("button", { name: "Confirm", exact: true }).click();
  await expect(page.getByRole("button", { name: "Post document", exact: true })).toBeDisabled(); await expect(page.getByText("Posted document is read-only")).toBeVisible();
  expect(await page.evaluate(() => (window as unknown as { documentCalls: { command: string }[] }).documentCalls.filter(c => c.command === "post_inventory_document").length)).toBe(1);
});
test("backend post failure remains visible and does not claim success", async ({ page }) => {
  await line(page); await page.getByRole("button", { name: "Mark ready" }).click();
  await page.evaluate(() => { (window as unknown as { documentFailure: string }).documentFailure = "Insufficient stock after another posting."; });
  await page.getByRole("button", { name: "Post document", exact: true }).click(); await page.getByRole("button", { name: "Confirm", exact: true }).click();
  await expect(page.getByText("Insufficient stock after another posting.")).toBeVisible(); await expect(page.getByText("Posted document is read-only")).toHaveCount(0);
});
test("document pagination and filters are sent to backend", async ({ page }) => {
  await page.evaluate(() => { (window as unknown as { documentTotal: number }).documentTotal = 41; });
  await page.getByLabel("Document type").selectOption("RECEIPT");
  await page.getByRole("button", { name: "Next documents" }).click();
  await expect(page.getByText("Page 2", { exact: true })).toBeVisible();
  const calls = await page.evaluate(() => (window as unknown as { documentCalls: { command: string; args: { query: { offset: number; limit: number; kind: string } } }[] }).documentCalls.filter(c => c.command === "list_inventory_documents"));
  expect(calls.at(-1)?.args.query).toMatchObject({ offset: 20, limit: 20, kind: "RECEIPT" });
});
