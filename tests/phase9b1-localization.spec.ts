import { expect, test } from "@playwright/test";
import { installAuthDouble } from "./auth-double";

test("desktop language switch persists and drives Arabic RTL / English LTR", async ({ page }) => {
  test.setTimeout(60_000);
  await installAuthDouble(page, true, "ACTIVE");
  await page.addInitScript(() => {
    localStorage.setItem("m2-health.onboarding.desktop.v1", JSON.stringify({ dismissed: true }));
    localStorage.setItem("m2-health.tutorial.desktop.v1", JSON.stringify({ dismissed: true, completed: false }));
    const target = window as unknown as { __TAURI_INTERNALS__: { invoke(command: string, args?: Record<string, unknown>): Promise<unknown> } };
    const original = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      if (command === "get_phase6_dashboard") return {
        operational: { postedSalesCount: 3, grossSalesMinor: 30000, discountMinor: 1000, taxMinor: 0, refundedMinor: 5000, netSalesMinor: 25000, netUnitsSold: 8, currentStockUnits: 42, lowStockPackageCount: 2, purchaseOrderCount: 4, receivedQuantity: 30, purchaseReturnQuantity: 2 },
        financial: { salesRevenueMinor: 29000, salesReturnsMinor: 5000, netSalesRevenueMinor: 24000, refundOutflowMinor: 5000, netCogsMinor: 12000, missingCogsJournalCount: 0, grossMarginMinor: 12000, operatingExpensesMinor: 3000, operatingResultMinor: 9000 },
        coverage: { eligibleSaleCount: 3, representedSaleCount: 2, unrepresentedSaleCount: 1, postedReturnCount: 1, representedReturnCount: 1, unrepresentedReturnCount: 0, voidCount: 0, representedVoidCount: 0, unrepresentedVoidCount: 0, unrepresentedEventCount: 1, coverageComplete: false, scopeNote: "Finance coverage is limited to posted journals." },
      };
      return original(command, args);
    };
  });
  await page.goto("/dashboard");
  await page.getByLabel("Local username").fill("admin.local");
  await page.getByLabel("Password").fill("VeryStrong!Pass2026");
  await page.getByRole("button", { name: "Sign in" }).click();
  await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
  await page.getByLabel("Language").selectOption("ar");
  await expect(page.locator("html")).toHaveAttribute("lang", "ar");
  await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
  await expect(page.getByRole("heading", { name: "لوحة التحكم" })).toBeVisible();
  await expect(page.getByRole("link", { name: "المشتريات" })).toBeVisible();
  await page.reload();
  await expect(page.locator("html")).toHaveAttribute("lang", "ar");
  await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
  await expect(page.getByRole("heading", { name: "مرحبًا بعودتك" })).toBeVisible();
  await page.getByLabel("اسم المستخدم المحلي").fill("admin.local");
  await page.getByLabel("كلمة المرور").fill("VeryStrong!Pass2026");
  await page.getByRole("button", { name: "تسجيل الدخول" }).click();
  await expect(page.getByRole("heading", { name: "لوحة التحكم" })).toBeVisible();
  await page.getByLabel("اللغة").selectOption("en");
  await expect(page.locator("html")).toHaveAttribute("lang", "en");
  await expect(page.locator("html")).toHaveAttribute("dir", "ltr");
  await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
});
