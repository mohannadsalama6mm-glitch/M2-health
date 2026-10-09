import { nativeRequest } from "./client";
import type { FinanceJournal, FinanceQuery, SalesFinancialSummary } from "./finance";
import type { ReportFilter, ReportingOverview } from "./reporting";

export interface Phase6Filter { dateFrom?:string|null; dateTo?:string|null; branchId?:string|null }
export interface FinanceCoverage {
  eligibleSaleCount:number; representedSaleCount:number; unrepresentedSaleCount:number;
  postedReturnCount:number; representedReturnCount:number; unrepresentedReturnCount:number;
  voidCount:number; representedVoidCount:number; unrepresentedVoidCount:number;
  unrepresentedEventCount:number; coverageComplete:boolean; scopeNote:string;
}
export interface Phase6Dashboard { operational:ReportingOverview; financial:SalesFinancialSummary; coverage:FinanceCoverage }
export interface SaleFinanceTrace { saleId:string; financePosted:boolean; historicalUnposted:boolean; journals:FinanceJournal[]; coverageNote:string }
export type CsvKind="SALES"|"INVENTORY_STOCK"|"EXPENSES"|"TRIAL_BALANCE"|"ACCOUNT_ACTIVITY";
export interface CsvExport { fileName:string; mimeType:string; content:string; rowCount:number }

const call=<T>(command:string,args:Record<string,unknown>)=>nativeRequest<T>(command,args);
const dashboardInflight=new Map<string,Promise<Phase6Dashboard>>();
const dashboard=(filter:Phase6Filter)=>{
  const key=JSON.stringify(filter),existing=dashboardInflight.get(key);if(existing)return existing;
  const request=call<Phase6Dashboard>("get_phase6_dashboard",{filter}).then(value=>{window.setTimeout(()=>dashboardInflight.delete(key),100);return value},error=>{dashboardInflight.delete(key);throw error});dashboardInflight.set(key,request);return request;
};
export const phase6={
  dashboard,
  reconciliation:(filter:Phase6Filter)=>call<FinanceCoverage>("get_phase6_reconciliation",{filter}),
  saleTrace:(saleId:string)=>call<SaleFinanceTrace>("get_sale_finance_trace",{saleId}),
  exportCsv:(kind:CsvKind,reportFilter:ReportFilter={},financeQuery:FinanceQuery={})=>call<CsvExport>("export_phase6_csv",{input:{kind,reportFilter,financeQuery}}),
};

export function downloadCsv(value:CsvExport){
  const blob=new Blob(["\uFEFF",value.content],{type:value.mimeType});
  const url=URL.createObjectURL(blob),anchor=document.createElement("a");
  anchor.href=url;anchor.download=value.fileName;anchor.click();URL.revokeObjectURL(url);
}
