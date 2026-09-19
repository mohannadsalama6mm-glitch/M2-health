import type { Page } from "@playwright/test";
import type { InventoryDocument, SaveDocument, DocumentQuery } from "../src/lib/tauri/inventory-documents";
import { installCatalogDouble } from "./catalog-double";

export async function installDocumentsDouble(page: Page) {
  await installCatalogDouble(page);
  await page.addInitScript(() => {
    const w = window as unknown as {
      __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
      documentCalls: { command: string; args?: Record<string, unknown> }[];
      documentExpired: boolean; documentFailure: string; postDelay: number; documentTotal: number;
    };
    const original = w.__TAURI_INTERNALS__.invoke;
    const documents: InventoryDocument[] = [];
    w.documentExpired = false; w.documentCalls = []; w.documentFailure = ""; w.postDelay = 0; w.documentTotal = 0;
    w.__TAURI_INTERNALS__.invoke = async (command, args) => {
      w.documentCalls.push({ command, args });
      if (command === "recovery_status") return { laterWorkflowsEnabled: false, inventoryWorkflowsEnabled: true };
      if (command === "list_branches") {
        const rows = await original(command, args) as { id: string; name: string }[];
        return [...rows, { ...rows[0], id: "dev-branch", name: "DEV destination", code: "DEV" }];
      }
      if (command === "list_workflow_batches") return [{ id: "batch-1", lotNumber: "DEV lot", expiryDate: w.documentExpired ? "2020-01-01" : null, costPriceMinor: null, quantity: 8 }];
      if (command === "list_inventory_documents") {
        const q = args!.query as DocumentQuery;
        const rows = documents.filter(d => d.branchId === q.branchId && (!q.kind || d.kind === q.kind) && (!q.status || d.status === q.status));
        return { items: rows.slice(q.offset, q.offset + q.limit), total: w.documentTotal || rows.length, limit: q.limit, offset: q.offset };
      }
      if (command === "get_inventory_document") return structuredClone(documents.find(d => d.id === args!.id));
      if (command === "save_inventory_document") {
        if (w.documentFailure) throw { message: w.documentFailure, code: "conflict" };
        const i = args!.input as SaveDocument;
        const d: InventoryDocument = { ...i, id: i.documentId || `doc-${documents.length + 1}`, documentNumber: `DEV-${documents.length + 1}`, status: "DRAFT", revision: (i.expectedRevision || 0) + 1, reversalOf: null, reversalReason: null, createdAt: "2026-09-14T00:00:00Z", updatedAt: "2026-09-14T00:00:00Z", postedAt: null,
          lines: i.lines.map((l, n) => ({ ...l, id: `line-${n}`, productLabel: "Panadol 500 mg", packageLabel: "Box · 10 tablets" })), movements: [],
          preview: i.lines.map((l, n) => ({ lineId: `line-${n}`, currentStock: l.action === "RECEIVE" ? 0 : 8, projectedStock: l.action === "RECEIVE" ? l.quantity : 8 + (l.action === "IN" ? l.quantity : -l.quantity), expiryDate: l.expiryDate })),
        };
        const index = documents.findIndex(x => x.id === d.id); if (index < 0) documents.push(d); else documents[index] = d;
        return structuredClone(d);
      }
      if (["mark_inventory_document_ready", "post_inventory_document", "cancel_inventory_document"].includes(command)) {
        if (w.documentFailure) throw { message: w.documentFailure, code: "validation" };
        const i = args!.input as { documentId: string };
        const d = documents.find(x => x.id === i.documentId)!;
        if (command === "mark_inventory_document_ready" && d.lines.some(l => l.action === "EXPIRY") && !w.documentExpired) throw { message: "EXPIRY requires a known expired date.", code: "validation" };
        if (command === "post_inventory_document") {
          await new Promise(resolve => setTimeout(resolve, w.postDelay));
          d.status = "POSTED"; d.postedAt = d.createdAt;
          d.movements = d.lines.flatMap(l => (d.kind === "TRANSFER" ? [-l.quantity, l.quantity] : [l.action === "RECEIVE" || l.action === "IN" ? l.quantity : -l.quantity]).map((quantityDelta, n) => ({ movementId: `canonical-${l.id}-${n}`, lineId: l.id, reversesMovementId: null, branchId: n ? d.destinationBranchId! : d.branchId, batchId: "batch-1", productPackageId: l.productPackageId, movementType: d.kind === "TRANSFER" ? (n ? "TRANSFER_IN" : "TRANSFER_OUT") : l.action === "RECEIVE" ? "PURCHASE_RECEIPT" : l.action, quantityDelta })));
        } else d.status = command === "cancel_inventory_document" ? "CANCELLED" : "READY";
        d.revision++; return structuredClone(d);
      }
      return original(command, args);
    };
  });
}

