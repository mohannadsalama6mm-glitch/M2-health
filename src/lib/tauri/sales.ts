import { nativeRequest } from "./client";
import type {
  CompleteSaleInput,
  CompleteSaleResult,
  PosProductPage,
  PosQuery,
  ReturnSaleInput,
  Sale,
  SaleDetail,
  SalePage,
  SaleQuery,
  SaleReturn,
  VoidSaleInput,
} from "./sales.types";
export * from "./sales.types";
export const completeSale = (input: CompleteSaleInput) =>
  nativeRequest<CompleteSaleResult>("complete_sale", { input });
export const returnSale = (input: ReturnSaleInput) =>
  nativeRequest<SaleReturn>("return_sale", { input });
export const voidSale = (input: VoidSaleInput) =>
  nativeRequest<Sale>("void_sale", { input });
export const listSales = (query: SaleQuery) =>
  nativeRequest<SalePage>("list_sales", { query });
export const getSale = (id: string) => nativeRequest<SaleDetail>("get_sale", { id });
export const posSearch = (query: PosQuery) =>
  nativeRequest<PosProductPage>("pos_search", { query });