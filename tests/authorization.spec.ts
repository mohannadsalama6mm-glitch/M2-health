import { expect, test } from "@playwright/test";
import { installAuthorizationDouble } from "./authorization-double";

async function login(page: Parameters<typeof installAuthorizationDouble>[0]) {
  await page.getByLabel("Username").fill("staff.local");
  await page.getByLabel("Password").fill("VeryStrong!Pass2026");
  await page.getByRole("button", { name: "Sign in" }).click();
}
async function clientNavigate(page: Parameters<typeof installAuthorizationDouble>[0], path: string) {
  await page.evaluate((next) => {
    window.history.pushState({}, "", next);
    window.dispatchEvent(new PopStateEvent("popstate"));
  }, path);
}

test("cashier navigation and direct routes reflect backend capabilities", async ({ page }) => {
  await installAuthorizationDouble(page, "CASHIER");
  await page.goto("/employees");
  await login(page);
  await expect(page.getByRole("heading", { name: "Access denied" })).toBeVisible();
  await expect(page.getByRole("link", { name: "POS / Sales" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Customers" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Finance" })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Employees" })).toHaveCount(0);
  await clientNavigate(page, "/finance");
  await expect(page.getByRole("heading", { name: "Access denied" })).toBeVisible();
  for (const path of ["/catalog/import", "/purchases/new", "/inventory/counts"]) {
    await clientNavigate(page, path);
    await expect(page.getByRole("heading", { name: "Access denied" })).toBeVisible();
  }
});

test("branch selector exposes memberships and propagates the selected branch", async ({ page }) => {
  await installAuthorizationDouble(page, "CASHIER");
  await page.goto("/employees");
  await login(page);
  const selector = page.getByLabel("Branch", { exact: true });
  await expect(selector.locator("option")).toHaveCount(2);
  await selector.selectOption("branch-b");
  await clientNavigate(page, "/catalog");
  await expect.poll(() => page.evaluate(() => {
    const calls = (window as unknown as { __phase7bCalls: Array<{ command: string; args: Record<string, unknown> }> }).__phase7bCalls;
    return calls.findLast((call) => call.command === "list_products")?.args.authorizationBranchId;
  })).toBe("branch-b");
});

test("unauthorized backend response clears the in-memory session", async ({ page }) => {
  await installAuthorizationDouble(page, "ADMIN");
  await page.goto("/employees");
  await login(page);
  await expect(page.getByRole("heading", { name: "Users" })).toBeVisible();
  await page.evaluate(() => (window as unknown as { __invalidatePhase7bSession(): void }).__invalidatePhase7bSession());
  await page.getByPlaceholder("Name or username").fill("expired");
  await expect(page.getByRole("heading", { name: "Welcome back" })).toBeVisible();
  await expect(page.getByText("Your session ended. Sign in again.")).toBeVisible();
  await expect(page.evaluate(() => ({ local: localStorage.length, session: sessionStorage.length }))).resolves.toEqual({ local: 0, session: 0 });
});

test("cashier cannot invoke finance export in the native double", async ({ page }) => {
  await installAuthorizationDouble(page, "CASHIER");
  await page.goto("/employees");
  await login(page);
  const result = await page.evaluate(async () => {
    try {
      await (window as unknown as { __TAURI_INTERNALS__: { invoke(command: string, args: Record<string, unknown>): Promise<unknown> } }).__TAURI_INTERNALS__.invoke("export_phase6_csv", { sessionToken: "c".repeat(64), input: { kind: "SALES" } });
      return "allowed";
    } catch (error) {
      return (error as { code: string }).code;
    }
  });
  expect(result).toBe("forbidden");
});

test("typed IPC replaces supplied tokens and actor identities with the in-memory principal", async ({ page }) => {
  await installAuthorizationDouble(page, "CASHIER");
  await page.goto("/employees");
  await login(page);
  await page.evaluate(async () => {
    const client = await import("/src/lib/tauri/client.ts");
    await client.nativeRequest("list_products", {
      sessionToken: "f".repeat(64),
      query: { limit: 1 },
      input: { createdBy: "forged-user" },
    });
  });
  const request = await page.evaluate(() => {
    const calls = (window as unknown as { __phase7bCalls: Array<{ command: string; args: Record<string, unknown> }> }).__phase7bCalls;
    return calls.findLast((call) => call.command === "list_products")?.args;
  });
  expect(request?.sessionToken).toBe("c".repeat(64));
  expect((request?.input as { createdBy: string }).createdBy).toBe("auth-user");
});

for (const viewport of [
  { width: 1366, height: 768 },
  { width: 1440, height: 900 },
  { width: 1920, height: 1080 },
]) {
  test(`permission-denied routes fit ${viewport.width}x${viewport.height}`, async ({ page }) => {
    await page.setViewportSize(viewport);
    await installAuthorizationDouble(page, "CASHIER");
    await page.goto("/finance");
    await login(page);
    await expect(page.getByRole("heading", { name: "Access denied" })).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  });
}
