import { nativeRequest } from "./client";

export type PurchaseStatus =
  | "DRAFT"
  | "ORDERED"
  | "PARTIALLY_RECEIVED"
  | "RECEIVED"
  | "CANCELLED";

export interface PurchaseLineInput {
  productPackageId: string;
  orderedQuantity: number;
  unitCostMinor: number;
}
export interface PurchaseDraftFields {
  supplierId: string;
  supplierReference?: string | null;
  expectedDate?: string | null;
  notes?: string | null;
  discountMinor: number;
  taxMinor: number;
  lines: PurchaseLineInput[];
}
export interface PurchaseLine {
  id: string;
  productId: string;
  productPackageId: string;
  productNameSnapshot: string;
  packageLabelSnapshot: string;
  orderedQuantity: number;
  unitCostMinor: number;
  lineTotalMinor: number;
  position: number;
  receivedQuantity: number;
  remainingQuantity: number;
  returnedQuantity: number;
  remainingReturnableQuantity: number;
}
export interface PurchaseOrder {
  id: string;
  purchaseNumber: string;
  supplierId: string;
  supplierCode: string;
  supplierName: string;
  status: PurchaseStatus;
  revision: number;
  supplierReference: string | null;
  expectedDate: string | null;
  notes: string | null;
  subtotalMinor: number;
  discountMinor: number;
  taxMinor: number;
  totalMinor: number;
  createdAt: string;
  updatedAt: string;
  orderedAt: string | null;
  cancelledAt: string | null;
  cancellationReason: string | null;
  lines: PurchaseLine[];
  receipts: PurchaseReceiptSummary[];
  returns: PurchaseReturnSummary[];
}
export interface PurchaseRow {
  id: string;
  purchaseNumber: string;
  supplierId: string;
  supplierCode: string;
  supplierName: string;
  status: PurchaseStatus;
  revision: number;
  totalMinor: number;
  lineCount: number;
  createdAt: string;
  orderedAt: string | null;
}
export interface PurchasePackage {
  productId: string;
  productPackageId: string;
  productName: string;
  packageLabel: string;
  manufacturerName: string | null;
  barcode: string | null;
  currentCostMinor: number | null;
}
export interface PurchaseReceiptLineInput {
  purchaseOrderLineId: string;
  quantity: number;
  lotNumber?: string | null;
  expiryDate?: string | null;
  acquisitionCostMinor?: number | null;
}
export interface PurchaseReceiptLine {
  id: string;
  purchaseOrderLineId: string;
  productPackageId: string;
  productNameSnapshot: string;
  packageLabelSnapshot: string;
  quantity: number;
  lotNumber: string | null;
  expiryDate: string | null;
  acquisitionCostMinor: number;
  batchId: string;
  movementId: string;
  position: number;
  returnedQuantity: number;
  remainingReturnableQuantity: number;
  currentBatchQuantity: number;
}
export interface PurchaseReceiptSummary {
  id: string;
  receiptNumber: string;
  branchId: string;
  branchCode: string;
  branchName: string;
  totalQuantity: number;
  postedAt: string;
}
export interface PurchaseReceipt {
  id: string;
  receiptNumber: string;
  purchaseOrderId: string;
  purchaseNumber: string;
  branchId: string;
  branchCode: string;
  branchName: string;
  supplierId: string;
  supplierCode: string;
  supplierName: string;
  totalQuantity: number;
  createdBy: string | null;
  postedAt: string;
  createdAt: string;
  lines: PurchaseReceiptLine[];
  returns: PurchaseReturnSummary[];
}
export interface PurchaseReturnLineInput { receiptLineId: string; quantity: number; }
export interface PurchaseReturnLine {
  id: string; receiptLineId: string; purchaseOrderLineId: string; productPackageId: string;
  productNameSnapshot: string; packageLabelSnapshot: string; receivedQuantity: number;
  previouslyReturnedQuantity: number; remainingReturnableQuantity: number; quantity: number;
  lotNumber: string | null; expiryDate: string | null; acquisitionCostMinor: number;
  batchId: string; movementId: string; position: number;
}
export interface PurchaseReturnSummary {
  id: string; returnNumber: string; purchaseOrderId: string; purchaseNumber: string;
  receiptId: string; receiptNumber: string; branchId: string; branchCode: string;
  branchName: string; supplierId: string; supplierCode: string; supplierName: string;
  totalQuantity: number; reason: string; postedAt: string;
}
export interface PurchaseReturn {
  id: string; returnNumber: string; purchaseOrderId: string; purchaseNumber: string;
  receiptId: string; receiptNumber: string; branchId: string; branchCode: string;
  branchName: string; supplierId: string; supplierCode: string; supplierName: string;
  sourcePurchaseRevision: number; reason: string; totalQuantity: number;
  createdBy: string | null; postedAt: string; createdAt: string; lines: PurchaseReturnLine[];
}
export interface Page<T> {
  items: T[];
  total: number;
  limit: number;
  offset: number;
}
export interface PurchaseQuery {
  search?: string | null;
  supplierId?: string | null;
  status?: PurchaseStatus | null;
  dateFrom?: string | null;
  dateTo?: string | null;
  limit?: number;
  offset?: number;
}

export const canonicalPurchasing = {
  createDraft: (input: PurchaseDraftFields & { id: string; requestId: string }) =>
    nativeRequest<PurchaseOrder>("create_purchase_order_draft", { input }),
  updateDraft: (
    input: PurchaseDraftFields & { id: string; expectedRevision: number },
  ) => nativeRequest<PurchaseOrder>("update_purchase_order_draft", { input }),
  order: (input: {
    id: string;
    expectedRevision: number;
    requestId: string;
  }) => nativeRequest<PurchaseOrder>("order_purchase_order", { input }),
  cancel: (input: {
    id: string;
    expectedRevision: number;
    requestId: string;
    reason: string;
  }) => nativeRequest<PurchaseOrder>("cancel_purchase_order", { input }),
  get: (id: string) =>
    nativeRequest<PurchaseOrder>("get_purchase_order", { id }),
  list: (query: PurchaseQuery) =>
    nativeRequest<Page<PurchaseRow>>("list_purchase_orders", { query }),
  searchPackages: (query: {
    search?: string | null;
    limit?: number;
    offset?: number;
  }) =>
    nativeRequest<Page<PurchasePackage>>("search_purchase_packages", {
      query,
    }),
  receive: (input: {
    id: string;
    purchaseOrderId: string;
    expectedRevision: number;
    requestId: string;
    branchId: string;
    createdBy?: string | null;
    lines: PurchaseReceiptLineInput[];
  }) => nativeRequest<PurchaseReceipt>("receive_purchase_order", { input }),
  getReceipt: (id: string) =>
    nativeRequest<PurchaseReceipt>("get_purchase_receipt", { id }),
  listReceipts: (query: {
    purchaseOrderId?: string | null;
    supplierId?: string | null;
    branchId?: string | null;
    limit?: number;
    offset?: number;
  }) => nativeRequest<Page<PurchaseReceiptSummary>>("list_purchase_receipts", { query }),
  createReturn: (input: {
    id: string; purchaseOrderId: string; receiptId: string; expectedRevision: number;
    requestId: string; reason: string; createdBy?: string | null; lines: PurchaseReturnLineInput[];
  }) => nativeRequest<PurchaseReturn>("create_purchase_return", { input }),
  getReturn: (id: string) => nativeRequest<PurchaseReturn>("get_purchase_return", { id }),
  listReturns: (query: {
    purchaseOrderId?: string | null; supplierId?: string | null; receiptId?: string | null;
    branchId?: string | null; search?: string | null; dateFrom?: string | null; dateTo?: string | null;
    limit?: number; offset?: number;
  }) => nativeRequest<Page<PurchaseReturnSummary>>("list_purchase_returns", { query }),
};
