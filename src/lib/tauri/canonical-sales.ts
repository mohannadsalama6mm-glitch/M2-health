import { nativeRequest } from "./client";
export const availability = (branchId: string, productPackageId: string) =>
  nativeRequest<{
    branchId: string;
    productPackageId: string;
    availableQuantity: number;
    currentPriceMinor: number | null;
    stockStatus: string;
  }>("get_pos_package_availability", { branchId, productPackageId });
export interface CartLine {
  displayName?: string | null;
  productPackageId: string;
  quantity: number;
  unitPriceMinor: number | null;
  discountMinor: number;
}
export interface Cart {
  branchId: string;
  customerId: string | null;
  customerName: string | null;
  notes: string | null;
  discountMinor: number;
  taxMinor: number;
  lines: CartLine[];
}
export interface Product {
  productPackageId: string;
  productId: string;
  productName: string;
  arabicName: string | null;
  scientificName: string | null;
  packageLabel: string;
  currentPriceMinor: number | null;
  availableQuantity: number;
  stockStatus: string;
  barcodeMatch: number;
}
export interface Allocation {
  id?: string;
  batchId: string;
  quantity: number;
  unitCostMinor: number | null;
  movementId?: string;
  lotNumber: string | null;
  expiryDate: string | null;
}
export interface Item {
  id: string;
  productPackageId: string;
  productNameSnapshot: string;
  packageLabelSnapshot: string;
  quantity: number;
  unitPriceMinor: number;
  discountMinor: number;
  saleDiscountMinor: number;
  taxMinor: number;
  lineTotalMinor: number;
  unitCostMinor: number | null;
  returnedQuantity: number;
  remainingQuantity: number;
  allocations: Allocation[];
}
export interface Preview {
  subtotalMinor: number;
  discountMinor: number;
  taxMinor: number;
  totalMinor: number;
  lines: Item[];
}
export type Method = "CASH" | "CARD" | "OTHER";
export interface Payment {
  paymentMethod: Method;
  amountMinor: number;
  reference: string | null;
}
export interface Sale {
  id: string;
  saleNumber: string | null;
  branchId: string;
  customerId: string | null;
  customerCodeSnapshot: string | null;
  customerName: string | null;
  customerPhoneSnapshot: string | null;
  status: "DRAFT" | "HELD" | "POSTED" | "VOIDED";
  revision: number;
  cart: Cart;
  subtotalMinor: number;
  discountMinor: number;
  taxMinor: number;
  totalMinor: number;
  paymentStatus: string;
  paymentMethods?: string | null;
  createdAt: string;
  postedAt: string | null;
  voidedAt: string | null;
  notes: string | null;
  items: Item[];
  payments: Payment[];
  returns: ReturnDocument[];
  voids: { id: string; reason: string; refundMinor: number }[];
  voidAllocations: { originalAllocationId: string; movementId: string }[];
  refundedMinor: number;
}
export interface ReturnInput {
  id: string;
  saleId: string;
  reason: string;
  lines: { saleItemId: string; quantity: number }[];
}
export interface ReturnDocument extends ReturnInput {
  status: string;
  refundMinor: number;
  allocations?: {
    originalAllocationId: string;
    quantity: number;
    movementId: string;
    refundMinor: number;
  }[];
}
export interface PostInput {
  id: string;
  expectedRevision: number;
  requestId: string;
  payments: Payment[];
}
export interface Query {
  branchId: string;
  search?: string;
  status?: string | null;
  dateFrom?: string | null;
  dateTo?: string | null;
  paymentMethod?: Method | null;
  limit: number;
  offset: number;
}
export const searchProducts = (query: Query) =>
  nativeRequest<{ items: Product[]; total: number }>("search_pos_products", {
    query,
  });
export const saveDraft = (input: {
  id: string;
  expectedRevision: number | null;
  cart: Cart;
}) => nativeRequest<Sale>("save_sale_draft", { input });
export const hold = (input: { id: string; expectedRevision: number }) =>
  nativeRequest<Sale>("hold_sale", { input });
export const resume = (input: { id: string; expectedRevision: number }) =>
  nativeRequest<Sale>("resume_sale", { input });
export const preview = (cart: Cart) =>
  nativeRequest<Preview>("preview_sale_totals", { cart });
export const post = (input: PostInput) =>
  nativeRequest<Sale>("post_sale", { input });
export const list = (query: Query) =>
  nativeRequest<{ items: Sale[]; total: number }>("list_canonical_sales", {
    query,
  });
export const listCustomerSales = (query: {
  customerId: string;
  branchId: string;
  limit?: number;
  offset?: number;
}) =>
  nativeRequest<{ items: Sale[]; total: number; limit: number; offset: number }>(
    "list_customer_sales",
    { query },
  );
export const get = (id: string) =>
  nativeRequest<Sale>("get_canonical_sale", { id });
export const previewReturn = (input: ReturnInput) =>
  nativeRequest<{ refundMinor: number }>("preview_sale_return", { input });
export const createReturn = (input: ReturnInput) =>
  nativeRequest<ReturnDocument>("create_sale_return", { input });
export const postReturn = (input: { id: string; requestId: string }) =>
  nativeRequest<Sale>("post_sale_return", { input });
export const voidSale = (input: {
  id: string;
  requestId: string;
  reason: string;
}) => nativeRequest<Sale>("void_canonical_sale", { input });
