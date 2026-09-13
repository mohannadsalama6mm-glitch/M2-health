import { expect, test, type Page } from "@playwright/test";
import { installCatalogDouble } from "./catalog-double";

test("large catalog pagination keeps bounded controls and requests the selected offset", async ({
  page,
}) => {
  await installCatalogDouble(page);
  await page.goto("/dashboard");
  await page.evaluate(() => {
    const host = window as unknown as {
      __TAURI_INTERNALS__: {
        invoke: (
          command: string,
          args?: Record<string, unknown>,
        ) => Promise<Record<string, unknown>>;
      };
      __offset: number;
    };
    const invoke = host.__TAURI_INTERNALS__.invoke;
    host.__TAURI_INTERNALS__.invoke = async (command, args) => {
      const result = await invoke(command, args);
      if (command === "list_products") {
        host.__offset = (args?.query as { offset: number }).offset;
        return { ...result, total: 25061, activeTotal: 25061 };
      }
      return result;
    };
  });
  await page.getByRole("link", { name: "Products", exact: true }).click();
  const nav = page.getByRole("navigation", { name: "Pagination", exact: true });
  await expect(
    nav.getByRole("button", { name: "2507", exact: true }),
  ).toBeVisible();
  expect(await nav.getByRole("button").count()).toBeLessThanOrEqual(9);
  await nav.getByRole("button", { name: "2507", exact: true }).click();
  await expect(
    nav.getByRole("button", { name: "Next page", exact: true }),
  ).toBeDisabled();
  await expect
    .poll(() =>
      page.evaluate(() => (window as unknown as { __offset: number }).__offset),
    )
    .toBe(25060);
  await nav.getByRole("button", { name: "Previous page", exact: true }).click();
  await expect
    .poll(() =>
      page.evaluate(() => (window as unknown as { __offset: number }).__offset),
    )
    .toBe(25050);
});
async function setup(page: Page, mode = "success") {
  await page.addInitScript((mode) => {
    const profile = {
      path: "canonical/source.csv",
      hash: "abc123",
      size: 1000,
      totalRows: 20,
      columns: [],
      profiledAt: "1789228800000",
    };
    const report = {
      id: "plan",
      mode: "dry_run",
      status: "validated",
      profile,
      counts: {
        ready: 17,
        exact_duplicate_source: 1,
        possible_duplicate: 1,
        ambiguous_price: 1,
        review_required: 2,
      },
      lookupCreated: { manufacturers: 2, categories: 1, routes: 1 },
      lookupReused: {},
      imported: 0,
      processed: 0,
      durationMs: 0,
      rowsPerSecond: 0,
      reportPath: "reports/plan.json",
      backupPath: null,
      error: null,
      rows: [
        {
          number: 4,
          fields: ["Review product"],
          disposition: "ambiguous_price",
          warnings: [],
          fingerprint: "a",
          identity: "b",
          priceMinor: null,
        },
      ],
    };
    let applying = false;
    let polls = 0;
    const branch = {
      id: "main",
      code: "MAIN",
      name: "Main Pharmacy",
      isActive: true,
    };
    Object.assign(window, {
      isTauri: true,
      __importCalls: 0,
      __TAURI_INTERNALS__: {
        invoke: async (command: string) => {
          if (command === "ensure_default_branch") return branch;
          if (command === "list_branches") return [branch];
          if (command === "profile_catalog_source") return profile;
          if (command === "get_catalog_import_report") return null;
          if (command === "dry_run_catalog_import") {
            if (mode === "dry-error")
              throw {
                code: "validation",
                message: "CSV columns do not match.",
              };
            return report;
          }
          if (command === "apply_catalog_import") {
            (window as unknown as { __importCalls: number }).__importCalls++;
            if (mode === "apply-error")
              throw {
                code: "validation",
                message: "Source changed. Run a fresh dry run.",
              };
            applying = true;
            return { ...report, mode: "apply", status: "applying" };
          }
          if (command === "get_catalog_import_status") {
            polls++;
            return {
              ...report,
              mode: "apply",
              status: applying && polls < 3 ? "applying" : "complete",
              imported: polls < 3 ? 8 : 17,
              processed: polls < 3 ? 8 : 20,
              backupPath: "backup.sqlite3",
              durationMs: 2000,
            };
          }
          throw new Error(command);
        },
      },
    });
  }, mode);
  await page.goto("/catalog/import");
}
test("profile, dry run counts, confirmation, progress, final report and no double import", async ({
  page,
}) => {
  await setup(page);
  await expect(
    page.getByText("20 source rows", { exact: false }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Apply Import", exact: true }),
  ).toBeDisabled();
  await page.getByRole("button", { name: "Dry Run", exact: true }).click();
  await expect(
    page.getByText("Review required", { exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Apply Import", exact: true }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
  expect(
    await page.evaluate(
      () => (window as unknown as { __importCalls: number }).__importCalls,
    ),
  ).toBe(0);
  await page.getByRole("button", { name: "Cancel", exact: true }).click();
  await page.getByRole("button", { name: "Apply Import", exact: true }).click();
  await page.getByRole("button", { name: "Confirm", exact: true }).click();
  await expect(
    page.getByRole("progressbar", { name: "Import progress" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Apply Import", exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByText("Import complete", { exact: true }),
  ).toBeVisible();
  await expect(page.getByText("Backup: backup.sqlite3")).toBeVisible();
  expect(
    await page.evaluate(
      () => (window as unknown as { __importCalls: number }).__importCalls,
    ),
  ).toBe(1);
});
test("dry run error never enables apply", async ({ page }) => {
  await setup(page, "dry-error");
  await page.getByRole("button", { name: "Dry Run", exact: true }).click();
  await expect(page.getByText("CSV columns do not match.")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Apply Import", exact: true }),
  ).toBeDisabled();
});
test("apply error consumes the UI plan and requires a new dry run", async ({
  page,
}) => {
  await setup(page, "apply-error");
  await page.getByRole("button", { name: "Dry Run", exact: true }).click();
  await page.getByRole("button", { name: "Apply Import", exact: true }).click();
  await page.getByRole("button", { name: "Confirm", exact: true }).click();
  await expect(
    page.getByText("Source changed. Run a fresh dry run."),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Apply Import", exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Dry Run", exact: true }),
  ).toBeEnabled();
});
