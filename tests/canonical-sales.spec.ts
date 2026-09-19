import { test, expect, type Page } from "@playwright/test";
import { installSalesDouble } from "./canonical-sales-double";
test.beforeEach(async ({ page }) => {
  await installSalesDouble(page);
  await page.goto("/sales");
});
for (const [width, height] of [
  [1366, 768],
  [1440, 900],
  [1920, 1080],
])
  test(`canonical POS checkout remains visible ${width}`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    await add(page);
    const button = page.getByRole("button", {
      name: "Payment / Post",
      exact: true,
    });
    await expect(button).toBeInViewport();
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
    await page.screenshot({ path: `docs/phase-4/pos-${width}.png` });
  });
async function calls(page: Page, command: string) {
  return page.evaluate(
    (c) =>
      (
        window as unknown as {
          salesCalls: { command: string; args: Record<string, unknown> }[];
        }
      ).salesCalls.filter((x) => x.command === c),
    command,
  );
}
async function add(page: Page) {
  await page
    .getByRole("table", { name: "POS Catalog results" })
    .getByRole("button", { name: "Add", exact: true })
    .click();
  await expect(page.getByLabel("Validated totals")).toContainText(
    "Total 1.00 EGP",
  );
}
async function post(page: Page) {
  await add(page);
  await page
    .getByRole("button", { name: "Payment / Post", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Confirm Post Sale", exact: true })
    .click();
  await expect(
    page.getByRole("heading", { name: "SAL-20260915-0001" }),
  ).toBeVisible();
}
test("real POS capability uses canonical search", async ({ page }) => {
  await expect(
    page.getByRole("table", { name: "POS Catalog results" }),
  ).toContainText("Canonical medicine");
  expect((await calls(page, "search_pos_products")).length).toBeGreaterThan(0);
  expect(await calls(page, "pos_search")).toHaveLength(0);
});
test("barcode goes to SQLite search", async ({ page }) => {
  await page.getByLabel("POS search / barcode").fill("123456");
  await expect
    .poll(
      async () => (await calls(page, "search_pos_products")).at(-1)?.args.query,
    )
    .toMatchObject({ search: "123456" });
});
test("Arabic search goes to backend unchanged", async ({ page }) => {
  await page.getByLabel("POS search / barcode").fill("دواء");
  await expect
    .poll(
      async () => (await calls(page, "search_pos_products")).at(-1)?.args.query,
    )
    .toMatchObject({ search: "دواء" });
});
test("add package to cart", async ({ page }) => {
  await add(page);
  await expect(page.getByRole("table", { name: "Sale cart" })).toContainText(
    "Canonical medicine",
  );
});
test("duplicate adds merge package quantity", async ({ page }) => {
  await add(page);
  await page.getByRole("button", { name: "Add", exact: true }).click();
  await expect(page.getByLabel("Quantity pkg-1", { exact: true })).toHaveValue(
    "2",
  );
  await expect(
    page.getByRole("table", { name: "Sale cart" }).locator("tbody tr"),
  ).toHaveCount(1);
});
test("quantity change requests backend totals", async ({ page }) => {
  await add(page);
  await page.getByLabel("Quantity pkg-1", { exact: true }).fill("3");
  await expect(page.getByLabel("Validated totals")).toContainText(
    "Total 3.00 EGP",
  );
});
test("line and sale discounts are sent for validated totals", async ({
  page,
}) => {
  await add(page);
  await page.getByLabel("Line discount pkg-1").fill("0.10");
  await page.getByLabel("Sale discount (EGP)").fill("0.05");
  await expect(page.getByLabel("Validated totals")).toContainText(
    "Total 0.85 EGP",
  );
});
test("optional integer tax is included by backend", async ({ page }) => {
  await add(page);
  await page.getByLabel("Tax amount (EGP, optional)").fill("0.07");
  await expect(page.getByLabel("Validated totals")).toContainText(
    "Total 1.07 EGP",
  );
});
test("insufficient eligible stock prevents payment", async ({ page }) => {
  await add(page);
  await page.getByLabel("Quantity pkg-1", { exact: true }).fill("9");
  await expect(page.getByText("Insufficient non-expired stock")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Payment / Post", exact: true }),
  ).toBeDisabled();
});
test("exact payment rejection remains visible", async ({ page }) => {
  await add(page);
  await page
    .getByRole("button", { name: "Payment / Post", exact: true })
    .click();
  await page.getByLabel("Payment amount (EGP)").fill("0.50");
  await page.getByRole("button", { name: "Confirm Post Sale" }).click();
  await expect(page.getByRole("dialog")).toContainText(
    "Payments must equal the exact sale total",
  );
});
test("hold persists without posting", async ({ page }) => {
  await add(page);
  await page.getByRole("button", { name: "Hold Sale" }).click();
  await expect(
    page.getByText("Held sale persisted. No stock reserved."),
  ).toBeVisible();
  expect(await calls(page, "post_sale")).toHaveLength(0);
});
test("held cart resumes after page reload", async ({ page }) => {
  await add(page);
  await page.getByRole("button", { name: "Hold Sale" }).click();
  await expect(
    page.getByText("Held sale persisted. No stock reserved."),
  ).toBeVisible();
  await page.goto("/sales/history");
  await page
    .getByRole("table", { name: "Canonical sales history" })
    .getByRole("link")
    .click();
  await page.getByRole("button", { name: "Resume Cart" }).click();
  await expect(page.getByLabel("Quantity pkg-1", { exact: true })).toHaveValue(
    "1",
  );
});
test("sale posts with canonical request ID and payment", async ({ page }) => {
  await post(page);
  const p = (await calls(page, "post_sale"))[0].args.input as {
    requestId: string;
    payments: unknown[];
  };
  expect(p.requestId).toMatch(/^[0-9a-f-]{36}$/);
  expect(p.payments).toEqual([
    { paymentMethod: "CASH", amountMinor: 100, reference: null },
  ]);
});
test("duplicate submit is disabled while posting", async ({ page }) => {
  await add(page);
  await page.evaluate(() => {
    (window as unknown as { salesDelay: number }).salesDelay = 1000;
  });
  await page
    .getByRole("button", { name: "Payment / Post", exact: true })
    .click();
  await page.getByRole("button", { name: "Confirm Post Sale" }).click();
  await expect(
    page.getByRole("button", { name: "Posting…", exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole("heading", { name: "SAL-20260915-0001" }),
  ).toBeVisible();
  expect(await calls(page, "post_sale")).toHaveLength(1);
});
test("receipt preview uses posted snapshots", async ({ page }) => {
  await post(page);
  await page.getByRole("button", { name: "Receipt Preview" }).click();
  await expect(page.getByRole("dialog")).toContainText("Canonical medicine");
  await expect(page.getByRole("dialog")).toContainText("Total 1.00 EGP");
});
test("sales history displays persisted sale", async ({ page }) => {
  await post(page);
  await page.getByRole("link", { name: "Sales History / Held" }).click();
  await expect(
    page.getByRole("table", { name: "Canonical sales history" }),
  ).toContainText("SAL-20260915-0001");
});
test("details expose batch costs and movement ID", async ({ page }) => {
  await post(page);
  await expect(
    page.getByRole("table", { name: "Allocations item-0" }),
  ).toContainText("0.30 EGP");
  await expect(
    page.getByRole("link", { name: "sale-movement-1" }),
  ).toBeVisible();
});
test("return previews refund and restores original allocation trace", async ({
  page,
}) => {
  await post(page);
  await page.getByRole("button", { name: "Return Items" }).click();
  await page.getByLabel("Return quantity Canonical medicine").fill("1");
  await page.getByLabel("Return reason").fill("Controlled return");
  await page.getByRole("button", { name: "Calculate Refund" }).click();
  await expect(page.getByText("Validated refund: 1.00 EGP")).toBeVisible();
  await page.getByRole("button", { name: "Confirm Return" }).click();
  await expect(page.getByText(/SALE_RETURN return-movement-1/)).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Void Sale", exact: true }),
  ).toBeDisabled();
});
test("void requires reason and explicit confirmation", async ({ page }) => {
  await post(page);
  await page.getByRole("button", { name: "Void Sale", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Confirm Void" }),
  ).toBeDisabled();
  await page.getByLabel("Void reason").fill("Administrative correction");
  await page.getByRole("button", { name: "Confirm Void" }).click();
  await expect(page.getByText(/reversal void-movement-1/)).toBeVisible();
});
test("backend search error never shows prototype fallback", async ({
  page,
}) => {
  await page.evaluate(() => {
    (window as unknown as { salesFailCommand: string }).salesFailCommand =
      "search_pos_products";
  });
  await page.getByLabel("POS search / barcode").fill("failure");
  await expect(page.getByText("Controlled backend rejection")).toBeVisible();
  await expect(
    page.getByRole("table", { name: "POS Catalog results" }),
  ).toHaveCount(0);
});
test("search loading hides stale results", async ({ page }) => {
  await expect(
    page.getByRole("table", { name: "POS Catalog results" }),
  ).toBeVisible();
  await page.evaluate(() => {
    (window as unknown as { salesDelay: number }).salesDelay = 1000;
  });
  await page.getByLabel("POS search / barcode").fill("next");
  await expect(page.getByText("Searching SQLite Catalog…")).toBeVisible();
  await expect(
    page.getByRole("table", { name: "POS Catalog results" }),
  ).toHaveCount(0);
});
test("Enter adds the selected search result", async ({ page }) => {
  await expect(
    page.getByRole("table", { name: "POS Catalog results" }),
  ).toBeVisible();
  await page.getByLabel("POS search / barcode").press("Enter");
  await expect(page.getByLabel("Quantity pkg-1", { exact: true })).toHaveValue(
    "1",
  );
});
test("Delete removes a focused cart line", async ({ page }) => {
  await add(page);
  await page.getByLabel("Quantity pkg-1", { exact: true }).press("Delete");
  await expect(page.getByText("Add a package to begin.")).toBeVisible();
});
test("history pagination sends bounded offset", async ({ page }) => {
  await post(page);
  await page.evaluate(() => {
    (window as unknown as { salesTotal: number }).salesTotal = 41;
  });
  await page.getByRole("link", { name: "Sales History / Held" }).click();
  await page.getByRole("button", { name: "Next sales" }).click();
  await expect
    .poll(
      async () =>
        (await calls(page, "list_canonical_sales")).at(-1)?.args.query,
    )
    .toMatchObject({ offset: 20, limit: 20 });
});
test("failed post retry preserves original payload and key", async ({
  page,
}) => {
  await add(page);
  await page.evaluate(() => {
    (window as unknown as { salesFailCommand: string }).salesFailCommand =
      "post_sale";
  });
  await page
    .getByRole("button", { name: "Payment / Post", exact: true })
    .click();
  await page.getByRole("button", { name: "Confirm Post Sale" }).click();
  await expect(page.getByRole("dialog")).toContainText(
    "Controlled backend rejection",
  );
  await page.evaluate(() => {
    (window as unknown as { salesFailCommand: string }).salesFailCommand = "";
  });
  await page.getByRole("button", { name: "Retry same sale request" }).click();
  await expect(
    page.getByRole("heading", { name: "SAL-20260915-0001" }),
  ).toBeVisible();
  const p = await calls(page, "post_sale");
  expect(p[0].args).toEqual(p[1].args);
});
