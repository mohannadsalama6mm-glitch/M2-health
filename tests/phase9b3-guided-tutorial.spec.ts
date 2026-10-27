import { expect, test, type Page } from "@playwright/test";
import { installAuthDouble } from "./auth-double";

const onboardingKey = "m2-health.onboarding.desktop.v1";
const tutorialKey = "m2-health.tutorial.desktop.v1";

async function installWorkspace(page: Page, locale: "en" | "ar" = "en") {
  await installAuthDouble(page, true, "ACTIVE");
  await page.addInitScript(({ onboardingKey, tutorialKey, locale }) => {
    if (!sessionStorage.getItem("phase9b3-seeded")) {
      localStorage.setItem(onboardingKey, JSON.stringify({ dismissed: true }));
      localStorage.removeItem(tutorialKey);
      localStorage.setItem("m2-health.locale.v1", locale);
      sessionStorage.setItem("phase9b3-seeded", "true");
    }
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
  }, { onboardingKey, tutorialKey, locale });
}

async function signIn(page: Page, locale: "en" | "ar" = "en") {
  await page.goto("/dashboard");
  await page.getByLabel(locale === "ar" ? "اسم المستخدم المحلي" : "Local username").fill("admin.local");
  await page.getByLabel(locale === "ar" ? "كلمة المرور" : "Password").fill("VeryStrong!Pass2026");
  await page.getByRole("button", { name: locale === "ar" ? "تسجيل الدخول" : "Sign in" }).click();
}

test("guided tour is first-run, skippable, persistent, and restartable", async ({ page }) => {
  test.setTimeout(90_000);
  await installWorkspace(page);
  await signIn(page);
  const tour = page.getByLabel("Guided tour");
  await expect(tour.getByRole("heading", { name: "Your pharmacy workspace" })).toBeVisible();
  await tour.getByRole("button", { name: "Next" }).click();
  await expect(tour.getByRole("heading", { name: "Confirm the active branch" })).toBeVisible();
  await expect(page.locator('[data-tutorial="branch"]')).toBeVisible();
  await tour.getByRole("button", { name: "Skip tour" }).click();
  await expect(tour).toBeHidden();
  await expect.poll(() => page.evaluate((key) => JSON.parse(localStorage.getItem(key) ?? "{}").dismissed, tutorialKey)).toBe(true);

  await page.reload();
  await signIn(page);
  await expect(tour).toBeHidden();

  await page.getByRole("button", { name: /Local Administrator.*Administrator/ }).click();
  await page.getByRole("menuitem", { name: "Restart guided tour" }).click();
  await expect(tour.getByRole("heading", { name: "Your pharmacy workspace" })).toBeVisible();
  await page.setViewportSize({ width: 700, height: 700 });
  expect(await tour.locator(".guided-tutorial__panel").evaluate((element) => {
    const box = element.getBoundingClientRect();
    return box.left >= 0 && box.right <= window.innerWidth && box.bottom <= window.innerHeight;
  })).toBe(true);
});

test("guided tour completes in Arabic RTL with localized contextual steps", async ({ page }) => {
  test.setTimeout(60_000);
  await installWorkspace(page, "ar");
  await signIn(page, "ar");
  await expect(page.locator("html")).toHaveAttribute("lang", "ar");
  await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
  const tour = page.getByLabel("الجولة الإرشادية");
  await expect(tour.getByRole("heading", { name: "مساحة عمل صيدليتك" })).toBeVisible();
  for (let step = 0; step < 6; step += 1) await tour.getByRole("button", { name: "التالي" }).click();
  await expect(tour.getByRole("heading", { name: "راجع الأداء بوضوح" })).toBeVisible();
  await tour.getByRole("button", { name: "إنهاء الجولة" }).click();
  await expect(tour).toBeHidden();
  await expect.poll(() => page.evaluate((key) => JSON.parse(localStorage.getItem(key) ?? "{}").completed, tutorialKey)).toBe(true);
});
