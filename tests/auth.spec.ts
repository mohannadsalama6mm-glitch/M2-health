import { expect, test } from "@playwright/test";
import { installAuthDouble } from "./auth-double";

test("first run creates one local administrator and keeps the session in memory", async ({ page }) => {
  await installAuthDouble(page);
  await page.goto("/dashboard");
  await expect(page.getByRole("heading", { name: "Create local administrator" })).toBeVisible();
  await page.getByLabel("Display name").fill("Local Administrator");
  await page.getByLabel("Username").fill("admin.local");
  await page.getByLabel("Password", { exact: true }).fill("VeryStrong!Pass2026");
  await page.getByLabel("Confirm password").fill("VeryStrong!Pass2026");
  await page.getByRole("button", { name: "Create administrator" }).click();
  await expect(page.getByText("Local Administrator", { exact: true })).toBeVisible();
  expect(await page.evaluate(() => ({ local: localStorage.length, session: sessionStorage.length }))).toEqual({ local: 0, session: 0 });
  await page.getByText("Local Administrator", { exact: true }).click();
  await page.getByRole("menuitem", { name: "Sign out" }).click();
  await expect(page.getByRole("heading", { name: "Welcome back" })).toBeVisible();
  await page.getByLabel("Username").fill("ADMIN.LOCAL");
  await page.getByLabel("Password").fill("wrong-password");
  await page.getByRole("button", { name: "Sign in" }).click();
  await expect(page.getByRole("alert")).toContainText("Invalid username or password.");
});

test("administrator manages canonical users, roles, and branch membership", async ({ page }) => {
  await installAuthDouble(page, true);
  await page.goto("/employees");
  await page.getByLabel("Username").fill("admin.local");
  await page.getByLabel("Password").fill("VeryStrong!Pass2026");
  await page.getByRole("button", { name: "Sign in" }).click();
  await expect(page.getByRole("heading", { name: "Users" })).toBeVisible();
  await page.getByRole("button", { name: "Add user" }).click();
  await page.getByLabel("Display name").fill("Mona Pharmacist");
  await page.getByLabel("Username").fill("mona.pharmacist");
  await page.getByLabel("Temporary password").fill("Mona!Secure2026");
  await page.getByLabel("Primary branch").selectOption("branch-main");
  await page.getByRole("button", { name: "Create user" }).click();
  await expect(page.getByRole("link", { name: /Mona Pharmacist/ })).toBeVisible();
  await page.getByRole("link", { name: /Mona Pharmacist/ }).click();
  await expect(page.getByRole("heading", { name: "Mona Pharmacist" })).toBeVisible();
  await expect(page.getByText("MAIN · Primary")).toBeVisible();
  await page.getByText("Cashier", { exact: true }).click();
  await expect(page.getByText("Cashier", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: /Local Administrator/ }).click();
  await page.getByRole("menuitem", { name: "Roles & permissions" }).click();
  await expect(page.getByRole("heading", { name: "Roles" })).toBeVisible();
  await expect(page.getByText("These durable roles are enforced for every registered native command.")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Administrator" })).toBeVisible();
});

for (const viewport of [
  { width: 1366, height: 768 },
  { width: 1440, height: 900 },
  { width: 1920, height: 1080 },
]) {
  test(`authentication and user administration fit ${viewport.width}x${viewport.height}`, async ({ page }) => {
    await page.setViewportSize(viewport);
    await installAuthDouble(page, true);
    await page.goto("/employees");
    await page.getByLabel("Username").fill("admin.local");
    await page.getByLabel("Password").fill("VeryStrong!Pass2026");
    await page.getByRole("button", { name: "Sign in" }).click();
    await expect(page.getByRole("heading", { name: "Users" })).toBeVisible();
    const overflow = await page.evaluate(() => ({
      horizontal: document.documentElement.scrollWidth > document.documentElement.clientWidth,
      authSecretsPersisted: localStorage.length + sessionStorage.length,
    }));
    expect(overflow).toEqual({ horizontal: false, authSecretsPersisted: 0 });
  });
}
