import { nativeRequest } from "./client";
import type {
  AdjustResult,
  AdjustStockInput,
  CompleteCountInput,
  CountQuery,
  CreateCountInput,
  ExpiryPage,
  ExpiryQuery,
  FefoAllocation,
  InventorySetting,
  LowStockPage,
  LowStockQuery,
  MovementQuery,
  OpeningStockInput,
  SaveCountItemInput,
  SetReorderInput,
  StockCount,
  StockCountDetail,
  StockCountItem,
  StockCountPage,
  StockMovement,
  StockMovementPage,
  StockOverviewPage,
  StockOverviewQuery,
  StockSummary,
  TransferInput,
  TransferResult,
  WriteOffInput,
} from "./inventory.types";
export { toMajor, toMinor } from "./catalog";
export * from "./inventory.types";
export const getStockOverview = (query: StockOverviewQuery) =>
  nativeRequest<StockOverviewPage>("get_stock_overview", { query });
export const getStockSummary = (branchId: string) =>
  nativeRequest<StockSummary>("get_stock_summary", { branchId });
export const listStockMovements = (query: MovementQuery) =>
  nativeRequest<StockMovementPage>("list_stock_movements", { query });
export const postOpeningStock = (input: OpeningStockInput) =>
  nativeRequest<StockMovement>("post_opening_stock", { input });
export const adjustStock = (input: AdjustStockInput) =>
  nativeRequest<AdjustResult>("adjust_stock", { input });
export const writeOffStock = (input: WriteOffInput) =>
  nativeRequest<StockMovement>("write_off_stock", { input });
export const transferStock = (input: TransferInput) =>
  nativeRequest<TransferResult>("transfer_stock", { input });
export const listStockCounts = (query: CountQuery) =>
  nativeRequest<StockCountPage>("list_stock_counts", { query });
export const getStockCount = (id: string) =>
  nativeRequest<StockCountDetail>("get_stock_count", { id });
export const createStockCount = (input: CreateCountInput) =>
  nativeRequest<StockCount>("create_stock_count", { input });
export const saveCountItem = (input: SaveCountItemInput) =>
  nativeRequest<StockCountItem>("save_count_item", { input });
export const completeStockCount = (input: CompleteCountInput) =>
  nativeRequest<StockCountDetail>("complete_stock_count", { input });
export const listExpiry = (query: ExpiryQuery) =>
  nativeRequest<ExpiryPage>("list_expiry", { query });
export const listLowStock = (query: LowStockQuery) =>
  nativeRequest<LowStockPage>("list_low_stock", { query });
export const setReorderLevel = (input: SetReorderInput) =>
  nativeRequest<InventorySetting>("set_reorder_level", { input });
export const fefoAllocation = (
  branchId: string,
  productPackageId: string,
  quantity?: number | null,
) =>
  nativeRequest<FefoAllocation[]>("fefo_allocation", {
    branchId,
    productPackageId,
    quantity,
  });