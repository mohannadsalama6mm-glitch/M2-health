import { nativeRequest } from "./client";

export const recoveryStatus = () =>
  nativeRequest<{
    laterWorkflowsEnabled: boolean;
    inventoryWorkflowsEnabled: boolean;
    stockCountsEnabled: boolean;
    inventoryAlertsEnabled: boolean;
    inventoryUiEnabled: boolean;
    salesEnabled: boolean;
    partnersEnabled: boolean;
  }>("recovery_status");

export interface CoreBalance {
  branchId: string;
  productPackageId: string;
  batchId: string | null;
  quantity: number;
}
export interface CoreMovement {
  id: string;
  productPackageId: string;
  batchId: string;
  movementType: string;
  quantityDelta: number;
  reason: string;
  createdAt: string;
}
export const coreInventory = (branchId: string, offset: number) =>
  nativeRequest<CoreBalance[]>("list_branch_inventory", {
    query: { branchId, limit: 20, offset },
  });
export const coreMovements = (branchId: string, offset: number) =>
  nativeRequest<CoreMovement[]>("list_stock_movements", {
    query: { branchId, limit: 20, offset },
  });
