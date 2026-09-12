import { test, expect } from "@playwright/test";
for (const [width, height] of [
  [1366, 768],
  [1440, 900],
  [1920, 1080],
]) {
  test(`dashboard at ${width}x${height}`, async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (e) => errors.push(e.message));
    page.on("console", (m) => {
      if (m.type() === "error") errors.push(m.text());
    });
    await page.setViewportSize({ width, height });
    await page.goto("/");
    await expect(
      page.getByRole("heading", { name: "Good morning, Mohannad" }),
    ).toBeVisible();
    await expect(page.getByText("UI demo · sample data")).toBeVisible();
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= window.innerWidth,
      ),
    ).toBe(true);
    await expect(
      page.getByRole("navigation", { name: "Pharmacy navigation" }),
    ).not.toContainText("Design system");
    await page
      .getByRole("button", { name: "Collapse sidebar", exact: true })
      .click();
    await expect(page.locator(".app-shell")).toHaveClass(/collapsed/);
    await page
      .getByRole("button", { name: "Expand sidebar", exact: true })
      .click();
    await page.keyboard.press("Control+k");
    await expect(
      page.getByRole("searchbox", { name: "Global search" }),
    ).toBeFocused();
    const bounds = await page.locator(".sidebar").boundingBox();
    await page
      .locator("main")
      .evaluate((el) => (el.scrollTop = el.scrollHeight));
    expect(await page.locator(".sidebar").boundingBox()).toEqual(bounds);
    expect(await page.evaluate(() => window.scrollY)).toBe(0);
    await page.locator("main").evaluate((el) => (el.scrollTop = 0));
    await expect(page.locator(".brand-superscript")).toBeVisible();
    await page.screenshot({
      path: `test-results/dashboard-${width}.png`,
      fullPage: true,
    });
    expect(errors).toEqual([]);
  });
}
test("design system controls and keyboard accessibility", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/design-system");
  await expect(
    page.getByRole("heading", { name: "The M² Health design system" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Open modal", exact: true }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
  for (let i = 0; i < 5; i++) {
    await page.keyboard.press("Tab");
    expect(
      await page.evaluate(
        () => !!document.activeElement?.closest('[role="dialog"]'),
      ),
    ).toBe(true);
  }
  await page.keyboard.press("Escape");
  await expect(
    page.getByRole("button", { name: "Open modal", exact: true }),
  ).toBeFocused();
  await page.getByRole("button", { name: "Open drawer", exact: true }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await page.keyboard.press("Escape");
  await page.getByRole("tab", { name: "Product", exact: true }).click();
  await page.keyboard.press("ArrowRight");
  await expect(
    page.getByRole("tab", { name: "Pricing", exact: true }),
  ).toHaveAttribute("aria-selected", "true");
  await page.getByRole("button", { name: "Increase quantity" }).click();
  await expect(page.getByRole("spinbutton", { name: "Quantity" })).toHaveValue(
    "2",
  );
  await page
    .getByRole("searchbox", { name: "Filter demo customers", exact: true })
    .fill("Sara");
  await expect(
    page
      .getByRole("table", { name: "Component demo transactions" })
      .getByRole("row"),
  ).toHaveCount(2);
  await page.getByRole("switch", { name: "RTL preview" }).click();
  await expect(page.locator(".showcase")).toHaveAttribute("dir", "rtl");
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
  expect(errors).toEqual([]);
  await page.screenshot({
    path: "test-results/design-system.png",
    fullPage: true,
  });
});
