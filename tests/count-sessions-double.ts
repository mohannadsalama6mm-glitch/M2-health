import type { Page } from '@playwright/test';
import type { CountDetail,CountEntry,CountLine,CountSession } from '../src/lib/tauri/stock-counts';
import { installCatalogDouble } from './catalog-double';
export async function installCountsDouble(page:Page){
 await installCatalogDouble(page);
 await page.addInitScript(()=>{
  const w=window as unknown as {__TAURI_INTERNALS__:{invoke:(c:string,a?:Record<string,unknown>)=>Promise<unknown>};countCalls:{command:string;args?:Record<string,unknown>}[];countLive:number;countFailure:string;countTimeoutOnce:boolean;countTotal:number;countExtraLine:boolean;countCorrections:number};
  const original=w.__TAURI_INTERNALS__.invoke;const sessions:CountDetail[]=[];const requests=new Map<string,CountDetail>();
  w.countCalls=[];w.countLive=10;w.countFailure='';w.countTimeoutOnce=false;w.countTotal=0;w.countExtraLine=false;w.countCorrections=0;
  const refresh=(d:CountDetail)=>{
   if(d.session.status!=='COMPLETED')for(const l of d.lines){l.currentLive=w.countLive;l.snapshotVariance=l.countedQuantity===null?null:l.countedQuantity-l.expectedQuantity;l.correctionDelta=l.excludedReason?0:l.countedQuantity===null?null:l.countedQuantity-w.countLive;l.interveningMovements=w.countLive!==l.expectedQuantity?1:0;}
   d.summary={total:d.lines.length,counted:d.lines.filter(l=>l.countedQuantity!==null).length,uncounted:d.lines.filter(l=>l.countedQuantity===null&&!l.excludedReason).length,excluded:d.lines.filter(l=>l.excludedReason).length,matches:d.lines.filter(l=>l.snapshotVariance===0).length,positive:d.lines.filter(l=>(l.snapshotVariance??0)>0).length,negative:d.lines.filter(l=>(l.snapshotVariance??0)<0).length,corrections:d.lines.filter(l=>(l.correctionDelta??0)!==0).length,intervening:d.lines.filter(l=>l.interveningMovements>0).length};d.reviewToken=`${d.session.revision}:${w.countLive}`;return structuredClone(d);
  };
  w.__TAURI_INTERNALS__.invoke=async(command,args)=>{
   w.countCalls.push({command,args});
   if(command==='recovery_status')return {laterWorkflowsEnabled:false,inventoryWorkflowsEnabled:true,stockCountsEnabled:true};
   if(command==='list_count_sessions'){const q=args!.query as {offset:number;limit:number;status:string|null};const filtered=sessions.filter(d=>!q.status||q.status===d.session.status);return {items:filtered.slice(q.offset,q.offset+q.limit).map(d=>({...d.session,summary:refresh(d).summary})),total:w.countTotal||filtered.length};}
   if(command==='get_count_session')return refresh(sessions.find(d=>d.session.id===args!.id)!);
   if(command==='create_count_session'){const i=args!.input as {branchId:string;notes:string|null};const session:CountSession={id:`count-${sessions.length+1}`,sessionNumber:`CNT-DEV-${sessions.length+1}`,branchId:i.branchId,status:'DRAFT',scopeType:'ALL_INVENTORY',notes:i.notes,createdBy:null,revision:1,snapshotWatermark:null,createdAt:'2026-09-14T00:00:00Z',startedAt:null,reviewedAt:null,completedAt:null,cancelledAt:null};const d={session,lines:[],summary:{total:0,counted:0,uncounted:0,excluded:0,matches:0,positive:0,negative:0,corrections:0,intervening:0},reviewToken:'',limit:20,offset:0};sessions.push(d);return refresh(d);}
   if(['start_count_session','record_count_entry','review_count_session','resume_count_session','cancel_count_session','complete_count_session'].includes(command)){
    const i=args!.input as CountEntry&{requestId:string;reviewToken:string};const d=sessions.find(d=>d.session.id===i.sessionId)!;
    if(command==='complete_count_session'&&requests.has(i.requestId))return structuredClone(requests.get(i.requestId));
    if(w.countFailure)throw {code:'conflict',message:w.countFailure};
    if(command==='start_count_session'){d.session.status='IN_PROGRESS';d.session.snapshotWatermark=1;d.session.startedAt=d.session.createdAt;d.lines=Array.from({length:w.countExtraLine?2:1},(_,n):CountLine=>({id:`line-${n+1}`,sessionId:d.session.id,productPackageId:'pkg-1',batchId:`batch-${n+1}`,expectedQuantity:10,countedQuantity:null,notes:null,excludedReason:null,countedAt:null,revision:1,productLabel:'DEV count medicine',packageLabel:'Box',lotNumber:`DEV LOT ${n+1}`,expiryDate:null,currentLive:10,snapshotVariance:null,correctionDelta:null,interveningMovements:0,movementId:null}));}
    if(command==='record_count_entry'){const l=d.lines.find(l=>l.id===i.lineId)!;l.countedQuantity=i.countedQuantity;l.notes=i.notes;l.excludedReason=i.excludedReason;l.revision++;}
    if(command==='review_count_session')d.session.status='REVIEW';
    if(command==='resume_count_session')d.session.status='IN_PROGRESS';
    if(command==='cancel_count_session')d.session.status='CANCELLED';
    if(command==='complete_count_session'){refresh(d);if(i.reviewToken!==d.reviewToken)throw {code:'conflict',message:'Stock changed during count review. Refresh review.'};for(const l of d.lines){if(l.correctionDelta){l.movementId=`count-movement-${l.id}`;w.countCorrections++;}}d.session.status='COMPLETED';d.session.completedAt=d.session.createdAt;}
    d.session.revision++;const result=refresh(d);
    if(command==='complete_count_session'){requests.set(i.requestId,result);if(w.countTimeoutOnce){w.countTimeoutOnce=false;throw {code:'io',message:'Completion response timed out. Retry with the same request.'};}}
    return result;
   }
   return original(command,args);
  };
 });
}
