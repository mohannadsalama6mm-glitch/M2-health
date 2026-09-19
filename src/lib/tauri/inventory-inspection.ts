import { nativeRequest } from "./client";
export interface InspectionQuery {
  branchId: string;
  productPackageId?: string | null;
  batchId?: string | null;
  search?: string;
  stockState?: string | null;
  manufacturerId?: string | null;
  categoryId?: string | null;
  routeId?: string | null;
  movementType?: string | null;
  dateFrom?: string | null;
  dateTo?: string | null;
  documentReference?: string;
  limit: number;
  offset: number;
}
export interface InventoryRow {
  productPackageId: string;
  productId: string;
  productName: string;
  packageLabel: string;
  branchId: string;
  quantity: number;
  batchCount: number;
  nearestExpiry: string | null;
  reorderLevel: number | null;
  minimumStock: number | null;
  maximumStock: number | null;
  stockState: string;
  packageActive: number;
  productActive: number;
}
export interface BatchRow {
  id: string;
  branchId: string;
  productPackageId: string;
  productId: string;
  productName: string;
  packageLabel: string;
  lotNumber: string | null;
  expiryDate: string | null;
  receivedAt: string | null;
  costPriceMinor: number | null;
  isActive: number;
  quantity: number;
}
export interface MovementRow {
  id: string;
  branchId: string;
  productPackageId: string;
  productName: string;
  packageLabel: string;
  batchId: string;
  movementType: string;
  quantityDelta: number;
  createdAt: string;
  reason: string;
  createdBy: string | null;
  referenceType: string | null;
  referenceId: string | null;
  documentId: string | null;
  documentNumber: string | null;
  countSessionId: string | null;
  countNumber: string | null;
}
export interface InspectionPage<T> {
  items: T[];
  total: number;
  businessDate: string;
}
export const overview = (query: InspectionQuery) =>
  nativeRequest<InspectionPage<InventoryRow>>("inspect_inventory_overview", {
    query,
  });
export const batches = (query: InspectionQuery) =>
  nativeRequest<InspectionPage<BatchRow>>("inspect_inventory_batches", {
    query,
  });
export const movements = (query: InspectionQuery) =>
  nativeRequest<InspectionPage<MovementRow>>("inspect_inventory_movements", {
    query,
  });
export interface OpeningInput {
  requestId: string;
  batch: {
    branchId: string;
    productPackageId: string;
    lotNumber: string | null;
    expiryDate: string | null;
    costPriceMinor: number | null;
  };
  quantity: number;
  reason: string;
  createdBy: null;
}
export const opening = (input: OpeningInput) =>
  nativeRequest<{ id: string; batchId: string; quantityDelta: number }>(
    "record_opening_stock",
    { input },
  );
export const saveLevel = (input: {
  branchId: string;
  productPackageId: string;
  reorderLevel: number;
  minimumStock: number | null;
  maximumStock: number | null;
}) => nativeRequest("set_inventory_level", { input });
