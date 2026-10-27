import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { Alert, BackLink, Button, Card, DetailList, EmptyState, LinkButton, PageHeader, SectionHeader, Status, WorkflowDialog } from "../../design-system";
import { useBranch } from "../../app/BranchContext";
import { useAuth } from "../../app/AuthContext";
import { createBranch, getBranch, type LocalBranch } from "../../lib/tauri/branches";

const message=(e:unknown)=>e instanceof Error?e.message:"The branch operation could not be completed.";
export function CanonicalBranches(){
 const {branches,loading,error,refresh}=useBranch(),auth=useAuth();
 const [open,setOpen]=useState(false),[busy,setBusy]=useState(false),[notice,setNotice]=useState(""),[localError,setLocalError]=useState("");
 return <div className="stack"><PageHeader title="Branches" description="Persisted local branch identities and authorization scope." actions={auth.hasPermission("branches.manage")?<Button variant="primary" onClick={()=>setOpen(true)}>Add branch</Button>:undefined}/>
 {error&&<Alert title="Branches unavailable" tone="danger">{error}</Alert>}{notice&&<Alert title="Branch created" tone="success">{notice}</Alert>}{localError&&<Alert title="Branch could not be created" tone="danger">{localError}</Alert>}
 {loading?<p role="status">Loading branches…</p>:branches.length?<div className="equal-columns">{branches.map(b=><Card key={b.id}><SectionHeader title={b.name} action={<Status value={b.isActive?"Active":"Inactive"}/>} /><DetailList items={[{label:"Code",value:b.code},{label:"Phone",value:b.phone||"—"},{label:"Address",value:b.address||"—"},{label:"Created",value:b.createdAt}]}/><div className="row end"><LinkButton to={`/branches/${b.id}`}>Branch details</LinkButton></div></Card>)}</div>:<EmptyState title="No branches" description="Create the first operational branch."/>}
 <WorkflowDialog open={open} onClose={()=>setOpen(false)} title="Add branch" description="Create a persisted branch in the local pharmacy database." submitLabel={busy?"Creating…":"Create branch"} fields={[{name:"name",label:"Branch name",required:true},{name:"code",label:"Unique code",required:true,pattern:"[A-Za-z0-9_-]{1,32}"},{name:"address",label:"Address"},{name:"phone",label:"Phone"}]} onSubmit={v=>{setBusy(true);setLocalError("");createBranch({name:v.name,code:v.code,address:v.address,phone:v.phone}).then(b=>{setNotice(`${b.name} (${b.code}) is ready.`);refresh()}).catch(e=>setLocalError(message(e))).finally(()=>setBusy(false))}} />
 </div>
}
export function CanonicalBranchDetails(){
 const {id=""}=useParams(),[branch,setBranch]=useState<LocalBranch|null>(null),[loading,setLoading]=useState(true),[error,setError]=useState("");
 useEffect(()=>{let active=true;getBranch(id).then(v=>active&&setBranch(v)).catch(e=>active&&setError(message(e))).finally(()=>active&&setLoading(false));return()=>{active=false}},[id]);
 if(loading)return <p role="status">Loading branch…</p>; if(error||!branch)return <EmptyState title="Branch unavailable" description={error||"The branch was not found."}/>;
 return <><BackLink to="/branches" label="Branches"/><div className="stack"><PageHeader title={branch.name} description={`${branch.code} · persisted local branch`}/><Card><DetailList items={[{label:"Status",value:<Status value={branch.isActive?"Active":"Inactive"}/>},{label:"Phone",value:branch.phone||"—"},{label:"Address",value:branch.address||"—"},{label:"Created",value:branch.createdAt},{label:"Last updated",value:branch.updatedAt}]}/></Card><Alert title="Cloud status is managed separately">Branch synchronization remains disabled until the existing rollout gates are explicitly approved. <LinkButton to="/sync">Open sync status</LinkButton></Alert></div></>;
}
