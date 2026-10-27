import { expect, test } from "@playwright/test";
import { installAuthDouble } from "./auth-double";

const desktopOnboardingStorageKey = "m2-health.onboarding.desktop.v1";

test("first-run setup is state-aware, localized, skippable, persistent, and resumable", async ({ page }) => {
  test.setTimeout(60_000);
  await installAuthDouble(page, true, "ACTIVE");
  await page.addInitScript(() => {
    localStorage.setItem("m2-health.tutorial.desktop.v1", JSON.stringify({ dismissed: true, completed: false }));
    const target = window as unknown as { __TAURI_INTERNALS__: { invoke(command: string, args?: Record<string, unknown>): Promise<unknown> } };
    const original = target.__TAURI_INTERNALS__.invoke;
    target.__TAURI_INTERNALS__.invoke = async (command, args) => {
      if (command === "get_phase6_dashboard") return {
        operational: { postedSalesCount: 0, grossSalesMinor: 0, discountMinor: 0, taxMinor: 0, refundedMinor: 0, netSalesMinor: 0, netUnitsSold: 0, currentStockUnits: 0, lowStockPackageCount: 0, purchaseOrderCount: 0, receivedQuantity: 0, purchaseReturnQuantity: 0 },
        financial: { salesRevenueMinor: 0, salesReturnsMinor: 0, netSalesRevenueMinor: 0, refundOutflowMinor: 0, netCogsMinor: 0, missingCogsJournalCount: 0, grossMarginMinor: 0, operatingExpensesMinor: 0, operatingResultMinor: 0 },
        coverage: { eligibleSaleCount: 0, representedSaleCount: 0, unrepresentedSaleCount: 0, postedReturnCount: 0, representedReturnCount: 0, unrepresentedReturnCount: 0, voidCount: 0, representedVoidCount: 0, unrepresentedVoidCount: 0, unrepresentedEventCount: 0, coverageComplete: true, scopeNote: "Finance coverage is complete." },
      };
      return original(command, args);
    };
  });
  await page.goto("/dashboard");
  await page.evaluate((key) => localStorage.removeItem(key), desktopOnboardingStorageKey);
  await page.reload();
  await page.getByLabel("Local username").fill("admin.local");
  await page.getByLabel("Password").fill("VeryStrong!Pass2026");
  await page.getByRole("button", { name: "Sign in" }).click();

  const dialog = page.getByRole("dialog");
  await expect(dialog.getByRole("heading", { name: "Finish setting up M² Health" })).toBeVisible();
  await expect(dialog.getByText("4 of 6 steps complete")).toBeVisible();
  await expect(dialog.getByRole("heading", { name: "Local administrator" }).locator("..").getByText("Complete")).toBeVisible();
  await expect(dialog.getByRole("heading", { name: "First pharmacy branch" }).locator("..").getByText("Complete")).toBeVisible();

  await dialog.getByLabel("Language").selectOption("ar");
  await expect(page.locator("html")).toHaveAttribute("lang", "ar");
  await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
  await expect(dialog.getByRole("heading", { name: "أكمل إعداد M² Health" })).toBeVisible();
  await dialog.getByRole("button", { name: "التخطي الآن" }).click();
  await expect(dialog).toBeHidden();
  await expect.poll(() => page.evaluate((key) => JSON.parse(localStorage.getItem(key) ?? "{}").dismissed, desktopOnboardingStorageKey)).toBe(true);

  await page.reload();
  await page.getByLabel("اسم المستخدم المحلي").fill("admin.local");
  await page.getByLabel("كلمة المرور").fill("VeryStrong!Pass2026");
  await page.getByRole("button", { name: "تسجيل الدخول" }).click();
  await expect(dialog).toBeHidden();
  await page.getByRole("button", { name: /Local Administrator.*Administrator/ }).click();
  await page.getByRole("menuitem", { name: "قائمة إعداد النظام" }).click();
  await dialog.getByLabel("أنشأت حساب البوابة أو يمكنني الوصول إليه").check();
  await dialog.getByLabel("راجعت إجراءات الإعداد الأساسية").check();
  await expect(dialog.getByText("اكتملت 6 من 6 خطوات")).toBeVisible();
  await dialog.getByRole("button", { name: "المتابعة إلى لوحة التحكم" }).click();
  await expect(page).toHaveURL(/\/dashboard$/);
});
