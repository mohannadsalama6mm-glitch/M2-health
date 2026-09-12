import { expect, test, type Page } from "@playwright/test";
import { installCatalogDouble } from "./catalog-double";

// Browser-only IPC test double; never installed in production code or a real DB.
async function nativeBoundary(
  page: Page,
  mode: "success" | "loading" | "error",
) {
  await page.addInitScript((mode) => {
    const branch = {
      id: "424b25b8-3c75-4d7a-87cd-988397081194",
      code: "MAIN",
      name: "Main Pharmacy",
      phone: "",
      address: "",
      isActive: true,
      createdAt: "2026-09-12T00:00:00.000Z",
      updatedAt: "2026-09-12T00:00:00.000Z",
    };
    Object.assign(window, {
      isTauri: true,
      __TAURI_INTERNALS__: {
        invoke: async (command: string) => {
          if (mode === "loading") await new Promise(() => {});
          if (mode === "error")
            throw {
              code: "database",
              message: "Local database could not be opened.",
            };
          if (command === "ensure_default_branch") return branch;
          if (command === "list_branches") return [branch];
          throw new Error(`Unexpected native command: ${command}`);
        },
      },
    });
  }, mode);
}

test("desktop boundary supplies local branches and a real catalog", async ({
  page,
}) => {
  await installCatalogDouble(page);
  await page.goto("/dashboard");
  const selector = page.getByRole("combobox", { name: "Branch", exact: true });
  await expect(selector).toHaveValue("424b25b8-3c75-4d7a-87cd-988397081194");
  await expect(selector.locator("option:checked")).toHaveText("Main Pharmacy");
  await page.goto("/catalog");
  await expect(
    page.getByRole("searchbox", { name: "Search Products" }),
  ).toBeVisible();
  await expect(page.getByText("1 results · local SQLite")).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Panadol 500 mg Paracetamol · Haleon" }),
  ).toBeVisible();
  await page
    .getByRole("link", { name: "Panadol 500 mg Paracetamol · Haleon" })
    .click();
  await expect(
    page.getByRole("heading", { name: "Panadol 500 mg" }),
  ).toBeVisible();
  await expect(selector).toHaveValue("424b25b8-3c75-4d7a-87cd-988397081194");
});

test("branch loading does not block the application shell", async ({
  page,
}) => {
  await nativeBoundary(page, "loading");
  await page.goto("/dashboard");
  const selector = page.getByRole("combobox", { name: "Branch", exact: true });
  await expect(selector).toBeDisabled();
  await expect(selector).toContainText("Loading local branches");
  await expect(
    page.getByRole("link", { name: "Products", exact: true }),
  ).toBeVisible();
});

test("database errors stay local and the branch connection can be retried", async ({
  page,
}) => {
  await nativeBoundary(page, "error");
  await page.goto("/dashboard");
  await expect(page.getByRole("alert")).toHaveText(
    "Local database could not be opened.",
  );
  await expect(
    page.getByRole("combobox", { name: "Branch", exact: true }),
  ).toBeDisabled();
  await page.evaluate(() => {
    Object.assign(window, {
      __TAURI_INTERNALS__: {
        invoke: async (command: string) => {
          const branch = {
            id: "recovered-test-id",
            name: "Recovered Pharmacy",
            isActive: true,
          };
          return command === "list_branches" ? [branch] : branch;
        },
      },
    });
  });
  await page
    .getByRole("button", { name: "Retry local branch connection" })
    .click();
  await expect(
    page.getByRole("combobox", { name: "Branch", exact: true }),
  ).toHaveValue("recovered-test-id");
  await expect(page.getByRole("alert")).toHaveCount(0);
});

test("browser preview keeps the original demo branches", async ({ page }) => {
  await page.goto("/dashboard");
  const selector = page.getByRole("combobox", { name: "Branch", exact: true });
  await expect(selector).toHaveValue("Main branch");
  await expect(selector.locator("option")).toHaveCount(3);
});
