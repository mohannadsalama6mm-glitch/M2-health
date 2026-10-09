import type { Page } from "@playwright/test";
import { installCatalogDouble } from "./catalog-double";

export async function installPurchasingDouble(page: Page) {
  await installCatalogDouble(page);
  await page.addInitScript(() => {
    type Order = {
      id: string; purchaseNumber: string; supplierId: string; supplierCode: string;
      supplierName: string; status: string; revision: number; supplierReference: string | null;
      expectedDate: string | null; notes: string | null; subtotalMinor: number;
      discountMinor: number; taxMinor: number; totalMinor: number; createdAt: string;
      updatedAt: string; orderedAt: string | null; cancelledAt: string | null;
      cancellationReason: string | null; lines: Array<Record<string, unknown>>; receipts: Array<Record<string, unknown>>; returns: Array<Record<string, unknown>>;
    };
    const w = window as unknown as {
      __TAURI_INTERNALS__: { invoke: (c: string, a?: Record<string, unknown>) => Promise<unknown> };
      purchasingCalls: { command: string; args: Record<string, unknown> }[];
      purchasingFailCommand: string; purchasingDelay: number; purchasingEmpty: boolean; purchasingReceiveTimeoutOnce: boolean; purchasingReturnTimeoutOnce: boolean;
    };
    const original = w.__TAURI_INTERNALS__.invoke;
    const now = "2026-10-03T10:00:00.000Z";
    const suppliers = [
      { id: "11111111-1111-4111-8111-111111111111", code: "SUP-000001", displayName: "Nile Medical", contactPerson: "Dina", primaryPhone: null, secondaryPhone: null, email: null, address: null, taxRegistrationId: null, notes: null, isActive: true, revision: 1, createdAt: now, updatedAt: now },
      { id: "22222222-2222-4222-8222-222222222222", code: "SUP-000002", displayName: "Active Pharma", contactPerson: null, primaryPhone: null, secondaryPhone: null, email: null, address: null, taxRegistrationId: null, notes: null, isActive: true, revision: 1, createdAt: now, updatedAt: now },
    ];
    const packages = [
      { productId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", productPackageId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", productName: "Panadol 500mg", packageLabel: "Box of 20", manufacturerName: "GSK", barcode: "622100000001", currentCostMinor: 1250 },
      { productId: "cccccccc-cccc-4ccc-8ccc-cccccccccccc", productPackageId: "dddddddd-dddd-4ddd-8ddd-dddddddddddd", productName: "Vitamin C", packageLabel: "Bottle", manufacturerName: "Eva", barcode: "622100000002", currentCostMinor: 800 },
    ];
    const make = (i: number, status = i === 1 ? "DRAFT" : i === 2 ? "ORDERED" : i === 3 ? "CANCELLED" : "DRAFT"): Order => ({
      id: `00000000-0000-4000-8000-${String(i).padStart(12, "0")}`,
      purchaseNumber: `PO-20261003-${String(i).padStart(4, "0")}`,
      supplierId: suppliers[0].id, supplierCode: suppliers[0].code, supplierName: suppliers[0].displayName,
      status, revision: 1, supplierReference: `REF-${i}`, expectedDate: "2026-10-10", notes: "Canonical purchase",
      subtotalMinor: 2500, discountMinor: 100, taxMinor: 200, totalMinor: 2600,
      createdAt: `2026-10-${String(Math.min(i, 28)).padStart(2, "0")}T10:00:00.000Z`, updatedAt: now,
      orderedAt: status === "ORDERED" ? now : null, cancelledAt: status === "CANCELLED" ? now : null,
      cancellationReason: status === "CANCELLED" ? "No longer required" : null,
      lines: [{ id: `line-${i}`, productId: packages[0].productId, productPackageId: packages[0].productPackageId, productNameSnapshot: packages[0].productName, packageLabelSnapshot: packages[0].packageLabel, orderedQuantity: 2, receivedQuantity: 0, remainingQuantity: 2, returnedQuantity: 0, remainingReturnableQuantity: 0, unitCostMinor: 1250, lineTotalMinor: 2500, position: 0 }], receipts: [], returns: [],
    });
    const orders: Order[] = Array.from({ length: 23 }, (_, i) => make(i + 1));
    orders[1].lines.push({id:"line-2b",productId:packages[1].productId,productPackageId:packages[1].productPackageId,productNameSnapshot:packages[1].productName,packageLabelSnapshot:packages[1].packageLabel,orderedQuantity:3,receivedQuantity:0,remainingQuantity:3,returnedQuantity:0,remainingReturnableQuantity:0,unitCostMinor:800,lineTotalMinor:2400,position:1});
    orders[1].subtotalMinor=4900; orders[1].totalMinor=5000;
    const receipts: Array<Record<string, unknown>> = [];
    const purchaseReturns: Array<Record<string, unknown>> = [];
    w.purchasingCalls = []; w.purchasingFailCommand = ""; w.purchasingDelay = 0; w.purchasingEmpty = false; w.purchasingReceiveTimeoutOnce=false; w.purchasingReturnTimeoutOnce=false;
    const save = () => sessionStorage.setItem("phase5b-orders", JSON.stringify(orders));
    const stored = sessionStorage.getItem("phase5b-orders");
    if (stored) orders.splice(0, orders.length, ...JSON.parse(stored));
    w.__TAURI_INTERNALS__.invoke = async (command, args = {}) => {
      w.purchasingCalls.push({ command, args: structuredClone(args) });
      if (command === "recovery_status") return { laterWorkflowsEnabled: false, inventoryWorkflowsEnabled: true, stockCountsEnabled: true, inventoryAlertsEnabled: true, inventoryUiEnabled: true, salesEnabled: true, partnersEnabled: true, purchasingEnabled: true };
      if (command === w.purchasingFailCommand) throw { code: command.includes("update") || command.includes("order") || command === "create_purchase_return" ? "conflict" : "validation", message: command.includes("update") || command === "receive_purchase_order" || command === "create_purchase_return" ? "Purchase order changed. Reload before continuing." : "Controlled backend rejection" };
      if ((command === "list_purchase_orders" || command === "search_purchase_packages") && w.purchasingDelay) await new Promise((resolve) => setTimeout(resolve, w.purchasingDelay));
      if (command === "list_canonical_suppliers") {
        const q = args.query as Record<string, unknown>, search = String(q.search ?? "").toLowerCase();
        const rows = suppliers.filter((s) => s.isActive && (!search || `${s.code} ${s.displayName}`.toLowerCase().includes(search)));
        return { items: rows, total: rows.length, limit: 20, offset: 0 };
      }
      if (command === "search_purchase_packages") {
        const q = args.query as Record<string, unknown>, search = String(q.search ?? "").toLowerCase();
        const rows = packages.filter((p) => !search || `${p.productName} ${p.packageLabel} ${p.manufacturerName} ${p.barcode}`.toLowerCase().includes(search));
        return { items: rows, total: rows.length, limit: 20, offset: 0 };
      }
      if (command === "list_purchase_orders") {
        const q = args.query as Record<string, unknown>, search = String(q.search ?? "").toLowerCase();
        let rows = w.purchasingEmpty ? [] : orders.filter((o) => (!q.status || o.status === q.status) && (!search || `${o.purchaseNumber} ${o.supplierName} ${o.supplierCode} ${o.supplierReference}`.toLowerCase().includes(search)));
        rows = rows.sort((a, b) => b.createdAt.localeCompare(a.createdAt));
        const offset = Number(q.offset ?? 0), limit = Number(q.limit ?? 20);
        return { items: structuredClone(rows.slice(offset, offset + limit).map((o) => ({ ...o, lineCount: o.lines.length, lines: undefined }))), total: rows.length, limit, offset };
      }
      if (command === "get_purchase_order") {
        const row = orders.find((o) => o.id === args.id); if (!row) throw { code: "notFound", message: "Purchase order not found." }; return structuredClone(row);
      }
      if (command === "receive_purchase_order") {
        if (w.purchasingDelay) await new Promise((resolve) => setTimeout(resolve, w.purchasingDelay));
        const input=args.input as {id:string;requestId:string;purchaseOrderId:string;branchId:string;lines:Array<{purchaseOrderLineId:string;quantity:number;lotNumber:string|null;expiryDate:string|null;acquisitionCostMinor:number}>};
        const replay=receipts.find((item)=>item.requestId===input.requestId); if(replay)return structuredClone(replay);
        const order=orders.find((o)=>o.id===input.purchaseOrderId)!;
        if (!["ORDERED","PARTIALLY_RECEIVED"].includes(order.status)) throw {code:"validation",message:"Only ordered or partially received purchases can receive stock."};
        const receiptLines=input.lines.map((requested,index)=>{const source=order.lines.find((line)=>line.id===requested.purchaseOrderLineId)!; if(requested.quantity>Number(source.remainingQuantity)) throw {code:"validation",message:"Received quantity cannot exceed the purchase line remaining quantity."}; source.receivedQuantity=Number(source.receivedQuantity)+requested.quantity; source.remainingQuantity=Number(source.orderedQuantity)-Number(source.receivedQuantity); source.remainingReturnableQuantity=Number(source.receivedQuantity)-Number(source.returnedQuantity); return {id:`receipt-line-${receipts.length}-${index}`,purchaseOrderLineId:source.id,productPackageId:source.productPackageId,productNameSnapshot:source.productNameSnapshot,packageLabelSnapshot:source.packageLabelSnapshot,quantity:requested.quantity,returnedQuantity:0,remainingReturnableQuantity:requested.quantity,currentBatchQuantity:requested.quantity,lotNumber:requested.lotNumber,expiryDate:requested.expiryDate,acquisitionCostMinor:requested.acquisitionCostMinor,batchId:`batch-${receipts.length}-${index}`,movementId:`movement-${receipts.length}-${index}`,position:index};});
        const receipt={id:input.id,requestId:input.requestId,receiptNumber:`PR-20261003-${String(receipts.length+1).padStart(4,"0")}`,purchaseOrderId:order.id,purchaseNumber:order.purchaseNumber,branchId:input.branchId,branchCode:"MAIN",branchName:"Main Pharmacy",supplierId:order.supplierId,supplierCode:order.supplierCode,supplierName:order.supplierName,totalQuantity:receiptLines.reduce((sum,line)=>sum+line.quantity,0),createdBy:"Mohannad",postedAt:now,createdAt:now,lines:receiptLines,returns:[]};
        receipts.push(receipt); order.revision++; order.status=order.lines.every((line)=>Number(line.remainingQuantity)===0)?"RECEIVED":"PARTIALLY_RECEIVED"; order.receipts.push({id:receipt.id,receiptNumber:receipt.receiptNumber,branchId:receipt.branchId,branchCode:receipt.branchCode,branchName:receipt.branchName,totalQuantity:receipt.totalQuantity,postedAt:receipt.postedAt}); save(); if(w.purchasingReceiveTimeoutOnce){w.purchasingReceiveTimeoutOnce=false;throw {code:"internal",message:"Receipt response timed out"}} return structuredClone(receipt);
      }
      if(command==="get_purchase_receipt") { const receipt=receipts.find((item)=>item.id===args.id); if(!receipt) throw {code:"notFound",message:"Purchase receipt not found."}; return structuredClone(receipt); }
      if(command==="list_purchase_receipts") { const query=args.query as Record<string,unknown>; const rows=receipts.filter((item)=>!query.purchaseOrderId||item.purchaseOrderId===query.purchaseOrderId); return {items:structuredClone(rows),total:rows.length,limit:20,offset:0}; }
      if(command==="create_purchase_return") {
        if(w.purchasingDelay)await new Promise((resolve)=>setTimeout(resolve,w.purchasingDelay));
        const input=args.input as {id:string;requestId:string;purchaseOrderId:string;receiptId:string;expectedRevision:number;reason:string;lines:Array<{receiptLineId:string;quantity:number}>};
        const replay=purchaseReturns.find((item)=>item.requestId===input.requestId);if(replay)return structuredClone(replay);
        const receipt=receipts.find((item)=>item.id===input.receiptId) as Record<string,unknown>; const order=orders.find((item)=>item.id===input.purchaseOrderId)!;
        if(input.expectedRevision!==order.revision)throw {code:"conflict",message:"Purchase order changed. Reload before returning stock."};
        const sourceLines=receipt.lines as Array<Record<string,unknown>>;
        const lines=input.lines.map((requested,index)=>{const source=sourceLines.find((item)=>item.id===requested.receiptLineId)!;if(requested.quantity>Number(source.remainingReturnableQuantity))throw {code:"validation",message:"Return quantity exceeds the receipt line's remaining returnable quantity."};if(requested.quantity>Number(source.currentBatchQuantity))throw {code:"validation",message:"Insufficient current stock in the original receipt batch."};const prior=Number(source.returnedQuantity);source.returnedQuantity=prior+requested.quantity;source.remainingReturnableQuantity=Number(source.quantity)-Number(source.returnedQuantity);source.currentBatchQuantity=Number(source.currentBatchQuantity)-requested.quantity;const purchaseLine=order.lines.find((item)=>item.id===source.purchaseOrderLineId)!;purchaseLine.returnedQuantity=Number(purchaseLine.returnedQuantity)+requested.quantity;purchaseLine.remainingReturnableQuantity=Number(purchaseLine.receivedQuantity)-Number(purchaseLine.returnedQuantity);return{id:`return-line-${purchaseReturns.length}-${index}`,receiptLineId:source.id,purchaseOrderLineId:source.purchaseOrderLineId,productPackageId:source.productPackageId,productNameSnapshot:source.productNameSnapshot,packageLabelSnapshot:source.packageLabelSnapshot,receivedQuantity:source.quantity,previouslyReturnedQuantity:prior,remainingReturnableQuantity:source.remainingReturnableQuantity,quantity:requested.quantity,lotNumber:source.lotNumber,expiryDate:source.expiryDate,acquisitionCostMinor:source.acquisitionCostMinor,batchId:source.batchId,movementId:`return-movement-${purchaseReturns.length}-${index}`,position:index}});
        const result={id:input.id,requestId:input.requestId,returnNumber:`PRT-20261003-${String(purchaseReturns.length+1).padStart(4,"0")}`,purchaseOrderId:order.id,purchaseNumber:order.purchaseNumber,receiptId:receipt.id,receiptNumber:receipt.receiptNumber,branchId:receipt.branchId,branchCode:receipt.branchCode,branchName:receipt.branchName,supplierId:order.supplierId,supplierCode:order.supplierCode,supplierName:order.supplierName,sourcePurchaseRevision:order.revision,reason:input.reason,totalQuantity:lines.reduce((sum,line)=>sum+line.quantity,0),createdBy:"Mohannad",postedAt:now,createdAt:now,lines};purchaseReturns.push(result);order.revision++;const summary={id:result.id,returnNumber:result.returnNumber,purchaseOrderId:result.purchaseOrderId,purchaseNumber:result.purchaseNumber,receiptId:result.receiptId,receiptNumber:result.receiptNumber,branchId:result.branchId,branchCode:result.branchCode,branchName:result.branchName,supplierId:result.supplierId,supplierCode:result.supplierCode,supplierName:result.supplierName,totalQuantity:result.totalQuantity,reason:result.reason,postedAt:result.postedAt};order.returns.push(summary);(receipt.returns as Array<Record<string,unknown>>).push(summary);if(w.purchasingReturnTimeoutOnce){w.purchasingReturnTimeoutOnce=false;throw {code:"internal",message:"Return response timed out"}}return structuredClone(result);
      }
      if(command==="get_purchase_return"){const item=purchaseReturns.find((row)=>row.id===args.id);if(!item)throw {code:"notFound",message:"Purchase return not found."};return structuredClone(item)}
      if(command==="list_purchase_returns"){const q=args.query as Record<string,unknown>;const search=String(q.search??"").toLowerCase();let rows=purchaseReturns.filter((item)=>(!q.purchaseOrderId||item.purchaseOrderId===q.purchaseOrderId)&&(!q.supplierId||item.supplierId===q.supplierId)&&(!q.receiptId||item.receiptId===q.receiptId)&&(!q.branchId||item.branchId===q.branchId)&&(!search||`${item.returnNumber} ${item.purchaseNumber} ${item.receiptNumber} ${item.reason}`.toLowerCase().includes(search)));const offset=Number(q.offset??0),limit=Number(q.limit??20);rows=rows.slice().reverse();return{items:structuredClone(rows.slice(offset,offset+limit)),total:rows.length,limit,offset}}
      if (command === "create_purchase_order_draft") {
        if (w.purchasingDelay) await new Promise((resolve) => setTimeout(resolve, w.purchasingDelay));
        const input = args.input as Record<string, unknown>, fields = input as Record<string, unknown>;
        let row = orders.find((o) => o.id === input.id);
        if (!row) { row = make(orders.length + 1, "DRAFT"); row.id = String(input.id); row.purchaseNumber = "PO-20261003-0024"; orders.push(row); }
        apply(row, fields); save(); return structuredClone(row);
      }
      if (command === "update_purchase_order_draft") {
        if (w.purchasingDelay) await new Promise((resolve) => setTimeout(resolve, w.purchasingDelay));
        const input = args.input as Record<string, unknown>, row = orders.find((o) => o.id === input.id)!;
        if (row.status !== "DRAFT") throw { code: "validation", message: "Only draft purchase orders can be edited." };
        row.revision++; apply(row, input); save(); return structuredClone(row);
      }
      if (command === "order_purchase_order" || command === "cancel_purchase_order") {
        if (w.purchasingDelay) await new Promise((resolve) => setTimeout(resolve, w.purchasingDelay));
        const input = args.input as Record<string, unknown>, row = orders.find((o) => o.id === input.id)!;
        row.status = command.startsWith("order") ? "ORDERED" : "CANCELLED"; row.revision++;
        if (row.status === "ORDERED") row.orderedAt = now; else { row.cancelledAt = now; row.cancellationReason = String(input.reason); }
        save(); return structuredClone(row);
      }
      return original(command, args);
    };
    function apply(row: Order, input: Record<string, unknown>) {
      row.supplierId = String(input.supplierId); const supplier = suppliers.find((s) => s.id === row.supplierId)!;
      row.supplierCode = supplier.code; row.supplierName = supplier.displayName; row.supplierReference = input.supplierReference as string | null;
      row.expectedDate = input.expectedDate as string | null; row.notes = input.notes as string | null;
      const lines = input.lines as Array<Record<string, unknown>>;
      row.lines = lines.map((line, i) => { const p = packages.find((x) => x.productPackageId === line.productPackageId)!; const total = Number(line.orderedQuantity) * Number(line.unitCostMinor); return { id: `line-new-${i}`, productId: p.productId, productPackageId: p.productPackageId, productNameSnapshot: p.productName, packageLabelSnapshot: p.packageLabel, orderedQuantity: line.orderedQuantity, receivedQuantity: 0, remainingQuantity: line.orderedQuantity, returnedQuantity: 0, remainingReturnableQuantity: 0, unitCostMinor: line.unitCostMinor, lineTotalMinor: total, position: i }; });
      row.subtotalMinor = row.lines.reduce((sum, line) => sum + Number(line.lineTotalMinor), 0); row.discountMinor = Number(input.discountMinor); row.taxMinor = Number(input.taxMinor); row.totalMinor = row.subtotalMinor - row.discountMinor + row.taxMinor;
    }
  });
}
