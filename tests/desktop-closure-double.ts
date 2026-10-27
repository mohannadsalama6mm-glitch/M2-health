import type { Page } from "@playwright/test";

export function installDesktopClosureDouble(page: Page) {
  return page.addInitScript(() => {
    const host = window as unknown as {
      __TAURI_INTERNALS__: {
        invoke: (
          command: string,
          args?: Record<string, unknown>,
        ) => Promise<unknown>;
      };
      desktopClosureCalls: { command: string; args: Record<string, unknown> }[];
    };
    const original = host.__TAURI_INTERNALS__.invoke;
    host.desktopClosureCalls = [];
    const settings = {
      pharmacyName: "Main Pharmacy",
      pharmacyPhone: "01000000000",
      pharmacyAddress: "Cairo",
      registrationNumber: "REG-1",
      timezone: "Africa/Cairo",
      receiptHeader: "Main Pharmacy",
      receiptFooter: "Thank you",
      updatedAt: "2026-10-19T10:00:00Z",
    };
    const backups: Array<Record<string, unknown>> = [];
    const alerts = {
      branchId: "branch-main",
      expiryWarningDays: 30,
      criticalExpiryDays: 7,
      expiryEnabled: true,
      lowStockEnabled: true,
    };
    host.__TAURI_INTERNALS__.invoke = async (command, args = {}) => {
      host.desktopClosureCalls.push({ command, args });
      if (command === "recovery_status")
        return {
          authEnabled: true,
          laterWorkflowsEnabled: true,
          inventoryWorkflowsEnabled: true,
          stockCountsEnabled: true,
          inventoryAlertsEnabled: true,
          inventoryUiEnabled: true,
          salesEnabled: true,
          partnersEnabled: true,
          purchasingEnabled: true,
          reportingEnabled: true,
          financeEnabled: true,
          dashboardEnabled: true,
        };
      if (command === "get_local_settings") return settings;
      if (command === "update_local_settings") {
        Object.assign(settings, args.input);
        return settings;
      }
      if (command === "get_inventory_alert_settings") return alerts;
      if (command === "update_inventory_alert_settings") {
        Object.assign(alerts, args.input);
        return alerts;
      }
      if (command === "get_cloud_foundation_status")
        return {
          mode: "LOCAL_ONLY",
          configured: false,
          endpointConfigured: false,
          deviceId: "local-test-device",
          deviceStatus: "LOCAL_ONLY",
          organizationId: null,
          organizationName: null,
          cloudIdentityLinked: false,
          cloudSubjectId: null,
          transportEnabled: false,
          syncAvailable: false,
          setupRequired: false,
          outboxPending: 0,
          inboxApplied: 0,
          inboxRejected: 0,
          openConflicts: 0,
          lastSyncAt: null,
          lastSuccessfulSyncAt: null,
          lastSyncOutcome: null,
          lastAcknowledgedAt: null,
          retryScheduled: 0,
          nextRetryAt: null,
        };
      if (command === "list_inventory_alerts")
        return {
          items: [
            {
              id: "alert-1",
              branchId: "branch-main",
              alertType: "LOW_STOCK",
              productPackageId: "package-1",
              batchId: null,
              severity: "HIGH",
              status: "ACTIVE",
              fingerprint: "alert-1",
              details: {},
              firstDetectedAt: "2026-10-19T10:00:00Z",
              lastDetectedAt: "2026-10-19T10:00:00Z",
              resolvedAt: null,
              updatedAt: "2026-10-19T10:00:00Z",
              productName: "Paracetamol",
              packageLabel: "20 tablets",
            },
          ],
          total: 1,
          businessDate: "2026-10-19",
        };
      if (command === "list_security_audit_events")
        return {
          total: 1,
          items: [
            {
              id: "audit-1",
              actorUserId: "admin-id",
              actorName: "Local Administrator",
              subjectUserId: "user-1",
              subjectName: "Mona Pharmacist",
              eventType: "USER_CREATED",
              detail: "Local user created.",
              createdAt: "2026-10-19T10:00:00Z",
            },
          ],
        };
      if (command === "get_local_backup_status")
        return { managedLocation: "C:\\M2Health\\backups", backups };
      if (command === "create_local_backup") {
        const backup = {
          id: "m2-health-backup-20261019T100000Z-test.db",
          createdAt: "20261019T100000Z",
          sizeBytes: 2048,
          schemaVersion: 22,
          integrity: "ok",
        };
        backups.unshift(backup);
        return backup;
      }
      if (command === "validate_local_backup")
        return backups.find((row) => row.id === args.id);
      if (command === "restore_local_backup")
        return {
          safetyBackupId: "m2-health-safety-20261019T100001Z-test.db",
          restartRequired: true,
        };
      return original(command, args);
    };
  });
}
