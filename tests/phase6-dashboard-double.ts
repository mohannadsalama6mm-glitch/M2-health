import type { Page } from "@playwright/test";
import { installCatalogDouble } from "./catalog-double";

export async function installPhase6DashboardDouble(page:Page){
 await installCatalogDouble(page);
 await page.addInitScript(()=>{
  const w=window as unknown as {__TAURI_INTERNALS__:{invoke:(c:string,a?:Record<string,unknown>)=>Promise<unknown>};phase6Calls:Array<{command:string;args:Record<string,unknown>}>;phase6Fail:boolean;phase6Delay:number};
  const original=w.__TAURI_INTERNALS__.invoke;w.phase6Calls=[];w.phase6Fail=false;w.phase6Delay=0;
  w.__TAURI_INTERNALS__.invoke=async(command,args={})=>{
   if(command==="recovery_status")return {...await original(command,args) as object,dashboardEnabled:true,reportingEnabled:true,financeEnabled:true};
   if(command==="get_phase6_dashboard"){
    w.phase6Calls.push({command,args:structuredClone(args)});if(w.phase6Delay)await new Promise(r=>setTimeout(r,w.phase6Delay));if(w.phase6Fail)throw {code:"database",message:"Controlled Dashboard failure"};
    return {operational:{postedSalesCount:3,grossSalesMinor:30000,discountMinor:1000,taxMinor:0,refundedMinor:5000,netSalesMinor:25000,netUnitsSold:8,currentStockUnits:42,lowStockPackageCount:2,purchaseOrderCount:4,receivedQuantity:30,purchaseReturnQuantity:2},financial:{salesRevenueMinor:29000,salesReturnsMinor:5000,netSalesRevenueMinor:24000,refundOutflowMinor:5000,netCogsMinor:12000,missingCogsJournalCount:0,grossMarginMinor:12000,operatingExpensesMinor:3000,operatingResultMinor:9000},coverage:{eligibleSaleCount:3,representedSaleCount:2,unrepresentedSaleCount:1,postedReturnCount:1,representedReturnCount:1,unrepresentedReturnCount:0,voidCount:0,representedVoidCount:0,unrepresentedVoidCount:0,unrepresentedEventCount:1,coverageComplete:false,scopeNote:"Finance includes only operational source events with posted Finance journals. Historical operational records were not backfilled."}};
   }
   return original(command,args);
  };
 });
}
