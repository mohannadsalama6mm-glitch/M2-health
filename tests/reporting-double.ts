import type { Page } from "@playwright/test";
import { installCatalogDouble } from "./catalog-double";

export async function installReportingDouble(page: Page) {
  await installCatalogDouble(page);
  await page.addInitScript(() => {
    const w = window as unknown as {
      __TAURI_INTERNALS__: { invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> };
      reportingCalls: Array<{command:string;args:Record<string,unknown>}>;
      reportingDelay: number; reportingFail: string; reportingEmpty: boolean;
    };
    const original = w.__TAURI_INTERNALS__.invoke;
    w.reportingCalls=[];w.reportingDelay=0;w.reportingFail="";w.reportingEmpty=false;
    const sale=(i:number)=>({id:`sale-${i}`,saleNumber:`SAL-20261004-${String(i).padStart(4,"0")}`,branchId:"main",customerId:"customer-1",customerName:"Ahmed Hassan",status:i===2?"VOIDED":"POSTED",postedAt:"2026-10-04T10:00:00.000Z",grossMinor:400,discountMinor:20,taxMinor:0,refundedMinor:i===1?200:0,netMinor:i===2?0:i===1?200:400,units:2,returnedUnits:i===1?1:0});
    const sets:Record<string,Array<Record<string,unknown>>>={
      get_sales_report:Array.from({length:23},(_,i)=>sale(i+1)),
      get_sales_by_product_report:[{productId:"product-1",productPackageId:"package-1",productName:"Panadol 500mg",packageLabel:"Box",quantitySold:5,quantityReturned:1,netQuantity:4,grossMinor:1000,refundedMinor:200,netMinor:800}],
      get_sales_by_customer_report:[{customerId:"customer-1",customerCode:"CUS-000001",customerName:"Ahmed Hassan",saleCount:2,lastSaleAt:"2026-10-04T10:00:00.000Z",grossMinor:800,refundedMinor:200,netMinor:600}],
      get_inventory_stock_report:[{branchId:"main",branchName:"Main Branch",productId:"product-1",productPackageId:"package-1",productName:"Panadol 500mg",packageLabel:"Box",quantity:14,reorderLevel:20,stockState:"LOW_STOCK",acquisitionCostValuationMinor:1400,unvaluedQuantity:0}],
      get_inventory_movement_report:[{id:"movement-1",branchId:"main",branchName:"Main Branch",productId:"product-1",productPackageId:"package-1",productName:"Panadol 500mg",packageLabel:"Box",batchId:"batch-1",lotNumber:"LOT-A",movementType:"SALE",quantityDelta:-2,referenceType:"sale",referenceId:"sale-1",reason:"Posted sale",createdAt:"2026-10-04T10:00:00.000Z"}],
      get_expiry_report:[{batchId:"batch-1",branchId:"main",branchName:"Main Branch",productId:"product-1",productPackageId:"package-1",productName:"Panadol 500mg",packageLabel:"Box",lotNumber:"LOT-A",expiryDate:"2026-10-20",quantity:14,expiryState:"EXPIRING",daysToExpiry:16}],
      get_low_stock_report:[{branchId:"main",branchName:"Main Branch",productId:"product-1",productPackageId:"package-1",productName:"Panadol 500mg",packageLabel:"Box",quantity:14,reorderLevel:20,stockState:"LOW_STOCK"}],
      get_purchasing_report:[{id:"purchase-1",purchaseNumber:"PO-20261004-0001",supplierId:"supplier-1",supplierName:"Nile Medical",status:"RECEIVED",createdAt:"2026-10-04T08:00:00.000Z",orderedQuantity:20,orderedValueMinor:2000,receivedQuantity:20,returnedQuantity:2,receiptCount:2,returnCount:1}],
      get_purchase_receipt_report:[{id:"receipt-1",documentNumber:"PR-20261004-0001",purchaseOrderId:"purchase-1",purchaseNumber:"PO-20261004-0001",branchId:"main",supplierId:"supplier-1",supplierName:"Nile Medical",quantity:20,acquisitionValueMinor:2000,postedAt:"2026-10-04T08:00:00.000Z"}],
      get_purchase_return_report:[{id:"return-1",documentNumber:"PRT-20261004-0001",purchaseOrderId:"purchase-1",purchaseNumber:"PO-20261004-0001",receiptId:"receipt-1",branchId:"main",supplierId:"supplier-1",supplierName:"Nile Medical",quantity:2,acquisitionValueMinor:200,reason:"Damaged",postedAt:"2026-10-04T09:00:00.000Z"}],
      get_supplier_analytics:[{supplierId:"supplier-1",supplierCode:"SUP-000001",supplierName:"Nile Medical",isActive:true,orderCount:1,orderedValueMinor:2000,receivedQuantity:20,receivedValueMinor:2000,returnedQuantity:2,returnedValueMinor:200,lastPurchaseAt:"2026-10-04T08:00:00.000Z"}],
    };
    w.__TAURI_INTERNALS__.invoke=async(command,args={})=>{
      if(command==="recovery_status")return {...await original(command,args) as object,reportingEnabled:true};
      if(command==="export_phase6_csv"){w.reportingCalls.push({command,args:structuredClone(args)});return {fileName:"report.csv",mimeType:"text/csv;charset=utf-8",content:"Sale,Customer\r\nSAL-1,Ahmed\r\n",rowCount:1}}
      if(command==="get_reporting_overview"){w.reportingCalls.push({command,args:structuredClone(args)});if(w.reportingDelay)await new Promise(r=>setTimeout(r,w.reportingDelay));if(w.reportingFail===command)throw {code:"database",message:"Controlled reporting failure"};return {postedSalesCount:2,grossSalesMinor:1000,discountMinor:100,taxMinor:0,refundedMinor:200,netSalesMinor:800,netUnitsSold:4,currentStockUnits:14,lowStockPackageCount:1,purchaseOrderCount:1,receivedQuantity:20,purchaseReturnQuantity:2}}
      if(command in sets){w.reportingCalls.push({command,args:structuredClone(args)});if(w.reportingDelay)await new Promise(r=>setTimeout(r,w.reportingDelay));if(w.reportingFail===command)throw {code:"database",message:"Controlled reporting failure"};const filter=(args.filter??{}) as Record<string,unknown>;const rows=w.reportingEmpty?[]:sets[command];const offset=Number(filter.offset??0),limit=Number(filter.limit??20);return {items:structuredClone(rows.slice(offset,offset+limit)),total:rows.length,limit,offset,...(command==="get_sales_report"?{summary:{postedSalesCount:2,grossSalesMinor:1000,discountMinor:100,taxMinor:0,refundedMinor:200,netSalesMinor:800,unitsSold:5,returnedQuantity:1,netUnitsSold:4}}:{})}}
      return original(command,args);
    };
  });
}
