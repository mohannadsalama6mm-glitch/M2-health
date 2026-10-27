import { expect, test, type Page } from "@playwright/test";
import { installAuthDouble } from "./auth-double";
import { installDesktopClosureDouble } from "./desktop-closure-double";

async function signIn(page: Page) {
  await page.getByLabel("Username").fill("admin.local");
  await page.getByLabel("Password").fill("VeryStrong!Pass2026");
  await page.getByRole("button", { name: "Sign in" }).click();
}

async function calls(page: Page, command: string) {
  return page.evaluate(
    (name) =>
      (
        window as unknown as {
          desktopClosureCalls: {
            command: string;
            args: Record<string, unknown>;
          }[];
        }
      ).desktopClosureCalls.filter((item) => item.command === name),
    command,
  );
}

test.beforeEach(async ({ page }) => {
  await installAuthDouble(page, true);
  await installDesktopClosureDouble(page);
});

test("desktop settings persist through IPC and keep cloud transport disabled", async ({
  page,
}) => {
  await page.goto("/settings");
  await signIn(page);
  await expect(
    page.getByText("Cloud transport", { exact: true }),
  ).toBeVisible();
  await expect(page.getByText("Disabled by rollout gates")).toBeVisible();
  await page.getByLabel("Pharmacy name").fill("North Pharmacy");
  await page.getByRole("button", { name: "Save pharmacy settings" }).click();
  await expect(
    page.getByText("Local pharmacy settings were saved."),
  ).toBeVisible();
  expect(await calls(page, "update_local_settings")).toHaveLength(1);
});

test("desktop notifications and audit log read canonical IPC data", async ({
  page,
}) => {
  await page.goto("/notifications");
  await signIn(page);
  await expect(page.getByText("Paracetamol · 20 tablets")).toBeVisible();
  expect(await calls(page, "list_inventory_alerts")).not.toHaveLength(0);
  await page.goto("/audit-log");
  await signIn(page);
  await expect(
    page.getByRole("table", { name: "Security audit events" }),
  ).toContainText("USER_CREATED");
  await page.getByRole("button", { name: "Inspect" }).click();
  await expect(
    page.getByText("Persisted immutable security event"),
  ).toBeVisible();
  expect(await calls(page, "list_security_audit_events")).not.toHaveLength(0);
});

test("backup restore requires explicit confirmation and reports its safety backup", async ({
  page,
}) => {
  await page.goto("/backup");
  await signIn(page);
  await page.getByRole("button", { name: "Create verified backup" }).click();
  await expect(page.getByRole("button", { name: "Restore" })).toBeVisible();
  await page.getByRole("button", { name: "Restore" }).click();
  await page.getByLabel("Type RESTORE").fill("RESTORE");
  await page.getByLabel("Restore reason").fill("Test recovery");
  await page.getByRole("button", { name: "Validate and restore" }).click();
  await expect(page.getByText(/Safety backup:/)).toBeVisible();
  expect(await calls(page, "validate_local_backup")).toHaveLength(1);
  expect(await calls(page, "restore_local_backup")).toHaveLength(1);
});
