import { nativeRequest } from "./client";
export type AlertType =
  "BATCH_EXPIRED" | "BATCH_EXPIRING" | "LOW_STOCK" | "OUT_OF_STOCK";
export type Severity = "CRITICAL" | "HIGH" | "MEDIUM";
export type AlertStatus = "ACTIVE" | "RESOLVED" | "DISMISSED";
export interface AlertSettings {
  branchId: string;
  expiryWarningDays: number;
  criticalExpiryDays: number;
  expiryEnabled: boolean;
  lowStockEnabled: boolean;
}
export interface InventoryAlert {
  id: string;
  branchId: string;
  alertType: AlertType;
  productPackageId: string;
  batchId: string | null;
  severity: Severity;
  status: AlertStatus;
  fingerprint: string;
  details: Record<string, unknown>;
  firstDetectedAt: string;
  lastDetectedAt: string;
  resolvedAt: string | null;
  updatedAt: string;
  productName: string | null;
  productId?: string;
  packageLabel: string;
}
export interface Monitor {
  branchId: string;
  productPackageId: string;
  batchId: string | null;
  productName: string | null;
  productId?: string;
  packageLabel: string;
  lotNumber: string | null;
  expiryDate: string | null;
  daysRemaining: number | null;
  quantity: number;
  costPriceMinor: number | null;
  reorderLevel: number | null;
  minimumStock: number | null;
  maximumStock: number | null;
  status: AlertStatus | null;
  severity: Severity | null;
}
export interface Page<T> {
  items: T[];
  total: number;
  businessDate: string;
}
export interface AlertQuery {
  branchId: string;
  alertType: AlertType | null;
  severity: Severity | null;
  status: AlertStatus | null;
  fromDate: string | null;
  toDate: string | null;
  limit: number;
  offset: number;
}
export interface MonitorQuery {
  branchId: string;
  minimumDays: number;
  maximumDays: number;
  expired: boolean;
  outOfStock: boolean;
  limit: number;
  offset: number;
}
export interface FefoRow {
  batch: {
    id: string;
    lotNumber: string | null;
    expiryDate: string | null;
    receivedAt: string | null;
    costPriceMinor: number | null;
  };
  quantity: number;
}
export interface AlertDetail {
  alert: InventoryAlert;
  history: {
    status: AlertStatus;
    severity: Severity;
    details: Record<string, unknown>;
    occurredAt: string;
  }[];
}
export const getAlertSettings = (branchId: string) =>
  nativeRequest<AlertSettings>("get_inventory_alert_settings", { branchId });
export const saveAlertSettings = (input: AlertSettings) =>
  nativeRequest<AlertSettings>("update_inventory_alert_settings", { input });
export const evaluateAlerts = (branchId: string) =>
  nativeRequest<{ businessDate: string; detected: number; resolved: number }>(
    "evaluate_inventory_alerts",
    { branchId },
  );
export const listAlerts = (query: AlertQuery) =>
  nativeRequest<Page<InventoryAlert>>("list_inventory_alerts", { query });
export const getAlert = (branchId: string, id: string) =>
  nativeRequest<AlertDetail>("get_inventory_alert", { branchId, id });
export const dismissAlert = (branchId: string, id: string) =>
  nativeRequest<void>("dismiss_inventory_alert", { branchId, id });
export const listMonitor = (query: MonitorQuery, low: boolean) =>
  nativeRequest<Page<Monitor>>(
    low ? "list_low_stock_inventory" : "list_expiry_inventory",
    { query },
  );
export const inspectFefo = (
  branchId: string,
  productPackageId: string,
  offset: number,
) =>
  nativeRequest<FefoRow[]>("get_fefo_inventory_batches", {
    branchId,
    productPackageId,
    offset,
    limit: 20,
  });
