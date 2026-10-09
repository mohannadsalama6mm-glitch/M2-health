import { nativeRequest } from "./client";

export interface ReportFilter {
  dateFrom?: string | null;
  dateTo?: string | null;
  branchId?: string | null;
  search?: string | null;
  status?: string | null;
  customerId?: string | null;
  supplierId?: string | null;
  productId?: string | null;
  productPackageId?: string | null;
  movementType?: string | null;
  expiryDays?: number | null;
  limit?: number;
  offset?: number;
}
export interface ReportPage<T> { items: T[]; total: number; limit: number; offset: number; summary?: Record<string, number> }
export interface ReportingOverview {
  postedSalesCount: number; grossSalesMinor: number; discountMinor: number;
  taxMinor: number; refundedMinor: number; netSalesMinor: number;
  netUnitsSold: number; currentStockUnits: number; lowStockPackageCount: number;
  purchaseOrderCount: number; receivedQuantity: number; purchaseReturnQuantity: number;
}
export type ReportRow = Record<string, string | number | boolean | null>;
export interface SaleReportRow extends ReportRow { id:string; saleNumber:string; status:string; grossMinor:number; refundedMinor:number; netMinor:number }
export interface ProductSalesRow extends ReportRow { productId:string; productPackageId:string; productName:string; quantitySold:number; quantityReturned:number; netQuantity:number; netMinor:number }
export interface CustomerSalesRow extends ReportRow { customerId:string; customerCode:string; customerName:string; saleCount:number; grossMinor:number; refundedMinor:number; netMinor:number }
export interface StockReportRow extends ReportRow { branchId:string; productId:string; productPackageId:string; quantity:number; reorderLevel:number; stockState:string; acquisitionCostValuationMinor:number; unvaluedQuantity:number }
export interface MovementReportRow extends ReportRow { id:string; branchId:string; productPackageId:string; movementType:string; quantityDelta:number; createdAt:string }
export interface ExpiryReportRow extends ReportRow { batchId:string; branchId:string; productPackageId:string; quantity:number; expiryDate:string|null; expiryState:string; daysToExpiry:number|null }
export interface LowStockReportRow extends ReportRow { branchId:string; productPackageId:string; quantity:number; reorderLevel:number; stockState:string }
export interface PurchaseReportRow extends ReportRow { id:string; purchaseNumber:string; supplierId:string; status:string; orderedQuantity:number; receivedQuantity:number; returnedQuantity:number; orderedValueMinor:number }
export interface PurchaseDocumentReportRow extends ReportRow { id:string; documentNumber:string; purchaseOrderId:string; purchaseNumber:string; supplierId:string; quantity:number; acquisitionValueMinor:number; postedAt:string }
export interface SupplierReportRow extends ReportRow { supplierId:string; supplierCode:string; supplierName:string; orderCount:number; orderedValueMinor:number; receivedQuantity:number; receivedValueMinor:number; returnedQuantity:number; returnedValueMinor:number }
const call = <T>(command: string, filter: ReportFilter) =>
  nativeRequest<T>(command, { filter });
export const reporting = {
  overview: (filter: ReportFilter) => call<ReportingOverview>("get_reporting_overview", filter),
  sales: (filter: ReportFilter) => call<ReportPage<SaleReportRow>>("get_sales_report", filter),
  salesByProduct: (filter: ReportFilter) => call<ReportPage<ProductSalesRow>>("get_sales_by_product_report", filter),
  salesByCustomer: (filter: ReportFilter) => call<ReportPage<CustomerSalesRow>>("get_sales_by_customer_report", filter),
  stock: (filter: ReportFilter) => call<ReportPage<StockReportRow>>("get_inventory_stock_report", filter),
  movements: (filter: ReportFilter) => call<ReportPage<MovementReportRow>>("get_inventory_movement_report", filter),
  expiry: (filter: ReportFilter) => call<ReportPage<ExpiryReportRow>>("get_expiry_report", filter),
  lowStock: (filter: ReportFilter) => call<ReportPage<LowStockReportRow>>("get_low_stock_report", filter),
  purchases: (filter: ReportFilter) => call<ReportPage<PurchaseReportRow>>("get_purchasing_report", filter),
  receipts: (filter: ReportFilter) => call<ReportPage<PurchaseDocumentReportRow>>("get_purchase_receipt_report", filter),
  returns: (filter: ReportFilter) => call<ReportPage<PurchaseDocumentReportRow>>("get_purchase_return_report", filter),
  suppliers: (filter: ReportFilter) => call<ReportPage<SupplierReportRow>>("get_supplier_analytics", filter),
};
