/* global window, console, URL */
// Controlled DEV3D branch only. Never applies Catalog imports or retries creation blindly.
import { chromium, expect } from '@playwright/test';
import fs from 'node:fs';
import process from 'node:process';
import { randomUUID } from 'node:crypto';
const stage=process.argv[2];
const output=new URL('../docs/phase-3d/native.json',import.meta.url);
const e=fs.existsSync(output)?JSON.parse(fs.readFileSync(output,'utf8')):{errors:[],documents:[]};
const browser=await chromium.connectOverCDP('http://127.0.0.1:9223');
const page=browser.contexts()[0].pages()[0];page.setDefaultTimeout(15000);
page.on('pageerror',x=>e.errors.push(String(x)));page.on('console',x=>{if(x.type()==='error')e.errors.push(x.text());});
const invoke=(command,args)=>page.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
const save=()=>fs.writeFileSync(output,JSON.stringify(e,null,2));
const pkg='40bad73e-196c-4a7e-81e7-b6dfb90e64af';
const main='f319a917-95d0-47bf-b007-ab76b7c044ae';
const oldDev='461dd52b-1136-409b-a019-9f5429b4ed09';
const stock=async b=>(await invoke('get_package_stock',{branchId:b,productPackageId:pkg})).quantity;
const alerts=()=>invoke('list_inventory_alerts',{query:{branchId:e.branch.id,limit:200,offset:0}});
const evaluate=()=>invoke('evaluate_inventory_alerts',{branchId:e.branch.id});
async function navigate(route){await page.goto(`http://127.0.0.1:1420/inventory/${route}`);await page.getByLabel('Monitoring branch',{exact:true}).selectOption(e.branch.id);await expect(page.getByText('Loading monitoring…')).toHaveCount(0);}
async function document(kind,action,batchId,quantity){
 const draft=await invoke('save_inventory_document',{input:{kind,branchId:e.branch.id,referenceText:`DEV PHASE3D ${action}`,notes:'Controlled advisory alert smoke. Intentional immutable DEV history.',lines:(Array.isArray(batchId)?batchId:[batchId]).map(batchId=>({productPackageId:pkg,batchId,action,quantity,reason:`DEV Phase3D ${action}`}))}});
 e.documents.push({draft});save();
 const ready=await invoke('mark_inventory_document_ready',{input:{documentId:draft.id,expectedRevision:draft.revision}});
 const requestId=randomUUID();e.documents.at(-1).request={documentId:ready.id,expectedRevision:ready.revision,requestId};save();
 const posted=await invoke('post_inventory_document',{input:e.documents.at(-1).request});e.documents.at(-1).posted=posted;save();return posted;
}
try{
 await page.goto('http://127.0.0.1:1420/dashboard');
 expect(await stock(main)).toBe(8);expect(await stock(oldDev)).toBe(2);
 if(stage==='seed'){
  if(e.branch||e.near||e.expired)throw new Error('DEV seed already recorded. Inspect/resume explicitly; do not seed twice.');
  const branches=await invoke('list_branches');expect(branches.some(b=>b.code==='DEV3D')).toBe(false);
  e.branch=await invoke('create_branch',{input:{code:'DEV3D',name:'DEV Phase 3D Alerts',phone:'',address:'Controlled development smoke only'}});save();
  const today=(await evaluate()).businessDate;
  const day=n=>{const d=new Date(`${today}T00:00:00Z`);d.setUTCDate(d.getUTCDate()+n);return d.toISOString().slice(0,10);};
  for(const [key,expiryDate] of [['near',day(10)],['expired',day(-1)],['later',day(120)],['unknown',null]]){
   const input={requestId:randomUUID(),batch:{branchId:e.branch.id,productPackageId:pkg,lotNumber:`DEV-PHASE3D-${key.toUpperCase()}`,expiryDate,costPriceMinor:100},quantity:1,reason:`DEV Phase3D ${key} expiry monitoring`,createdBy:null};
   e[`${key}Request`]=input;save();e[key]=await invoke('record_opening_stock',{input});save();
  }
  await evaluate();e.beforeLevel=await alerts();expect(e.beforeLevel.items.map(a=>a.alertType).sort()).toEqual(['BATCH_EXPIRED','BATCH_EXPIRING']);
  expect((await invoke('list_low_stock_inventory',{query:{branchId:e.branch.id,limit:20,offset:0}})).total).toBe(0);
  e.level=await invoke('set_inventory_level',{input:{branchId:e.branch.id,productPackageId:pkg,reorderLevel:4,minimumStock:0,maximumStock:10}});save();
  await navigate('alerts');
  await page.getByLabel('Warning days').fill('75');await page.getByLabel('Critical days').fill('20');await page.getByRole('button',{name:'Save alert settings'}).click();
  await expect(page.getByRole('table',{name:'Inventory alerts'}).locator('tbody tr')).toHaveCount(3);
  e.initial=await alerts();e.settings=await invoke('get_inventory_alert_settings',{branchId:e.branch.id});
  expect(e.initial.items.map(a=>a.alertType).sort()).toEqual(['BATCH_EXPIRED','BATCH_EXPIRING','LOW_STOCK']);
  for(let i=0;i<10;i++)await evaluate();e.repeated=await alerts();expect(e.repeated.items.map(a=>a.id).sort()).toEqual(e.initial.items.map(a=>a.id).sort());
  await page.screenshot({path:'docs/phase-3d/alerts-native.png',fullPage:true});
  await navigate('expiry');await expect(page.getByRole('table',{name:'Expiry batches'})).toContainText('DEV-PHASE3D-EXPIRED');
  await page.getByLabel('Expiry window').selectOption('30');await expect(page.getByRole('table',{name:'Expiry batches'})).toContainText('DEV-PHASE3D-NEAR');
  await page.screenshot({path:'docs/phase-3d/expiry-native.png',fullPage:true});
  await navigate('low-stock');await expect(page.getByRole('table',{name:'Configured low stock'})).toContainText('4 / 0 / 10');
  e.fefo=await invoke('get_fefo_inventory_batches',{branchId:e.branch.id,productPackageId:pkg,limit:20,offset:0});expect(e.fefo.map(r=>r.batch.id)).toEqual([e.near.batchId,e.later.batchId,e.unknown.batchId]);
  e.mainFefo=await invoke('get_fefo_inventory_batches',{branchId:main,productPackageId:pkg,limit:20,offset:0});expect(e.mainFefo.every(r=>!e.fefo.some(d=>d.batch.id===r.batch.id))).toBe(true);expect(e.mainFefo.reduce((sum,r)=>sum+r.quantity,0)).toBe(8);
  e.seedStock=await stock(e.branch.id);expect(e.seedStock).toBe(4);e.seedPassed=true;save();
 } else if(stage==='resolve'){
  if(!e.seedPassed||e.resolvePassed||e.documents.length)throw new Error('Resolution already started or seed incomplete; inspect before proceeding.');
  await document('WRITE_OFF','EXPIRY',e.expired.batchId,1);
  e.afterExpiry=await alerts();expect(e.afterExpiry.items.find(a=>a.alertType==='BATCH_EXPIRED').status).toBe('RESOLVED');
  await document('ADJUSTMENT','OUT',[e.near.batchId,e.later.batchId,e.unknown.batchId],1);
  e.atZero=await alerts();expect(e.atZero.items.filter(a=>a.status==='ACTIVE').map(a=>a.alertType)).toEqual(['OUT_OF_STOCK']);expect(await stock(e.branch.id)).toBe(0);
  e.zeroFefo=await invoke('get_fefo_inventory_batches',{branchId:e.branch.id,productPackageId:pkg,limit:20,offset:0});expect(e.zeroFefo).toEqual([]);
  e.zeroExpired=await invoke('list_expiry_inventory',{query:{branchId:e.branch.id,expired:true,limit:20,offset:0}});expect(e.zeroExpired.total).toBe(0);
  await navigate('low-stock');await page.getByLabel('Stock condition').selectOption('out');await expect(page.getByRole('table',{name:'Configured low stock'}).locator('tbody tr')).toHaveCount(1);
  await document('ADJUSTMENT','IN',e.near.batchId,5);
  e.final=await alerts();expect(e.final.items.filter(a=>a.status==='ACTIVE').map(a=>a.alertType)).toEqual(['BATCH_EXPIRING']);expect(e.final.items).toHaveLength(4);
  e.histories=await Promise.all(e.final.items.map(a=>invoke('get_inventory_alert',{branchId:e.branch.id,id:a.id})));
  await navigate('alerts');await page.getByLabel('Alert status').selectOption('RESOLVED');await expect(page.getByRole('table',{name:'Inventory alerts'}).locator('tbody tr')).toHaveCount(3);
  await page.getByRole('button',{name:'History',exact:true}).first().click();await expect(page.getByRole('heading',{name:'Alert history'})).toBeVisible();
  await page.screenshot({path:'docs/phase-3d/history-native.png',fullPage:true});
  await page.getByLabel('Stock alerts enabled').uncheck();await page.getByRole('button',{name:'Save alert settings'}).click();
  await expect(page.getByRole('button',{name:'Save alert settings'})).toBeEnabled();e.settings=await invoke('get_inventory_alert_settings',{branchId:e.branch.id});expect(e.settings).toMatchObject({expiryWarningDays:75,criticalExpiryDays:20,expiryEnabled:true,lowStockEnabled:false});
  e.mainSettings=await invoke('get_inventory_alert_settings',{branchId:main});expect(e.mainSettings).toMatchObject({expiryWarningDays:90,criticalExpiryDays:30,expiryEnabled:true,lowStockEnabled:true});
  e.finalStock=await stock(e.branch.id);expect(e.finalStock).toBe(5);e.resolvePassed=true;
 } else if(stage==='restart'){
  if(!e.resolvePassed)throw new Error('Complete controlled resolution first.');
  await navigate('alerts');expect(await invoke('get_inventory_alert_settings',{branchId:e.branch.id})).toEqual(e.settings);
  for(let i=0;i<10;i++)await evaluate();const after=await alerts();expect(after.items.map(a=>[a.id,a.status]).sort()).toEqual(e.final.items.map(a=>[a.id,a.status]).sort());
  const histories=await Promise.all(after.items.map(a=>invoke('get_inventory_alert',{branchId:e.branch.id,id:a.id})));
  expect(histories.map(x=>[x.alert.id,x.history.length]).sort()).toEqual(e.histories.map(x=>[x.alert.id,x.history.length]).sort());
  await navigate('fefo');const product=await invoke('get_product',{id:'1e49bfb5-878c-458b-bce0-5849485282bf'});
  await page.getByLabel('Search Catalog for FEFO').fill(product.commercialNameEn);await page.getByLabel('FEFO product').selectOption(product.id);await page.getByLabel('FEFO package').selectOption(pkg);await expect(page.getByRole('table',{name:'FEFO batches'})).toContainText('DEV-PHASE3D-NEAR');
  await page.screenshot({path:'docs/phase-3d/fefo-native.png',fullPage:true});
  await page.goto('http://127.0.0.1:1420/catalog');await expect(page.getByText('25061 results · local SQLite')).toBeVisible();
  await page.goto('http://127.0.0.1:1420/inventory');await expect(page.getByRole('table',{name:'Verified inventory balances'})).toBeVisible();
  await page.goto('http://127.0.0.1:1420/dashboard');await expect(page.getByRole('heading',{name:/Good morning/})).toBeVisible();
  e.restart={settingsPersist:true,idsAndStatusesPersist:true,historyPersist:true,noDuplicates:true,stock:await stock(e.branch.id),mainStock:await stock(main),oldDevStock:await stock(oldDev),catalogVisible:true};
 } else throw new Error('Choose seed, resolve, or restart.');
 expect(await stock(main)).toBe(8);expect(await stock(oldDev)).toBe(2);expect(e.errors).toEqual([]);save();console.log(`${stage}: PASS`);
}catch(error){e.failure={stage,message:String(error),url:page.url(),main:await page.locator('main').innerText().catch(()=>'')};save();throw error;}finally{await browser.close();}

