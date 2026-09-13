// Phase 4/5 POS contracts only. Mirrors src-tauri/src/db/models/sales.rs.
// Money is integer EGP minor units; timestamps are UTC ISO-8601.
export interface Sale {
  id: string;
  branchId: string;
  sequence: number;
  receiptNumber: string;
  customerId: string | null;
  customerName: string;
  customerPhone: string;
  status: "completed" | "void";
  subtotalMinor: number;
  discountMinor: number;
  taxMinor: number;
  totalMinor: number;
  paidMinor: number;
  changeMinor: number;
  paymentMethod: "cash" | "card" | "other";
  user: string;
  note: string;
  voidReason: string;
  voidedAt: string | null;
  voidedBy: string | null;
  completedAt: string;
  createdAt: string;
  updatedAt: string;
}
export interface SaleLineInput {
  productPackageId: string;
  quantity: number;
}
export interface CompleteSaleInput {
  branchId: string;
  customerId?: string | null;
  customerName?: string;
  customerPhone?: string;
  discountMinor?: number;
  taxMinor?: number;
  paidMinor: number;
  paymentMethod?: string;
  user?: string | null;
  note?: string;
  lines: SaleLineInput[];
}
export interface CompleteSaleResult {
  sale: Sale;
  movements: import("./inventory.types").StockMovement[];
}
export interface SaleItem {
  id: string;
  saleId: string;
  productPackageId: string;
  productName: string;
  packageLabel: string;
  quantity: number;
  sellingPriceMinor: number;
  costPriceMinor: number | null;
  lineTotalMinor: number;
  lineCostMinor: number | null;
  position: number;
}
export interface SaleBatchAllocation {
  batchId: string;
  batchNumber: string;
  expiryDate: string;
  quantity: number;
  costPriceMinor: number | null;
}
export interface SaleItemView {
  id: string;
  productPackageId: string;
  productName: string;
  packageLabel: string;
  quantity: number;
  sellingPriceMinor: number;
  costPriceMinor: number | null;
  lineTotalMinor: number;
  lineCostMinor: number | null;
  position: number;
  batches: SaleBatchAllocation[];
  returnedQuantity: number;
  returnableQuantity: number;
}
export interface SalePayment {
  id: string;
  saleId: string;
  method: string;
  amountMinor: number;
  createdAt: string;
}
export interface ReturnItemInput {
  saleItemId: string;
  quantity: number;
  refundMinor: number;
}
export interface ReturnSaleInput {
  saleId: string;
  reason: string;
  user?: string | null;
  items: ReturnItemInput[];
}
export interface SaleReturnItemView {
  id: string;
  saleItemId: string;
  productName: string;
  packageLabel: string;
  quantity: number;
  refundMinor: number;
  lineRefundMinor: number;
}
export interface SaleReturn {
  id: string;
  saleId: string;
  branchId: string;
  reason: string;
  user: string;
  totalRefundMinor: number;
  returnedAt: string;
  items: SaleReturnItemView[];
}
export interface VoidSaleInput {
  saleId: string;
  reason: string;
  user?: string | null;
}
export interface SaleDetail {
  sale: Sale;
  branchName: string;
  items: SaleItemView[];
  returns: SaleReturn[];
}
export interface SaleRow {
  id: string;
  branchId: string;
  branchName: string;
  sequence: number;
  receiptNumber: string;
  customerName: string;
  status: "completed" | "void";
  totalMinor: number;
  paidMinor: number;
  paymentMethod: string;
  itemCount: number;
  user: string;
  completedAt: string;
  voidReason: string;
}
export interface SalePage {
  items: SaleRow[];
  total: number;
  limit: number;
  offset: number;
}
export interface SaleQuery {
  branchId?: string | null;
  status?: string | null;
  search?: string | null;
  limit?: number | null;
  offset?: number | null;
}
export interface PosProduct {
  packageId: string;
  productId: string;
  productName: string;
  packageLabel: string;
  packSize: string | null;
  sellingPriceMinor: number | null;
  quantity: number;
  status: "in_stock" | "out_of_stock";
}
export interface PosProductPage {
  items: PosProduct[];
  total: number;
  limit: number;
  offset: number;
}
export interface PosQuery {
  branchId: string;
  search?: string | null;
  limit?: number | null;
  offset?: number | null;
}