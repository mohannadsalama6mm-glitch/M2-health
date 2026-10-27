import { expect, test } from "@playwright/test";
import { installAuthDouble } from "./auth-double";

test("first launch starts the seven-day trial only after explicit activation", async ({ page }) => {
  await installAuthDouble(page, false, "UNACTIVATED");
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Start M² Health" })).toBeVisible();
  await expect(page.getByText("device-early-access")).toBeVisible();
  await page.getByRole("button", { name: "Start 7-day free trial" }).click();
  await expect(page.getByRole("heading", { name: "Create local administrator" })).toBeVisible();
});

test("manual activation rejects invalid codes and accepts a verified envelope", async ({ page }) => {
  await installAuthDouble(page, false, "UNACTIVATED");
  await page.goto("/");
  await page.getByLabel("Manual activation code").fill("invalid-code");
  await page.getByRole("button", { name: "Activate this device" }).click();
  await expect(page.getByText("The activation code signature could not be verified.")).toBeVisible();
  await page.getByLabel("Manual activation code").fill("valid-manual-license");
  await page.getByRole("button", { name: "Activate this device" }).click();
  await expect(page.getByRole("heading", { name: "Create local administrator" })).toBeVisible();
});

test("expired installations keep data accessible through an explicit read-only path", async ({ page }) => {
  await installAuthDouble(page, true, "EXPIRED");
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Activation required" })).toBeVisible();
  await expect(page.getByText("Your pharmacy data is safe")).toBeVisible();
  await page.getByRole("button", { name: "Open read-only workspace" }).click();
  await expect(page.getByRole("heading", { name: "Welcome back" })).toBeVisible();
});
