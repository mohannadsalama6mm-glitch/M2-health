import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { Alert, Button, Card, Input, PageHeader, Select, StatCard } from "../../design-system";
import { BarChart3, Boxes, CircleDollarSign, PackageCheck } from "lucide-react";
import { useBranch } from "../../app/BranchContext";
import { phase6, type Phase6Dashboard } from "../../lib/tauri/phase6";
import "./canonical-dashboard.css";

const money=(n:number|null|undefined)=>n==null?"Unavailable":`${(n/100).toFixed(2)} EGP`;
const errorText=(e:unknown)=>e instanceof Error?e.message:typeof e==="object"&&e&&"message" in e?String(e.message):"Dashboard data could not be loaded.";

export function CanonicalDashboard(){
 const {branches,branchId}=useBranch();
 const [dateFrom,setDateFrom]=useState(""),[dateTo,setDateTo]=useState(""),[branch,setBranch]=useState(""),[applied,setApplied]=useState({dateFrom:"",dateTo:"",branch:""}),[value,setValue]=useState<Phase6Dashboard|null>(null),[loading,setLoading]=useState(true),[error,setError]=useState(""),[attempt,setAttempt]=useState(0);
 const filter=useMemo(()=>({dateFrom:applied.dateFrom||null,dateTo:applied.dateTo||null,branchId:applied.branch||null}),[applied]);
 useEffect(()=>{let active=true;Promise.resolve().then(()=>{if(active){setLoading(true);setError("")}});phase6.dashboard(filter).then(v=>active&&setValue(v)).catch(e=>active&&setError(errorText(e))).finally(()=>active&&setLoading(false));return()=>{active=false}},[filter,attempt]);
 const reset=()=>{setDateFrom("");setDateTo("");setBranch("");setApplied({dateFrom:"",dateTo:"",branch:""})};
 return <div className="stack canonical-dashboard">
  <PageHeader title="Dashboard" description="Canonical operational and limited Finance results from the local pharmacy database." actions={<Link className="button button--primary" to="/sales">New sale</Link>}/>
  <Card><div className="dashboard-filters"><Input label="From date" type="date" value={dateFrom} max={dateTo||undefined} onChange={e=>setDateFrom(e.target.value)}/><Input label="To date" type="date" value={dateTo} min={dateFrom||undefined} onChange={e=>setDateTo(e.target.value)}/><Select label="Branch" showLabel value={branch} onChange={e=>setBranch(e.target.value)}><option value="">All branches</option>{branches.map(b=><option key={b.id} value={b.id}>{b.name}</option>)}</Select><Button variant="primary" onClick={()=>setApplied({dateFrom,dateTo,branch})}>Apply filters</Button><Button onClick={reset}>Reset filters</Button></div><p className="muted">Dates are inclusive. Current branch: {branches.find(b=>b.id===branchId)?.name??branchId}. Stock is the current ledger balance; period filters apply to dated activity.</p></Card>
  {error&&<Alert title="Dashboard could not be loaded" tone="danger">{error} <Button onClick={()=>setAttempt(v=>v+1)}>Retry</Button></Alert>}
  {loading&&<p role="status">Loading Dashboard…</p>}
  {!loading&&!error&&value&&<DashboardBody value={value}/>} 
 </div>;
}
function DashboardBody({value:v}:{value:Phase6Dashboard}){const o=v.operational,f=v.financial,c=v.coverage;return <>
 <section><h2>Operational overview</h2><div className="canonical-dashboard__stats"><StatCard label="Net posted sales" value={money(o.netSalesMinor)} detail={`${o.postedSalesCount} posted sales · ${o.netUnitsSold} net units`} icon={CircleDollarSign}/><StatCard label="Current stock" value={String(o.currentStockUnits)} detail={`${o.lowStockPackageCount} low/out-of-stock thresholds`} icon={Boxes}/><StatCard label="Received" value={String(o.receivedQuantity)} detail={`${o.purchaseOrderCount} purchase orders`} icon={PackageCheck}/><StatCard label="Supplier returns" value={String(o.purchaseReturnQuantity)} detail="Posted purchase return units" icon={PackageCheck}/></div></section>
 <section><div className="canonical-dashboard__heading"><h2>Finance overview</h2><Link to="/finance">Open Finance</Link></div><div className="canonical-dashboard__stats"><StatCard label="Net sales revenue" value={money(f.netSalesRevenueMinor)} detail="Posted Finance journals" icon={CircleDollarSign}/><StatCard label="Net COGS" value={money(f.netCogsMinor)} detail="Posted Finance journals" icon={BarChart3}/><StatCard label="Gross margin" value={money(f.grossMarginMinor)} detail={f.grossMarginMinor==null?"Unavailable: incomplete allocation cost":"Limited Finance scope"} icon={CircleDollarSign}/><StatCard label="Operating result" value={money(f.operatingResultMinor)} detail="Gross margin less recorded expenses" icon={CircleDollarSign}/></div></section>
 <Alert title={c.coverageComplete?"Finance coverage is complete for this filter":"Finance coverage is partial"} tone={c.coverageComplete?"success":"warning"}>{c.scopeNote} {c.representedSaleCount} of {c.eligibleSaleCount} Sales, {c.representedReturnCount} of {c.postedReturnCount} Returns, and {c.representedVoidCount} of {c.voidCount} Voids are represented. <Link to="/reports">Compare operational reports</Link>.</Alert>
 </>}
