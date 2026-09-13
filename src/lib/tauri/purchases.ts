import { nativeRequest } from "./client";
import type {
  CompletePurchaseInput,
  CompletePurchaseResult,
  Purchase,
  PurchaseDetail,
  PurchasePage,
  PurchaseProductPage,
  PurchaseQuery,
  PosPurchaseQuery,
  VoidPurchaseInput,
} from "./purchases.types";

export const completePurchase = (input: CompletePurchaseInput) =>
  nativeRequest<CompletePurchaseResult>("complete_purchase", { input });
export const voidPurchase = (input: VoidPurchaseInput) =>
  nativeRequest<Purchase>("void_purchase", { input });
export const listPurchases = (query: PurchaseQuery = {}) =>
  nativeRequest<PurchasePage>("list_purchases", { query });
export const getPurchase = (id: string) =>
  nativeRequest<PurchaseDetail>("get_purchase", { id });
export const purchasePosSearch = (query: PosPurchaseQuery) =>
  nativeRequest<PurchaseProductPage>("purchase_pos_search", { query });