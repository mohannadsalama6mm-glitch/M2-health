// Phase 5 purchase contracts. Mirrors src-tauri/src/db/models/purchases.rs.
// Money is integer EGP minor units; timestamps are UTC ISO-8601.
export interface Purchase {
  id: string;
  branchId: string;
  supplierId: string | null;
  sequence: number;
  purchaseNumber: string;
  invoiceNumber: string;
  status: "completed" | "void";
  subtotalMinor: number;
  discountMinor: number;
  taxMinor: number;
  totalMinor: number;
  paidMinor: number;
  changeMinor: number;
  paymentMethod: string;
  user: string;
  note: string;
  voidReason: string;
  voidedAt: string | null;
  voidedBy: string | null;
  completedAt: string;
  createdAt: string;
  updatedAt: string;
}
export interface PurchaseLineInput {
  productPackageId: string;
  quantity: number;
  unitCostMinor: number;
}
export interface CompletePurchaseInput {
  branchId: string;
  supplierId?: string | null;
  invoiceNumber?: string;
  discountMinor?: number;
  taxMinor?: number;
  paidMinor: number;
  paymentMethod?: string;
  user?: string | null;
  note?: string;
  lines: PurchaseLineInput[];
}
export interface CompletePurchaseResult {
  purchase: Purchase;
  movements: import("./inventory.types").StockMovement[];
}
export interface PurchaseItem {
  id: string;
  purchaseId: string;
  productPackageId: string;
  productName: string;
  packageLabel: string;
  quantity: number;
  unitCostMinor: number;
  lineTotalMinor: number;
  position: number;
}
export interface PurchaseBatchAllocationView {
  id: string;
  batchId: string;
  quantity: number;
  unitCostMinor: number | null;
}
export interface PurchaseItemView {
  id: string;
  productPackageId: string;
  productName: string;
  packageLabel: string;
  quantity: number;
  unitCostMinor: number;
  lineTotalMinor: number;
  position: number;
  batches: PurchaseBatchAllocationView[];
}
export interface PurchasePayment {
  id: string;
  purchaseId: string;
  method: string;
  amountMinor: number;
  createdAt: string;
}
export interface VoidPurchaseInput {
  purchaseId: string;
  reason: string;
  user?: string | null;
}
export interface PurchaseDetail {
  purchase: Purchase;
  branchName: string;
  supplierName: string | null;
  items: PurchaseItemView[];
}
export interface PurchaseRow {
  id: string;
  branchId: string;
  branchName: string;
  supplierName: string | null;
  sequence: number;
  purchaseNumber: string;
  invoiceNumber: string;
  status: "completed" | "void";
  totalMinor: number;
  paidMinor: number;
  paymentMethod: string;
  itemCount: number;
  user: string;
  completedAt: string;
  voidReason: string;
}
export interface PurchasePage {
  items: PurchaseRow[];
  total: number;
  limit: number;
  offset: number;
}
export interface PurchaseQuery {
  branchId?: string | null;
  status?: string | null;
  search?: string | null;
  limit?: number | null;
  offset?: number | null;
}
export interface PurchaseProduct {
  packageId: string;
  productId: string;
  productName: string;
  packageLabel: string;
  packSize: string | null;
  costPriceMinor: number | null;
  status: string;
}
export interface PurchaseProductPage {
  items: PurchaseProduct[];
  total: number;
  limit: number;
  offset: number;
}
export interface PosPurchaseQuery {
  branchId: string;
  search?: string | null;
  limit?: number | null;
  offset?: number | null;
}