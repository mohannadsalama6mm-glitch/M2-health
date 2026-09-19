import { nativeRequest } from "./client";
export type DocumentKind = "RECEIPT" | "ADJUSTMENT" | "WRITE_OFF" | "TRANSFER" | "REVERSAL";
export type DocumentStatus = "DRAFT" | "READY" | "POSTED" | "CANCELLED";
export type LineAction = "RECEIVE" | "IN" | "OUT" | "DAMAGE" | "EXPIRY" | "TRANSFER";
export interface DocumentLineInput {
  productPackageId: string; batchId: string | null; action: LineAction; quantity: number; reason: string;
  lotNumber: string | null; expiryDate: string | null; costPriceMinor: number | null; supplierReference: string | null;
}
export interface DocumentHeader {
  id: string; documentNumber: string; kind: DocumentKind; status: DocumentStatus; revision: number;
  branchId: string; destinationBranchId: string | null; referenceText: string | null; supplierReference: string | null;
  notes: string | null; createdBy: string | null; reversalOf: string | null; reversalReason: string | null;
  createdAt: string; updatedAt: string; postedAt: string | null;
}
export interface InventoryDocument extends DocumentHeader {
  lines: (DocumentLineInput & { id: string; productLabel: string; packageLabel: string })[];
  movements: { movementId: string; lineId: string; reversesMovementId: string | null; branchId: string; batchId: string; productPackageId: string; movementType: string; quantityDelta: number }[];
  preview: { lineId: string; currentStock: number; projectedStock: number; expiryDate: string | null }[];
}
export interface SaveDocument {
  documentId: string | null; expectedRevision: number | null; kind: DocumentKind;
  branchId: string; destinationBranchId: string | null; referenceText: string | null;
  supplierReference: string | null; notes: string | null; createdBy: string | null; lines: DocumentLineInput[];
}
export interface DocumentQuery { branchId: string; kind?: DocumentKind | null; status?: DocumentStatus | null; dateFrom?: string | null; dateTo?: string | null; limit: number; offset: number }
export interface BatchChoice { id: string; lotNumber: string | null; expiryDate: string | null; costPriceMinor: number | null; quantity: number }
export const saveDocument = (input: SaveDocument) => nativeRequest<InventoryDocument>("save_inventory_document", { input });
export const getDocument = (id: string) => nativeRequest<InventoryDocument>("get_inventory_document", { id });
export const listDocuments = (query: DocumentQuery) => nativeRequest<{ items: DocumentHeader[]; total: number; limit: number; offset: number }>("list_inventory_documents", { query });
export const readyDocument = (documentId: string, expectedRevision: number) => nativeRequest<InventoryDocument>("mark_inventory_document_ready", { input: { documentId, expectedRevision } });
export const cancelDocument = (documentId: string, expectedRevision: number) => nativeRequest<InventoryDocument>("cancel_inventory_document", { input: { documentId, expectedRevision } });
export const postDocument = (documentId: string, expectedRevision: number, requestId: string) => nativeRequest<InventoryDocument>("post_inventory_document", { input: { documentId, expectedRevision, requestId } });
export const reverseDocument = (documentId: string, requestId: string, reason: string) => nativeRequest<InventoryDocument>("reverse_inventory_document", { input: { documentId, requestId, reason } });
export const workflowBatches = (branchId: string, productPackageId: string, offset = 0) => nativeRequest<BatchChoice[]>("list_workflow_batches", { branchId, productPackageId, offset });
/** Keep retry identity through reloads. The backend owns deduplication and payload checks. */
export function documentRequestKey(operation: string, id: string, revision: number, reason = "") {
  const key = `m2:inventory:${operation}:${id}:${revision}:${reason}`;
  const old = localStorage.getItem(key);
  if (old) return old;
  const value = crypto.randomUUID(); localStorage.setItem(key, value); return value;
}
