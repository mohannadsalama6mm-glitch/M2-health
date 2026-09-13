import { nativeRequest } from "./client";
export type MovementType =
  | "OPENING"
  | "PURCHASE_RECEIPT"
  | "SALE"
  | "SALE_RETURN"
  | "PURCHASE_RETURN"
  | "ADJUSTMENT_IN"
  | "ADJUSTMENT_OUT"
  | "DAMAGE"
  | "EXPIRY"
  | "TRANSFER_OUT"
  | "TRANSFER_IN"
  | "STOCK_COUNT_CORRECTION";
export interface CreateInventoryBatch {
  branchId: string;
  productPackageId: string;
  lotNumber?: string | null;
  expiryDate?: string | null;
  receivedAt?: string | null;
  costPriceMinor?: number | null;
  supplierReference?: string | null;
}
export interface InventoryBatch extends Required<CreateInventoryBatch> {
  id: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}
export interface StockOperation {
  requestId: string;
  branchId: string;
  productPackageId: string;
  batchId: string;
  quantity: number;
  reason: string;
  createdBy?: string | null;
}
export interface OpeningStock {
  batch: CreateInventoryBatch;
  requestId: string;
  quantity: number;
  reason: string;
  createdBy?: string | null;
}
export interface Adjustment {
  operation: StockOperation;
  direction: "in" | "out";
}
export interface InventoryMovement {
  id: string;
  branchId: string;
  productPackageId: string;
  batchId: string;
  movementType: MovementType;
  quantityDelta: number;
  referenceType: string | null;
  referenceId: string | null;
  reason: string;
  createdBy: string | null;
  transferId: string | null;
  createdAt: string;
}
export interface InventoryBalance {
  branchId: string;
  productPackageId: string;
  batchId: string | null;
  quantity: number;
}
export interface InventoryQuery {
  branchId: string;
  productPackageId?: string | null;
  batchId?: string | null;
  limit?: number;
  offset?: number;
}
export interface SetInventoryLevel {
  branchId: string;
  productPackageId: string;
  reorderLevel: number;
  minimumStock?: number | null;
  maximumStock?: number | null;
}
export interface InventoryLevel extends Required<SetInventoryLevel> {
  id: string;
  createdAt: string;
  updatedAt: string;
}
export interface LowStockItem {
  branchId: string;
  productPackageId: string;
  quantity: number;
  reorderLevel: number;
}
export interface ExpiryItem {
  batch: InventoryBatch;
  quantity: number;
}
export interface ExpiryQuery {
  branchId: string;
  asOf: string;
  withinDays: number;
  expiredOnly: boolean;
  limit?: number;
  offset?: number;
}
export interface TransferStock {
  requestId: string;
  sourceBranchId: string;
  destinationBranchId: string;
  productPackageId: string;
  sourceBatchId: string;
  quantity: number;
  reason: string;
  createdBy?: string | null;
}
export interface TransferResult {
  id: string;
  sourceBatchId: string;
  destinationBatchId: string;
  outMovement: InventoryMovement;
  inMovement: InventoryMovement;
}
export const createInventoryBatch = (input: CreateInventoryBatch) =>
  nativeRequest<InventoryBatch>("create_inventory_batch", { input });
export const recordOpeningStock = (input: OpeningStock) =>
  nativeRequest<InventoryMovement>("record_opening_stock", { input });
export const recordBatchOpeningStock = (input: StockOperation) =>
  nativeRequest<InventoryMovement>("record_batch_opening_stock", { input });
export const recordAdjustment = (input: Adjustment) =>
  nativeRequest<InventoryMovement>("record_adjustment", { input });
export const recordDamage = (input: StockOperation) =>
  nativeRequest<InventoryMovement>("record_damage", { input });
export const recordExpiryWriteOff = (input: StockOperation) =>
  nativeRequest<InventoryMovement>("record_expiry_write_off", { input });
export const transferInventoryStock = (input: TransferStock) =>
  nativeRequest<TransferResult>("transfer_inventory_stock", { input });
export const getPackageStock = (branchId: string, productPackageId: string) =>
  nativeRequest<InventoryBalance>("get_package_stock", {
    branchId,
    productPackageId,
  });
export const getBatchStock = (branchId: string, batchId: string) =>
  nativeRequest<InventoryBalance>("get_batch_stock", { branchId, batchId });
export const listBranchInventory = (query: InventoryQuery) =>
  nativeRequest<InventoryBalance[]>("list_branch_inventory", { query });
export const listStockMovements = (query: InventoryQuery) =>
  nativeRequest<InventoryMovement[]>("list_stock_movements", { query });
export const setInventoryLevel = (input: SetInventoryLevel) =>
  nativeRequest<InventoryLevel>("set_inventory_level", { input });
export const listLowStock = (query: InventoryQuery) =>
  nativeRequest<LowStockItem[]>("list_low_stock", { query });
export const listExpiringBatches = (query: ExpiryQuery) =>
  nativeRequest<ExpiryItem[]>("list_expiring_batches", { query });
export const getFefoBatches = (
  branchId: string,
  productPackageId: string,
  asOf: string,
  limit = 50,
  offset = 0,
) =>
  nativeRequest<ExpiryItem[]>("get_fefo_batches", {
    branchId,
    productPackageId,
    asOf,
    limit,
    offset,
  });
