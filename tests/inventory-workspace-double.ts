import type {Page} from '@playwright/test';
import {installDocumentsDouble} from './inventory-documents-double';
import {installAlertsDouble} from './inventory-alerts-double';
export async function installWorkspaceDouble(page:Page){
 await installDocumentsDouble(page);await installAlertsDouble(page,{catalog:false});
 await page.addInitScript(()=>{
 const w=window as unknown as {__TAURI_INTERNALS__:{invoke:(c:string,a?:Record<string,unknown>)=>Promise<unknown>};inspectionCalls:{command:string;args?:Record<string,unknown>}[];inspectionStock:number;inspectionTotal:number;inspectionError:string;inspectionDelay:number};
 const original=w.__TAURI_INTERNALS__.invoke;w.inspectionCalls=[];w.inspectionStock=5;w.inspectionTotal=1;w.inspectionError='';w.inspectionDelay=0;
 w.__TAURI_INTERNALS__.invoke=async(command,args)=>{
  w.inspectionCalls.push({command,args});
  if(command==='recovery_status')return {laterWorkflowsEnabled:false,inventoryWorkflowsEnabled:true,stockCountsEnabled:true,inventoryAlertsEnabled:true,inventoryUiEnabled:true};
  if(command==='list_count_sessions')return {items:[],total:0,limit:20,offset:0};
  if(command==='set_inventory_level')return args!.input;
  if(command==='record_opening_stock'){w.inspectionStock+=(args!.input as {quantity:number}).quantity;return {id:'opening-native-1',batchId:'new-opening-batch',quantityDelta:(args!.input as {quantity:number}).quantity};}
  if(command.startsWith('inspect_inventory_')){
   if(w.inspectionDelay)await new Promise(r=>setTimeout(r,w.inspectionDelay));
   if(w.inspectionError)throw {code:'database',message:w.inspectionError};
   const q=args!.query as {branchId:string;stockState:string|null};const qty=w.inspectionStock;
   const shared={branchId:q.branchId,productPackageId:'pkg-1',productId:'p-1',productName:'Panadol 500 mg',packageLabel:'Box · 10 tablets'};
   const item=command==='inspect_inventory_overview'?{...shared,quantity:qty,batchCount:1,nearestExpiry:'2030-01-01',reorderLevel:null,minimumStock:null,maximumStock:null,stockState:qty===0?'OUT_OF_STOCK':'IN_STOCK',packageActive:1,productActive:1}:command==='inspect_inventory_batches'?{...shared,id:'batch-1',lotNumber:'DEV lot',expiryDate:'2030-01-01',receivedAt:'2026-09-01',costPriceMinor:100,isActive:1,quantity:qty}:{...shared,id:'movement-1',batchId:'batch-1',movementType:'PURCHASE_RECEIPT',quantityDelta:qty,createdAt:'2026-09-15T00:00:00Z',reason:'DEV receipt',createdBy:'tester',referenceType:'inventory_document_line',referenceId:'line-1',documentId:'doc-1',documentNumber:'RCV-DEV-1',countSessionId:null,countNumber:null};
   return {items:w.inspectionTotal===0?[]:[item],total:w.inspectionTotal,businessDate:'2026-09-15'};
  }
  return original(command,args);
 };
 });
}
