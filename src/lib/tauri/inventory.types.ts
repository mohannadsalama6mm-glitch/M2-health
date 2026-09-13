// Phase 3 DTO contracts only. Mirrors src-tauri/src/db/models/inventory.rs.
// Money is integer EGP minor units; timestamps are UTC ISO-8601.
export interface InventoryBatch {
  id: string;
  branchId: string;
  productPackageId: string;
  batchNumber: string;
  expiryDate: string;
  costPriceMinor: number | null;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}
export interface OpeningStockInput {
  branchId: string;
  productPackageId: string;
  batchNumber: string;
  expiryDate?: string;
  costPriceMinor?: number | null;
  quantity: number;
  reason?: string;
  user?: string | null;
}
export interface AdjustStockInput {
  branchId: string;
  productPackageId: string;
  batchId?: string | null;
  batchNumber?: string | null;
  expiryDate?: string | null;
  newQuantity: number;
  reason: string;
  user?: string | null;
}
export interface WriteOffInput {
  batchId: string;
  kind: "damage" | "expired" | "supplier_return";
  quantity: number;
  reason: string;
  user?: string | null;
}
export interface TransferInput {
  fromBranchId: string;
  toBranchId: string;
  productPackageId: string;
  batchId: string;
  quantity: number;
  reason?: string;
  user?: string | null;
}
export interface SetReorderInput {
  branchId: string;
  productPackageId: string;
  reorderLevel: number;
}
export interface CreateCountInput {
  branchId: string;
  scope?: string;
  categoryId?: string | null;
}
export interface SaveCountItemInput {
  stockCountId: string;
  batchId: string;
  countedQuantity: number;
}
export interface CompleteCountInput {
  stockCountId: string;
}
export interface StockMovement {
  id: string;
  branchId: string;
  productPackageId: string;
  batchId: string;
  movementType: string;
  quantityDelta: number;
  costPriceMinor: number | null;
  referenceType: string | null;
  referenceId: string | null;
  reason: string;
  user: string;
  occurredAt: string;
  createdAt: string;
}
export interface StockMovementItem {
  id: string;
  occurredAt: string;
  movementType: string;
  product: string;
  pack: string | null;
  batch: string | null;
  branch: string;
  incoming: number | null;
  outgoing: number | null;
  user: string;
  reason: string;
}
export interface StockMovementPage {
  items: StockMovementItem[];
  total: number;
  limit: number;
  offset: number;
}
export interface MovementQuery {
  branchId?: string | null;
  packageId?: string | null;
  batchId?: string | null;
  movementType?: string | null;
  search?: string | null;
  sort?: string | null;
  sortDirection?: string | null;
  limit?: number | null;
  offset?: number | null;
}
export interface BatchMini {
  batchId: string;
  batchNumber: string;
  expiryDate: string;
  quantity: number;
}
export interface StockRow {
  packageId: string;
  productId: string;
  nameEn: string | null;
  nameAr: string | null;
  scientificName: string | null;
  manufacturerName: string | null;
  packageLabel: string;
  packSize: string | null;
  quantity: number;
  reorderLevel: number;
  batchCount: number;
  batches: BatchMini[];
  minExpiryDays: number | null;
  sellingPriceMinor: number | null;
  costPriceMinor: number | null;
  status: string;
}
export interface StockOverviewQuery {
  branchId: string;
  search?: string | null;
  manufacturerId?: string | null;
  categoryId?: string | null;
  status?: string | null;
  sort?: string | null;
  sortDirection?: string | null;
  limit?: number | null;
  offset?: number | null;
}
export interface StockOverviewPage {
  items: StockRow[];
  total: number;
  limit: number;
  offset: number;
}
export interface StockSummary {
  inStock: number;
  low: number;
  outOfStock: number;
  valueMinor: number;
}
export interface ExpiryRow {
  batchId: string;
  packageId: string;
  productId: string;
  nameEn: string | null;
  nameAr: string | null;
  scientificName: string | null;
  packageLabel: string;
  batchNumber: string;
  expiryDate: string;
  days: number;
  quantity: number;
  valueMinor: number;
  costPriceMinor: number | null;
  branchName: string;
}
export interface ExpiryQuery {
  branchId: string;
  windowDays?: number | null;
  search?: string | null;
  limit?: number | null;
  offset?: number | null;
}
export interface ExpiryPage {
  items: ExpiryRow[];
  total: number;
  limit: number;
  offset: number;
}
export interface LowStockRow {
  packageId: string;
  productId: string;
  nameEn: string | null;
  nameAr: string | null;
  packageLabel: string;
  quantity: number;
  reorderLevel: number;
  status: string;
}
export interface LowStockPage {
  items: LowStockRow[];
  total: number;
  limit: number;
  offset: number;
}
export interface LowStockQuery {
  branchId: string;
  search?: string | null;
  limit?: number | null;
  offset?: number | null;
}
export interface FefoAllocation {
  batchId: string;
  batchNumber: string;
  expiryDate: string;
  available: number;
  take: number;
}
export interface StockCount {
  id: string;
  branchId: string;
  scope: string;
  categoryId: string | null;
  status: string;
  startedAt: string;
  completedAt: string | null;
  createdAt: string;
  updatedAt: string;
}
export interface StockCountItem {
  id: string;
  stockCountId: string;
  productPackageId: string;
  batchId: string;
  systemQuantity: number;
  countedQuantity: number;
  variance: number;
}
export interface StockCountItemView {
  id: string;
  productPackageId: string;
  batchId: string;
  product: string;
  pack: string | null;
  batchNumber: string;
  systemQuantity: number;
  countedQuantity: number;
  variance: number;
}
export interface StockCountDetail {
  count: StockCount;
  branchName: string;
  items: StockCountItemView[];
}
export interface StockCountRow {
  id: string;
  branchName: string;
  scope: string;
  categoryName: string | null;
  status: string;
  startedAt: string;
  completedAt: string | null;
  itemCount: number;
  discrepancyCount: number;
}
export interface StockCountPage {
  items: StockCountRow[];
  total: number;
  limit: number;
  offset: number;
}
export interface CountQuery {
  branchId?: string | null;
  status?: string | null;
  search?: string | null;
  limit?: number | null;
  offset?: number | null;
}
export interface InventorySetting {
  branchId: string;
  productPackageId: string;
  reorderLevel: number;
  createdAt: string;
  updatedAt: string;
}
export interface AdjustResult {
  newQuantity: number;
  movements: StockMovement[];
}
export interface TransferResult {
  out: StockMovement;
  incoming: StockMovement;
  fromBatchId: string;
  toBatchId: string;
}
export const MOVEMENT_TYPES = [
  "opening",
  "purchase",
  "sale",
  "customer_return",
  "supplier_return",
  "adjustment",
  "damage",
  "expired",
  "transfer_out",
  "transfer_in",
  "count_correction",
] as const;
export type MovementType = (typeof MOVEMENT_TYPES)[number];