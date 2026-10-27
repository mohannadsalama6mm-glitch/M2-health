import { expect, test, type Page } from "@playwright/test";
import { installAuthDouble } from "./auth-double";

async function installEmptyWorkspace(page: Page) {
  await page.addInitScript(() => {
    localStorage.setItem("m2-health.onboarding.desktop.v1", JSON.stringify({ dismissed: true }));
    const target = window as unknown as { __TAURI_INTERNALS__: { invoke(command: string, args?: Record<string, unknown>): Promise<unknown> } };
    const original = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      if (["list_manufacturers", "list_categories", "list_routes"].includes(command)) return [];
      if (command === "list_products") return { items: [], total: 0, activeTotal: 0, limit: 10, offset: 0 };
      if (command === "get_stock_overview") return { items: [], total: 0, limit: 10, offset: 0 };
      if (command === "get_stock_summary") return { totalPackages: 0, totalUnits: 0, lowStock: 0, outOfStock: 0, expiring: 0 };
      if (command === "list_stock_counts") return { items: [], total: 0, limit: 8, offset: 0 };
      if (command === "list_purchase_orders" || command === "list_purchase_returns" || command === "list_canonical_suppliers" || command === "list_canonical_customers") return { items: [], total: 0, limit: 20, offset: 0 };
      if (command === "search_pos_products" || command === "list_canonical_sales") return { items: [], total: 0 };
      if (command === "search_purchase_packages") return { items: [], total: 0, limit: 20, offset: 0 };
      if (command === "preview_sale_totals") return { subtotalMinor: 0, discountMinor: 0, taxMinor: 0, totalMinor: 0, lines: [] };
      if (command === "get_phase6_dashboard") return {
        operational: { postedSalesCount: 0, grossSalesMinor: 0, discountMinor: 0, taxMinor: 0, refundedMinor: 0, netSalesMinor: 0, netUnitsSold: 0, currentStockUnits: 0, lowStockPackageCount: 0, purchaseOrderCount: 0, receivedQuantity: 0, purchaseReturnQuantity: 0 },
        financial: { salesRevenueMinor: 0, salesReturnsMinor: 0, netSalesRevenueMinor: 0, refundOutflowMinor: 0, netCogsMinor: 0, missingCogsJournalCount: 0, grossMarginMinor: 0, operatingExpensesMinor: 0, operatingResultMinor: 0 },
        coverage: { eligibleSaleCount: 0, representedSaleCount: 0, unrepresentedSaleCount: 0, postedReturnCount: 0, representedReturnCount: 0, unrepresentedReturnCount: 0, voidCount: 0, representedVoidCount: 0, unrepresentedVoidCount: 0, unrepresentedEventCount: 0, coverageComplete: true, scopeNote: "Finance coverage is complete." },
      };
      return original(command, args);
    };
  });
}

async function signIn(page: Page) {
  await page.goto("/dashboard");
  await page.getByLabel("Local username").fill("admin.local");
  await page.getByLabel("Password").fill("VeryStrong!Pass2026");
  await page.getByRole("button", { name: "Sign in" }).click();
  await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
}

test.beforeEach(async ({ page }) => {
  await installAuthDouble(page, true, "ACTIVE");
  await installEmptyWorkspace(page);
});

test("navigation hierarchy, branch scope, shortcut truthfulness, localization, and onboarding reopen", async ({ page }) => {
  await signIn(page);
  await expect(page.locator('[data-navigation-priority="primary"]')).toHaveCount(3);
  await expect(page.locator('[data-navigation-priority="secondary"]')).toHaveCount(2);
  await expect(page.locator('[data-navigation-priority="utility"]')).toHaveCount(2);
  await expect(page.getByText("Ctrl K")).toHaveCount(0);
  await expect(page.getByLabel("Report / filter scope")).toContainText("All branches");
  await expect(page.getByLabel("Report / filter scope")).toContainText("Main Pharmacy");

  await page.getByLabel("Language").selectOption("ar");
  await expect(page.locator("html")).toHaveAttribute("lang", "ar");
  await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
  await expect(page.getByLabel("نطاق التقرير / المرشح")).toContainText("كل الفروع");
  await page.getByRole("button", { name: /Local Administrator.*Administrator/ }).click();
  await page.getByRole("menuitem", { name: "قائمة إعداد النظام" }).click();
  await expect(page.getByRole("dialog").getByRole("heading", { name: "أكمل إعداد M² Health" })).toBeVisible();
});

test("core empty states provide a localized primary next action", async ({ page }) => {
  await signIn(page);
  const cases = [
    ["/catalog", "No catalog items yet", "Add first product", "Shared catalog · branch stock", "لا توجد أصناف في الدليل بعد"],
    ["/purchases", "No purchase orders yet", "Create purchase order", "Purchasing scope", "لا توجد أوامر شراء بعد"],
    ["/inventory", "No stock is available in this branch", "Add stock", "Active operational branch", "لا يوجد مخزون متاح في هذا الفرع"],
    ["/inventory/counts", "No stock counts yet", "Start stock count", "Active operational branch", "لا توجد عمليات جرد بعد"],
    ["/sales", "Nothing is ready to sell", "Add catalog item", "Active operational branch", "لا توجد أصناف جاهزة للبيع"],
    ["/sales/history", "No sales yet", "Start a sale", "Active operational branch", "لا توجد مبيعات بعد"],
    ["/suppliers", "No suppliers yet", "Add supplier", "Shared partner directory", "لا يوجد موردون بعد"],
    ["/customers", "No customers yet", "Add customer", "Shared partner directory", "لا يوجد عملاء بعد"],
    ["/purchases/returns", "No supplier returns yet", "Review purchases", "Active operational branch", "لا توجد مرتجعات موردين بعد"],
  ] as const;
  for (const [route, title, action, scope] of cases) {
    await page.evaluate((nextRoute) => {
      history.pushState({}, "", nextRoute);
      window.dispatchEvent(new PopStateEvent("popstate"));
    }, route);
    const emptyState = page.getByRole("heading", { name: title }).locator("..");
    await expect(emptyState).toBeVisible();
    await expect(emptyState.getByRole("button", { name: action }).or(emptyState.getByRole("link", { name: action }))).toBeVisible();
    await expect(page.getByLabel(scope)).toBeVisible();
  }
  await page.evaluate(() => { history.pushState({}, "", "/purchases/new"); window.dispatchEvent(new PopStateEvent("popstate")); });
  const supplierGuidance = page.getByRole("heading", { name: "Add a supplier before purchasing" }).locator("..");
  await expect(supplierGuidance.getByRole("link", { name: "Open suppliers" })).toBeVisible();

  await page.getByLabel("Language").selectOption("ar");
  await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
  for (const [route, , , , arabicTitle] of cases) {
    await page.evaluate((nextRoute) => { history.pushState({}, "", nextRoute); window.dispatchEvent(new PopStateEvent("popstate")); }, route);
    await expect(page.getByRole("heading", { name: arabicTitle })).toBeVisible();
    await expect(page.locator(".scope-notice")).toBeVisible();
    expect(await page.locator("main").evaluate((element) => element.scrollWidth <= element.clientWidth + 1)).toBe(true);
  }
});
