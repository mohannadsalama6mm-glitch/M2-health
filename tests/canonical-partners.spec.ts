import { expect, test, type Page } from "@playwright/test";
import { installCanonicalPartnersDouble } from "./canonical-partners-double";

async function calls(page: Page, command: string) {
  return page.evaluate(
    (name) =>
      (
        window as unknown as {
          partnerCalls: { command: string; args: Record<string, unknown> }[];
        }
      ).partnerCalls.filter((call) => call.command === name),
    command,
  );
}

test.beforeEach(async ({ page }) => {
  await installCanonicalPartnersDouble(page);
  await page.goto("/suppliers");
});

test("native supplier route uses canonical IPC and never prototype handlers", async ({
  page,
}) => {
  await expect(
    page.getByRole("table", { name: "Canonical suppliers" }),
  ).toContainText("Nile Medical");
  expect(await calls(page, "list_canonical_suppliers")).not.toHaveLength(0);
  expect(await calls(page, "list_suppliers")).toHaveLength(0);
});

test("customer route uses canonical IPC", async ({ page }) => {
  await page.getByRole("link", { name: "Customers", exact: true }).click();
  await expect(
    page.getByRole("table", { name: "Canonical customers" }),
  ).toContainText("Ahmed Hassan");
  expect(await calls(page, "list_canonical_customers")).not.toHaveLength(0);
});

test("loading state is visible", async ({ page }) => {
  await page.evaluate(
    () => ((window as unknown as { partnerDelay: number }).partnerDelay = 500),
  );
  await page.getByRole("link", { name: "Customers", exact: true }).click();
  await expect(
    page.getByText("Loading customers…", { exact: true }),
  ).toBeVisible();
});

test("empty state distinguishes an unused directory", async ({ page }) => {
  await page.evaluate(
    () =>
      ((window as unknown as { partnerEmpty: boolean }).partnerEmpty = true),
  );
  await page.getByRole("link", { name: "Customers", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "No customers yet" }),
  ).toBeVisible();
});

test("no-results state reflects filters", async ({ page }) => {
  await page.getByLabel("Search suppliers").fill("no such supplier");
  await expect(
    page.getByRole("heading", { name: "No suppliers match these filters" }),
  ).toBeVisible();
});

test("backend errors are visible and retryable", async ({ page }) => {
  await page.evaluate(
    () =>
      ((
        window as unknown as { partnerFailCommand: string }
      ).partnerFailCommand = "list_canonical_customers"),
  );
  await page.getByRole("link", { name: "Customers", exact: true }).click();
  await expect(page.getByText("Controlled backend rejection")).toBeVisible();
  await page.evaluate(
    () =>
      ((
        window as unknown as { partnerFailCommand: string }
      ).partnerFailCommand = ""),
  );
  await page.getByRole("button", { name: "Retry" }).click();
  await expect(
    page.getByRole("table", { name: "Canonical customers" }),
  ).toBeVisible();
});

test("search is debounced and sent to the backend", async ({ page }) => {
  await page.getByLabel("Search suppliers").fill("Dina");
  await expect
    .poll(
      async () =>
        (await calls(page, "list_canonical_suppliers")).at(-1)?.args.query,
    )
    .toMatchObject({ search: "Dina", limit: 20, offset: 0 });
});

test("active filter is sent to the backend", async ({ page }) => {
  await page.getByLabel("Partner status").selectOption("inactive");
  await expect(
    page.getByRole("table", { name: "Canonical suppliers" }),
  ).toContainText("Delta Pharma");
  await expect
    .poll(
      async () =>
        (await calls(page, "list_canonical_suppliers")).at(-1)?.args.query,
    )
    .toMatchObject({ isActive: false });
});

test("customer pagination uses SQL-style offsets", async ({ page }) => {
  await page.goto("/customers");
  await expect(page.getByText("25 total · Page 1")).toBeVisible();
  await page.getByRole("button", { name: "Next" }).click();
  await expect(page.getByText("25 total · Page 2")).toBeVisible();
  await expect
    .poll(
      async () =>
        (await calls(page, "list_canonical_customers")).at(-1)?.args.query,
    )
    .toMatchObject({ limit: 20, offset: 20 });
});

test("supplier create sends the full canonical field set", async ({ page }) => {
  await page.getByRole("button", { name: "Add supplier" }).click();
  const dialog = page.getByRole("dialog");
  await dialog.getByLabel("Name").fill("New Supplier");
  await dialog.getByLabel("Contact person").fill("Mona");
  await dialog.getByLabel("Primary phone").fill("0101");
  await dialog.getByLabel("Tax / registration ID").fill("REG-2");
  await dialog.getByRole("button", { name: "Save supplier" }).click();
  await expect(page.getByText("New Supplier added.")).toBeVisible();
  expect(
    (await calls(page, "create_canonical_supplier")).at(-1)?.args.input,
  ).toMatchObject({
    displayName: "New Supplier",
    contactPerson: "Mona",
    primaryPhone: "0101",
    taxRegistrationId: "REG-2",
    secondaryPhone: null,
  });
});

test("customer create omits supplier-only fields", async ({ page }) => {
  await page.goto("/customers");
  await page.getByRole("button", { name: "Add customer" }).click();
  const dialog = page.getByRole("dialog");
  await dialog.getByLabel("Name").fill("New Customer");
  await dialog.getByRole("button", { name: "Save customer" }).click();
  const input = (await calls(page, "create_canonical_customer")).at(-1)?.args
    .input as Record<string, unknown>;
  expect(input).not.toHaveProperty("contactPerson");
  expect(input).not.toHaveProperty("taxRegistrationId");
});

test("frontend requires a name before create", async ({ page }) => {
  await page.getByRole("button", { name: "Add supplier" }).click();
  await expect(
    page.getByRole("button", { name: "Save supplier" }),
  ).toBeDisabled();
  expect(await calls(page, "create_canonical_supplier")).toHaveLength(0);
});

test("backend create rejection remains in the dialog", async ({ page }) => {
  await page.evaluate(
    () =>
      ((
        window as unknown as { partnerFailCommand: string }
      ).partnerFailCommand = "create_canonical_supplier"),
  );
  await page.getByRole("button", { name: "Add supplier" }).click();
  await page.getByRole("dialog").getByLabel("Name").fill("Rejected");
  await page.getByRole("button", { name: "Save supplier" }).click();
  await expect(page.getByRole("dialog")).toContainText(
    "Controlled backend rejection",
  );
});

test("supplier detail shows real linked purchasing documents", async ({
  page,
}) => {
  await page.getByRole("link", { name: /Nile Medical/ }).click();
  await expect(
    page.getByRole("heading", { name: "Nile Medical" }),
  ).toBeVisible();
  await expect(page.getByText("SUP-000001 · Revision 1")).toBeVisible();
  await expect(page.getByRole("table", { name: "Supplier purchase orders" })).toContainText("PUR-0001");
  await expect(page.getByRole("table", { name: "Supplier receipts" })).toContainText("REC-0001");
  await expect(page.getByRole("table", { name: "Supplier returns" })).toContainText("PRN-0001");
});

test("customer detail shows backend-paginated sales with navigation", async ({ page }) => {
  await page.goto("/customers/c1");
  const table = page.getByRole("table", { name: "Customer sales history" });
  await expect(table).toContainText("SAL-20261003-0001");
  await expect(table.getByRole("link", { name: "SAL-20261003-0001" })).toHaveAttribute("href", "/sales/sale-customer-1");
  expect((await calls(page, "list_customer_sales")).at(-1)?.args.query).toMatchObject({ customerId: "c1", limit: 10, offset: 0 });
});
test("customer history exposes loading, empty and structured error states", async ({ page }) => {
  await page.evaluate(() => ((window as unknown as { partnerDelay: number }).partnerDelay = 500));
  await page.getByRole("link", { name: "Customers", exact: true }).click();
  await page.getByRole("link", { name: /Ahmed Hassan/ }).click();
  await expect(page.getByText("Loading transaction history…")).toBeVisible();
  await page.evaluate(() => {
    const w = window as unknown as { partnerDelay: number; partnerHistoryTotal: number };
    w.partnerDelay = 0; w.partnerHistoryTotal = 0;
  });
  await page.getByRole("link", { name: "Customers", exact: true }).first().click();
  await page.getByRole("link", { name: /Ahmed Hassan/ }).click();
  await expect(page.getByText("No sales are linked to this customer.")).toBeVisible();
  await page.evaluate(() => ((window as unknown as { partnerFailCommand: string }).partnerFailCommand = "list_customer_sales"));
  await page.getByRole("link", { name: "Customers", exact: true }).first().click();
  await page.getByRole("link", { name: /Ahmed Hassan/ }).click();
  await expect(page.getByText("Controlled backend rejection")).toBeVisible();
});
test("customer history pagination is backend driven", async ({ page }) => {
  await page.evaluate(() => ((window as unknown as { partnerHistoryTotal: number }).partnerHistoryTotal = 25));
  await page.getByRole("link", { name: "Customers", exact: true }).click();
  await page.getByRole("link", { name: /Ahmed Hassan/ }).click();
  await expect(page.getByRole("table", { name: "Customer sales history" })).toBeVisible();
  const next = page.getByRole("button", { name: "Next" }).last();
  await expect(next).toBeEnabled();
  await next.click();
  await expect.poll(async () => (await calls(page, "list_customer_sales")).at(-1)?.args.query).toMatchObject({ customerId: "c1", limit: 10, offset: 10 });
});

test("edit sends expected revision", async ({ page }) => {
  await page.goto("/suppliers/s1");
  await page.getByRole("button", { name: "Edit supplier" }).click();
  await page.getByRole("dialog").getByLabel("Name").fill("Nile Updated");
  await page.getByRole("button", { name: "Save supplier" }).click();
  expect(
    (await calls(page, "update_canonical_supplier")).at(-1)?.args.input,
  ).toMatchObject({
    id: "s1",
    expectedRevision: 1,
    displayName: "Nile Updated",
  });
  await expect(
    page.getByRole("heading", { name: "Nile Updated" }),
  ).toBeVisible();
});

test("deactivate sends expected revision and updates status", async ({
  page,
}) => {
  await page.goto("/suppliers/s1");
  await page.getByRole("button", { name: "Deactivate" }).click();
  expect(
    (await calls(page, "set_canonical_supplier_active")).at(-1)?.args.input,
  ).toEqual({ id: "s1", expectedRevision: 1, active: false });
  await expect(page.getByText("supplier deactivated.")).toBeVisible();
  await expect(page.getByRole("button", { name: "Activate" })).toBeVisible();
});

test("status conflict is displayed", async ({ page }) => {
  await page.goto("/suppliers/s1");
  await page.evaluate(
    () =>
      ((
        window as unknown as { partnerFailCommand: string }
      ).partnerFailCommand = "set_canonical_supplier_active"),
  );
  await page.getByRole("button", { name: "Deactivate" }).click();
  await expect(page.getByText("Controlled backend rejection")).toBeVisible();
});

for (const [width, height] of [
  [1366, 768],
  [1440, 900],
  [1920, 1080],
]) {
  test(`partner list fits supported viewport ${width}`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    await page.goto("/customers");
    await expect(
      page.getByRole("table", { name: "Canonical customers" }),
    ).toBeVisible();
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
  });
  test(`integrated partner detail fits supported viewport ${width}`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    await page.goto("/suppliers/s1");
    await expect(page.getByRole("table", { name: "Supplier purchase orders" })).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  });
}

test("create dialog remains usable at minimum viewport and focuses name", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1366, height: 768 });
  await page.getByRole("button", { name: "Add supplier" }).click();
  const name = page.getByRole("dialog").getByLabel("Name");
  await expect(name).toBeFocused();
  const save = page.getByRole("button", { name: "Save supplier" });
  await save.scrollIntoViewIfNeeded();
  await expect(save).toBeInViewport();
});
