import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { BarChart3, Boxes, CircleDollarSign, PackageCheck } from "lucide-react";
import { Alert, Button, Card, EmptyState, Input, LinkButton, PageHeader, Select, StatCard, Table } from "../../design-system";
import { useBranch } from "../../app/BranchContext";
import { ScopeNotice } from "../../app/ScopeNotice";
import { useLocalization } from "../../localization/LocalizationContext";
import { reporting, type ReportFilter, type ReportPage, type ReportRow, type ReportingOverview } from "../../lib/tauri/reporting";
import { downloadCsv, phase6 } from "../../lib/tauri/phase6";
import "./reports.css";

const PAGE_SIZE = 20;
const money = (value: unknown) => `${(Number(value ?? 0) / 100).toFixed(2)} EGP`;
const text = (value: unknown) => value == null || value === "" ? "—" : String(value);
const errorText = (e: unknown) => e instanceof Error ? e.message : typeof e === "object" && e && "message" in e ? String(e.message) : "The report could not be loaded.";
type Kind = "overview" | "sales" | "sales-product" | "sales-customer" | "stock" | "movements" | "expiry" | "low-stock" | "purchases" | "receipts" | "returns" | "suppliers";
const reports: Record<Exclude<Kind,"overview">,{label:string;load:(filter:ReportFilter)=>Promise<ReportPage<ReportRow>>}> = {
  sales:{label:"Sales detail",load:reporting.sales}, "sales-product":{label:"Sales by product",load:reporting.salesByProduct}, "sales-customer":{label:"Sales by customer",load:reporting.salesByCustomer},
  stock:{label:"Current stock",load:reporting.stock}, movements:{label:"Inventory movements",load:reporting.movements}, expiry:{label:"Expiry",load:reporting.expiry}, "low-stock":{label:"Low and out of stock",load:reporting.lowStock},
  purchases:{label:"Purchase orders",load:reporting.purchases}, receipts:{label:"Purchase receipts",load:reporting.receipts}, returns:{label:"Purchase returns",load:reporting.returns}, suppliers:{label:"Supplier analytics",load:reporting.suppliers},
};
const groups = {
  Sales: ["sales","sales-product","sales-customer"],
  Inventory: ["stock","movements","expiry","low-stock"],
  Purchasing: ["purchases","receipts","returns","suppliers"],
} as const;

export function CanonicalReports(){
 const {t}=useLocalization();
 const {branches}=useBranch();
 const [kind,setKind]=useState<Kind>("overview"), [dateFrom,setDateFrom]=useState(""), [dateTo,setDateTo]=useState(""), [branch,setBranch]=useState(""), [search,setSearch]=useState(""), [status,setStatus]=useState(""), [applied,setApplied]=useState({dateFrom:"",dateTo:"",branch:"",search:"",status:""}), [page,setPage]=useState(0), [attempt,setAttempt]=useState(0);
 const [overview,setOverview]=useState<ReportingOverview|null>(null), [result,setResult]=useState<ReportPage<ReportRow>|null>(null), [error,setError]=useState(""), [loading,setLoading]=useState(true),[exporting,setExporting]=useState(false);
 const filter=useMemo<ReportFilter>(()=>({dateFrom:applied.dateFrom||null,dateTo:applied.dateTo||null,branchId:applied.branch||null,search:applied.search||null,status:applied.status||null,expiryDays:90,limit:PAGE_SIZE,offset:page*PAGE_SIZE}),[applied,page]);
 useEffect(()=>{let active=true;Promise.resolve().then(()=>{if(active){setLoading(true);setError("")}});const work=kind==="overview"?reporting.overview(filter):reports[kind].load(filter);work.then(data=>{if(!active)return;if(kind==="overview"){setOverview(data as ReportingOverview);setResult(null)}else{setResult(data as ReportPage<ReportRow>);setOverview(null)}}).catch(e=>active&&setError(errorText(e))).finally(()=>active&&setLoading(false));return()=>{active=false}},[kind,filter,attempt]);
 const choose=(next:Kind)=>{setKind(next);setPage(0);setStatus("")};
 const columns=columnsFor(kind);
 const exportKind=kind==="sales"?"SALES":kind==="stock"?"INVENTORY_STOCK":null;
 const exportReport=async()=>{if(!exportKind)return;setExporting(true);setError("");try{downloadCsv(await phase6.exportCsv(exportKind,{...filter,limit:undefined,offset:undefined}))}catch(e){setError(errorText(e))}finally{setExporting(false)}};
 return <div className="stack reports-workspace">
  <PageHeader title={t("page.reportsTitle")} description={t("page.reportsCopy")} />
  <nav className="report-tabs" aria-label="Report categories">
   <Button variant={kind==="overview"?"primary":"secondary"} onClick={()=>choose("overview")}>Overview</Button>
   {Object.entries(groups).map(([group,kinds])=><Select key={group} label={`${group} reports`} value={kinds.includes(kind as never)?kind:""} onChange={e=>choose(e.target.value as Kind)}><option value="">{group}</option>{kinds.map(k=><option key={k} value={k}>{reports[k].label}</option>)}</Select>)}
  </nav>
  <Card><div className="report-filters">
   <Input label="From date" type="date" value={dateFrom} max={dateTo||undefined} onChange={e=>setDateFrom(e.target.value)} />
   <Input label="To date" type="date" value={dateTo} min={dateFrom||undefined} onChange={e=>setDateTo(e.target.value)} />
   <Select label="Branch" showLabel value={branch} onChange={e=>setBranch(e.target.value)}><option value="">All branches</option>{branches.map(b=><option key={b.id} value={b.id}>{b.name}</option>)}</Select>
   {kind!=="overview"&&<Input label="Search report" type="search" placeholder="Number, product, partner…" value={search} onChange={e=>setSearch(e.target.value)} />}
   {(kind==="sales"||kind==="purchases")&&<Select label="Status" showLabel value={status} onChange={e=>setStatus(e.target.value)}><option value="">All statuses</option>{(kind==="sales"?["POSTED","VOIDED"]:["DRAFT","ORDERED","PARTIALLY_RECEIVED","RECEIVED","CANCELLED"]).map(s=><option key={s}>{s}</option>)}</Select>}
   <Button variant="primary" onClick={()=>{setApplied({dateFrom,dateTo,branch,search,status});setPage(0)}}>Apply filters</Button>
   <Button onClick={()=>{setDateFrom("");setDateTo("");setBranch("");setSearch("");setStatus("");setApplied({dateFrom:"",dateTo:"",branch:"",search:"",status:""});setPage(0)}}>Reset filters</Button>
  </div></Card>
  <ScopeNotice kind="report" reportScope={applied.branch ? branches.find(b=>b.id===applied.branch)?.name??applied.branch : t("scope.allBranches")} />
  {error&&<Alert title="Report could not be loaded" tone="danger">{error} <Button onClick={()=>setAttempt(v=>v+1)}>Retry</Button></Alert>}
  {loading&&<p role="status">Loading report…</p>}
  {!loading&&!error&&kind==="overview"&&overview&&<Overview value={overview}/>} 
  {!loading&&!error&&kind!=="overview"&&result&&<>{kind==="sales"&&result.summary&&<div className="report-summary" aria-label="Sales report summary"><strong>{result.summary.postedSalesCount} posted sales</strong><span>Gross {money(result.summary.grossSalesMinor)}</span><span>Discounts {money(result.summary.discountMinor)}</span><span>Tax {money(result.summary.taxMinor)}</span><span>Refunded {money(result.summary.refundedMinor)}</span><strong>Net {money(result.summary.netSalesMinor)}</strong><span>{result.summary.returnedQuantity} units returned</span></div>}<Card><div className="canonical-dashboard__heading"><div><h2>{reports[kind].label}</h2><p className="muted">{result.total} matching records</p></div>{exportKind&&<Button disabled={exporting} onClick={exportReport}>{exporting?"Preparing CSV…":"Export CSV"}</Button>}</div>{result.items.length===0?<EmptyState title={t("empty.reportsTitle")} description={t("empty.reportsCopy")} action={<LinkButton primary to="/sales">{t("empty.reportsAction")}</LinkButton>}/>:<><Table label={reports[kind].label} rows={result.items} rowKey={(r)=>String(r.id??r.productPackageId??r.customerId??r.supplierId??r.batchId)} columns={columns}/><div className="report-pagination"><Button disabled={page===0} onClick={()=>setPage(v=>v-1)}>Previous</Button><span>Page {page+1}</span><Button disabled={(page+1)*PAGE_SIZE>=result.total} onClick={()=>setPage(v=>v+1)}>Next</Button></div></>}</Card></>}
 </div>
}

function Overview({value:v}:{value:ReportingOverview}){const stats=[
 {label:"Gross posted sales",value:money(v.grossSalesMinor),detail:`${v.postedSalesCount} posted sales`,icon:CircleDollarSign},
 {label:"Refunded amount",value:money(v.refundedMinor),detail:"Posted linked refunds",icon:BarChart3},
 {label:"Net posted sales",value:money(v.netSalesMinor),detail:`${v.netUnitsSold} net units`,icon:CircleDollarSign},
 {label:"Current stock units",value:String(v.currentStockUnits),detail:`${v.lowStockPackageCount} low/out-of-stock thresholds`,icon:Boxes},
 {label:"Received quantity",value:String(v.receivedQuantity),detail:`${v.purchaseOrderCount} purchase orders`,icon:PackageCheck},
 {label:"Purchase return quantity",value:String(v.purchaseReturnQuantity),detail:"Posted supplier returns",icon:PackageCheck},
];return <div className="report-stats">{stats.map(s=><StatCard key={s.label} {...s}/>)}</div>}

function columnsFor(kind:Kind){const col=(key:string,header:string,render?:(r:ReportRow)=>React.ReactNode)=>({key,header,render:render??((r:ReportRow)=>text(r[key]))});const cash=(key:string,header:string)=>col(key,header,r=>money(r[key]));const link=(key:string,header:string,to:(r:ReportRow)=>string)=>col(key,header,r=><Link to={to(r)}>{text(r[key])}</Link>);switch(kind){
 case "sales":return [link("saleNumber","Sale",r=>`/sales/${r.id}`),col("customerName","Customer"),col("status","Status"),col("units","Units"),col("returnedUnits","Returned"),cash("grossMinor","Gross"),cash("refundedMinor","Refunded"),cash("netMinor","Net"),col("postedAt","Posted")];
 case "sales-product":return [link("productName","Product",r=>`/catalog/${r.productId}`),col("packageLabel","Package"),col("quantitySold","Sold"),col("quantityReturned","Returned"),col("netQuantity","Net units"),cash("netMinor","Net sales")];
 case "sales-customer":return [link("customerCode","Customer",r=>`/customers/${r.customerId}`),col("customerName","Name"),col("saleCount","Sales"),cash("grossMinor","Gross"),cash("refundedMinor","Refunded"),cash("netMinor","Net")];
 case "stock":return [link("productName","Product",r=>`/catalog/${r.productId}`),col("packageLabel","Package"),col("branchName","Branch"),col("quantity","Stock"),col("reorderLevel","Reorder"),col("stockState","State"),cash("acquisitionCostValuationMinor","Acquisition valuation"),col("unvaluedQuantity","Unvalued units")];
 case "movements":return [link("productName","Product",r=>`/catalog/${r.productId}`),col("packageLabel","Package"),col("branchName","Branch"),col("movementType","Movement"),col("quantityDelta","Quantity"),col("lotNumber","Lot"),col("referenceId","Reference"),col("createdAt","Created")];
 case "expiry":return [link("productName","Product",r=>`/catalog/${r.productId}`),col("packageLabel","Package"),col("branchName","Branch"),col("lotNumber","Lot"),col("expiryDate","Expiry"),col("daysToExpiry","Days"),col("quantity","Stock"),col("expiryState","State")];
 case "low-stock":return [link("productName","Product",r=>`/catalog/${r.productId}`),col("packageLabel","Package"),col("branchName","Branch"),col("quantity","Stock"),col("reorderLevel","Reorder"),col("stockState","State")];
 case "purchases":return [link("purchaseNumber","Purchase",r=>`/purchases/${r.id}`),link("supplierName","Supplier",r=>`/suppliers/${r.supplierId}`),col("status","Status"),col("orderedQuantity","Ordered"),col("receivedQuantity","Received"),col("returnedQuantity","Returned"),cash("orderedValueMinor","Ordered value")];
 case "receipts":return [link("documentNumber","Receipt",r=>`/purchases/receipts/${r.id}`),link("purchaseNumber","Purchase",r=>`/purchases/${r.purchaseOrderId}`),col("supplierName","Supplier"),col("quantity","Quantity"),cash("acquisitionValueMinor","Acquisition value"),col("postedAt","Posted")];
 case "returns":return [link("documentNumber","Return",r=>`/purchases/returns/${r.id}`),link("purchaseNumber","Purchase",r=>`/purchases/${r.purchaseOrderId}`),col("supplierName","Supplier"),col("quantity","Quantity"),cash("acquisitionValueMinor","Acquisition value"),col("postedAt","Posted")];
 case "suppliers":return [link("supplierCode","Supplier",r=>`/suppliers/${r.supplierId}`),col("supplierName","Name"),col("orderCount","Orders"),cash("orderedValueMinor","Ordered value"),col("receivedQuantity","Received"),cash("receivedValueMinor","Received value"),col("returnedQuantity","Returned")];
 default:return [];
 }}
