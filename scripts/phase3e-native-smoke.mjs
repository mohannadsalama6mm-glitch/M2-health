/* global window, document, console, URL */
import { chromium, expect } from '@playwright/test';
import fs from 'node:fs';
import process from 'node:process';
const stage=process.argv[2]||'inspect';
const output=new URL('../docs/phase-3e/native.json',import.meta.url);
const e=fs.existsSync(output)?JSON.parse(fs.readFileSync(output,'utf8')):{errors:[]};
const browser=await chromium.connectOverCDP('http://127.0.0.1:9223');
const page=browser.contexts()[0].pages().find(p=>p.url().includes('1420'));
page.setDefaultTimeout(15000);
page.on('pageerror',x=>e.errors.push(String(x)));page.on('console',x=>{if(x.type()==='error')e.errors.push(x.text());});
const save=()=>fs.writeFileSync(output,JSON.stringify(e,null,2));
const invoke=(command,args)=>page.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
const pkg='40bad73e-196c-4a7e-81e7-b6dfb90e64af',product='1e49bfb5-878c-458b-bce0-5849485282bf';
const main='f319a917-95d0-47bf-b007-ab76b7c044ae',dev='461dd52b-1136-409b-a019-9f5429b4ed09',dev3d='0fc16eaf-cefa-4576-8201-9b03541036f8';
const query=(branchId,extra={})=>({branchId,productPackageId:pkg,limit:200,offset:0,...extra});
const stock=async b=>(await invoke('get_package_stock',{branchId:b,productPackageId:pkg})).quantity;
async function route(path,branch=dev3d){await page.goto('http://127.0.0.1:1420'+path);await page.getByLabel('Branch',{exact:true}).selectOption(branch);}
async function consistency(expected){
 const overview=await invoke('inspect_inventory_overview',{query:query(dev3d)});
 const batches=await invoke('inspect_inventory_batches',{query:query(dev3d)});
 const movements=await invoke('inspect_inventory_movements',{query:query(dev3d)});
 const fefo=await invoke('get_fefo_inventory_batches',{branchId:dev3d,productPackageId:pkg,limit:200,offset:0});
 const values={overview:overview.items[0].quantity,batches:batches.items.reduce((s,r)=>s+r.quantity,0),movements:movements.items.reduce((s,r)=>s+r.quantityDelta,0),fefo:fefo.reduce((s,r)=>s+r.quantity,0),ledger:await stock(dev3d)};
 expect(Object.values(values)).toEqual(Array(5).fill(expected));
 return {values,overview,batches,movements,fefo};
}
try {
 await route('/inventory',main);await expect(page.getByRole('table',{name:'Canonical inventory overview'})).toBeVisible();
 expect(await stock(main)).toBe(8);expect(await stock(dev)).toBe(2);
 if(stage==='final'){
  await route('/inventory');await expect(page.getByText('Local Inventory · persisted SQLite data')).toBeVisible();await expect(page.getByLabel('Global search',{exact:true})).toBeDisabled();
  const shell=await page.evaluate(()=>({mainOverflow:window.getComputedStyle(document.querySelector('main')).overflowY,bodyWidth:document.documentElement.scrollWidth,viewport:window.innerWidth}));expect(shell.mainOverflow).toMatch(/auto|scroll/);expect(shell.bodyWidth).toBeLessThanOrEqual(shell.viewport);
  e.final={consistency:await consistency(6),shell,main:await stock(main),dev:await stock(dev),passed:true};
 } else if(stage==='inspect'){
  e.before=await consistency(5);e.product=await invoke('get_product',{id:product});save();
  for(const [b,q]of [[main,8],[dev,2],[dev3d,5]]){
   await page.getByLabel('Inventory branch',{exact:true}).selectOption(b);
   await expect(page.getByRole('table',{name:'Canonical inventory overview'}).locator('tbody tr').first().locator('td').nth(2)).toHaveText(String(q));
  }
  e.inspectPassed=true;
 } else if(stage==='opening'){
  if(e.openingIntent||!e.inspectPassed)throw new Error('Opening already attempted or inspection missing. Inspect before retry.');
  await route(`/inventory/opening?package=${pkg}&product=${product}`);
  await page.getByLabel('Opening quantity').fill('1');await page.getByLabel('Opening lot').fill('DEV-PHASE3E-OPENING');await page.getByLabel('Opening cost (EGP)').fill('1');await page.getByLabel('Opening reason').fill('DEV Phase3E controlled native opening +1');
  await page.getByRole('button',{name:'Review opening stock'}).click();expect(await stock(dev3d)).toBe(5);
  e.openingIntent=true;save();await page.getByRole('button',{name:'Confirm opening stock'}).click();await expect(page.getByText(/Opening stock recorded:/)).toBeVisible();
  e.openingMessage=await page.getByText(/Opening stock recorded:/).innerText();e.after=await consistency(6);e.openingPassed=true;
 } else if(stage==='integration'){
  e.integration={search:[],traceability:[]};save();
  await route('/inventory');
  for(const search of [e.product.commercialNameEn,e.product.commercialNameAr,e.product.scientificName,'دواء']){
   await page.getByLabel('Search inventory').fill(search);
   const result=await invoke('inspect_inventory_overview',{query:query(dev3d,{search})});
   if(search==='دواء'){expect(result.total).toBe(0);await expect(page.getByText(/No matching inventory records/)).toBeVisible();}
   else {expect(result.total).toBe(1);await expect(page.getByRole('table',{name:'Canonical inventory overview'})).toBeVisible();}
   e.integration.search.push({search,total:result.total});
  }
  await page.getByLabel('Search inventory').fill('');await page.getByLabel('Stock state').selectOption('OUT_OF_STOCK');await expect(page.getByText(/No matching inventory records/)).toBeVisible();
  const all=await invoke('inspect_inventory_batches',{query:query(dev3d)});
  const first=await invoke('inspect_inventory_batches',{query:query(dev3d,{limit:1,offset:0})});const second=await invoke('inspect_inventory_batches',{query:query(dev3d,{limit:1,offset:1})});expect(first.total).toBe(all.total);expect(first.items[0].id).not.toBe(second.items[0].id);
  const overpage=await invoke('inspect_inventory_overview',{query:query(dev3d,{limit:1,offset:1})});expect(overpage.total).toBe(1);expect(overpage.items).toHaveLength(0);
  e.integration.pagination=true;e.integration.zeroBatches=all.items.filter(b=>b.quantity===0);expect(e.integration.zeroBatches.length).toBe(3);
  for(const branch of [main,dev,dev3d]){
   const movements=await invoke('inspect_inventory_movements',{query:query(branch)});
   const unique=[...new Map(movements.items.filter(m=>m.documentId||m.countSessionId).map(m=>[m.documentId||m.countSessionId,m])).values()];
   for(const m of unique){await route('/inventory/movements',branch);const label=m.documentNumber||m.countNumber;await page.getByRole('link',{name:label,exact:true}).first().click();await expect(page.getByText(m.countSessionId?'Completed count is read-only':'Posted document is read-only',{exact:false}).first()).toBeVisible();e.integration.traceability.push({id:m.documentId||m.countSessionId,label});save();}
  }
  await route('/inventory');for(const label of ['Receive Stock','Adjust Stock','Damage / Expiry Write-Off','Transfer Stock']){await route('/inventory');await page.getByRole('link',{name:label,exact:true}).click();await expect(page.getByRole('heading',{name:label,exact:true})).toBeVisible();}
  await route('/inventory/expiry');await page.getByLabel('Expiry window').selectOption('30');await page.getByRole('table',{name:'Expiry batches'}).getByRole('link',{name:'Write-off workflow'}).click();await expect(page.getByLabel('Batch',{exact:true})).toHaveValue(e.before.fefo[0].batch.id);
  e.integration.passed=true;
 } else if(stage==='level'){
  await route(`/inventory/levels?package=${pkg}&product=${product}`);await expect(page.getByLabel('Reorder level',{exact:true})).toHaveValue(/^[46]$/);
  await page.getByLabel('Reorder level',{exact:true}).fill('6');await page.getByRole('button',{name:'Save reorder level'}).click();await expect(page.getByText('Reorder level saved.')).toBeVisible();
  await route(`/inventory/levels?package=${pkg}&product=${product}`);await expect(page.getByLabel('Reorder level',{exact:true})).toHaveValue('6');
  const low=await invoke('list_low_stock_inventory',{query:{branchId:dev3d,limit:20,offset:0}});expect(low.total).toBe(1);e.levelDuring=low;save();
  await page.getByLabel('Reorder level',{exact:true}).fill('4');await page.getByRole('button',{name:'Save reorder level'}).click();await expect(page.getByText('Reorder level saved.')).toBeVisible();
  await route(`/inventory/levels?package=${pkg}&product=${product}`);await expect(page.getByLabel('Reorder level',{exact:true})).toHaveValue('4');expect((await invoke('list_low_stock_inventory',{query:{branchId:dev3d,limit:20,offset:0}})).total).toBe(0);e.levelPassed=true;
 } else if(stage==='screens'||stage==='restart'){
  e[stage]={consistency:await consistency(6),screens:[]};save();
  for(const [path,label]of [['/inventory','Canonical inventory overview'],[`/inventory/batches?package=${pkg}`,'Canonical inventory batches'],['/inventory/movements','Canonical stock movements'],['/inventory/expiry','Expiry batches'],['/inventory/alerts','Inventory alerts'],[`/inventory/fefo?package=${pkg}&product=${product}`,'FEFO batches']]){
   await route(path);await expect(page.getByRole('table',{name:label})).toBeVisible();e[stage].screens.push(path);save();
  }
  await route('/inventory/low-stock');await expect(page.getByRole('heading',{name:/Low Stock/i}).first()).toBeVisible();
  await route('/inventory/documents');await expect(page.getByRole('button',{name:'Receive Stock',exact:true})).toBeVisible();
  await route('/inventory/count-sessions',dev);await expect(page.getByRole('button',{name:'New count',exact:true})).toBeVisible();
  await route(`/inventory/levels?package=${pkg}&product=${product}`);await expect(page.getByLabel('Reorder level',{exact:true})).toHaveValue('4');
  e[stage].settings=await invoke('get_inventory_alert_settings',{branchId:dev3d});expect(e[stage].settings.lowStockEnabled).toBe(false);
  await route('/catalog');await expect(page.getByText('25061 results · local SQLite')).toBeVisible();
  for(const [width,height]of [[1366,768],[1440,900],[1920,1080]]){await page.setViewportSize({width,height});await route('/inventory');await expect(page.getByRole('table',{name:'Canonical inventory overview'})).toBeVisible();expect(await page.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth)).toBe(true);await page.screenshot({path:`docs/phase-3e/${stage}-${width}.png`});}
  e[stage].passed=true;
 }
 expect(e.errors).toEqual([]);save();console.log(JSON.stringify({stage,passed:true,errors:e.errors}));
}catch(error){e.failure={stage,error:String(error)};save();throw error;}finally{await browser.close();}
