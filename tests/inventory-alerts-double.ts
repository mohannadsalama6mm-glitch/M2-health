import type { Page } from "@playwright/test";
import { installCatalogDouble } from "./catalog-double";
import type {
  AlertSettings,
  InventoryAlert,
  Monitor,
} from "../src/lib/tauri/inventory-alerts";
export async function installAlertsDouble(
  page: Page,
  options: { catalog?: boolean; preserveRecovery?: boolean } = {},
) {
  if (options.catalog !== false) await installCatalogDouble(page);
  await page.addInitScript((options) => {
    const w = window as unknown as {
      __TAURI_INTERNALS__: {
        invoke: (c: string, a?: Record<string, unknown>) => Promise<unknown>;
      };
      alertCalls: { command: string; args?: Record<string, unknown> }[];
      alertFailure: string;
      alertTotal: number;
      alertEmpty: boolean;
    };
    const original = w.__TAURI_INTERNALS__.invoke;
    w.alertCalls = [];
    w.alertFailure = "";
    w.alertTotal = 1;
    w.alertEmpty = false;
    let settings: AlertSettings = {
      branchId: "424b25b8-3c75-4d7a-87cd-988397081194",
      expiryWarningDays: 90,
      criticalExpiryDays: 30,
      expiryEnabled: true,
      lowStockEnabled: true,
    };
    const alert: InventoryAlert = {
      id: "alert-1",
      branchId: settings.branchId,
      alertType: "BATCH_EXPIRED",
      productPackageId: "pkg-1",
      batchId: "batch-1",
      severity: "CRITICAL",
      status: "ACTIVE",
      fingerprint: "stable-alert",
      details: { quantity: 2 },
      firstDetectedAt: "2026-09-14T00:00:00Z",
      lastDetectedAt: "2026-09-14T00:00:00Z",
      resolvedAt: null,
      updatedAt: "2026-09-14T00:00:00Z",
      productName: "DEV alert medicine",
      packageLabel: "Box",
    };
    const monitor: Monitor = {
      branchId: settings.branchId,
      productPackageId: "pkg-1",
      batchId: "batch-1",
      productName: "DEV alert medicine",
      packageLabel: "Box",
      lotNumber: "DEV EXPIRY",
      expiryDate: "2026-09-13",
      daysRemaining: -1,
      quantity: 2,
      costPriceMinor: 100,
      reorderLevel: 3,
      minimumStock: 1,
      maximumStock: 10,
      status: "ACTIVE",
      severity: "CRITICAL",
    };
    w.__TAURI_INTERNALS__.invoke = async (command, args) => {
      w.alertCalls.push({ command, args });
      if (command === "recovery_status" && !options.preserveRecovery)
        return {
          laterWorkflowsEnabled: false,
          inventoryWorkflowsEnabled: true,
          stockCountsEnabled: true,
          inventoryAlertsEnabled: true,
        };
      if (
        command.includes("inventory_alert") ||
        command === "list_expiry_inventory" ||
        command === "list_low_stock_inventory" ||
        command === "get_fefo_inventory_batches"
      ) {
        if (w.alertFailure) throw { code: "database", message: w.alertFailure };
        if (command === "get_inventory_alert_settings")
          return structuredClone(settings);
        if (command === "update_inventory_alert_settings") {
          settings = structuredClone(args!.input as AlertSettings);
          return settings;
        }
        if (command === "evaluate_inventory_alerts")
          return { businessDate: "2026-09-14", detected: 1, resolved: 0 };
        if (command === "list_inventory_alerts") {
          const q = args!.query as { status: string };
          return {
            items: w.alertEmpty
              ? []
              : [{ ...alert, status: q.status || alert.status }],
            total: w.alertEmpty ? 0 : w.alertTotal,
            businessDate: "2026-09-14",
          };
        }
        if (command === "dismiss_inventory_alert") {
          alert.status = "DISMISSED";
          return null;
        }
        if (command === "get_inventory_alert")
          return {
            alert,
            history: [
              {
                status: "ACTIVE",
                severity: "CRITICAL",
                details: { quantity: 2 },
                occurredAt: alert.firstDetectedAt,
              },
            ],
          };
        if (command === "get_fefo_inventory_batches")
          return w.alertEmpty
            ? []
            : [
                {
                  batch: {
                    id: "known",
                    lotNumber: "KNOWN LOT",
                    expiryDate: "2026-10-01",
                    receivedAt: "2026-09-01",
                    costPriceMinor: 100,
                  },
                  quantity: 2,
                },
                {
                  batch: {
                    id: "unknown",
                    lotNumber: "UNKNOWN LOT",
                    expiryDate: null,
                    receivedAt: null,
                    costPriceMinor: null,
                  },
                  quantity: 1,
                },
              ];
        const q = args!.query as { outOfStock: boolean };
        return {
          items: w.alertEmpty
            ? []
            : [{ ...monitor, quantity: q.outOfStock ? 0 : 2 }],
          total: w.alertEmpty ? 0 : w.alertTotal,
          businessDate: "2026-09-14",
        };
      }
      return original(command, args);
    };
  }, options);
}
