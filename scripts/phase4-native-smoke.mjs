/* global window, document, console, URL */
// Controlled DEV3D only. Never replay a write stage automatically after an uncertain result.
import {chromium,expect} from '@playwright/test';
import fs from 'node:fs';
import process from 'node:process';
const stage=process.argv[2];
const folder=new URL('../docs/phase-4/',import.meta.url),output=new URL('native.json',folder);
const checks=JSON.parse(fs.readFileSync(new URL('checks.json',folder),'utf8'));
if(!checks.automatedPassed)throw new Error('All automated checks must pass before native smoke.');
const e=fs.existsSync(output)?JSON.parse(fs.readFileSync(output,'utf8')):{errors:[]};
const save=()=>fs.writeFileSync(output,JSON.stringify(e,null,2));
const browser=await chromium.connectOverCDP('http://127.0.0.1:9223');
const page=browser.contexts()[0].pages().find(p=>p.url().includes('1420'));page.setDefaultTimeout(20000);
page.on('pageerror',x=>e.errors.push(String(x)));page.on('console',x=>{if(x.type()==='error')e.errors.push(x.text());});
const pkg='40bad73e-196c-4a7e-81e7-b6dfb90e64af',branch='0fc16eaf-cefa-4576-8201-9b03541036f8',main='f319a917-95d0-47bf-b007-ab76b7c044ae',dev='461dd52b-1136-409b-a019-9f5429b4ed09';
const invoke=(command,args)=>page.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
const stock=async b=>(await invoke('get_package_stock',{branchId:b,productPackageId:pkg})).quantity;
async function route(path){await page.goto('http://127.0.0.1:1420'+path);await page.getByLabel('Branch',{exact:true}).selectOption(branch);}
async function capture(){await page.evaluate(()=>{window.phase4Calls=[];const original=window.__TAURI_INTERNALS__.invoke;window.__TAURI_INTERNALS__.invoke=async(command,args)=>{window.phase4Calls.push({command,args});return original(command,args);};});}
async function captured(command){return page.evaluate(c=>window.phase4Calls.filter(x=>x.command===c).at(-1)?.args,command);}
async function add(){await page.getByLabel('POS search / barcode').fill('1 2 3 (ONE TWO THREE)');await expect(page.getByRole('table',{name:'POS Catalog results'}).locator('tbody tr')).toHaveCount(1);await page.getByRole('table',{name:'POS Catalog results'}).getByRole('button',{name:'Add',exact:true}).click();await expect(page.getByLabel('Validated totals')).toBeVisible();}
try{
 await route('/sales');expect(await stock(main)).toBe(8);expect(await stock(dev)).toBe(2);
 if(stage==='sale'){
  if(e.saleIntent)throw new Error('Sale stage already attempted; inspect stored evidence before continuing.');
  expect(await stock(branch)).toBe(6);e.baselineStock=6;
  e.availability=await invoke('get_pos_package_availability',{branchId:branch,productPackageId:pkg});expect(e.availability.availableQuantity).toBe(6);save();
  await add();await page.getByLabel('Sale notes').fill('DEV PHASE4 controlled sale quantity 1');await capture();
  await page.getByRole('button',{name:'Payment / Post',exact:true}).click();e.saleIntent=true;save();await page.getByRole('button',{name:'Confirm Post Sale',exact:true}).click();await expect(page.getByRole('heading',{name:/^SAL-/})).toBeVisible();
  e.postRequest=(await captured('post_sale')).input;e.sale=await invoke('get_canonical_sale',{id:e.postRequest.id});save();
  expect(e.sale.status).toBe('POSTED');expect(e.sale.items).toHaveLength(1);expect(e.sale.payments).toHaveLength(1);expect(e.sale.items[0].quantity).toBe(1);expect(e.sale.items[0].allocations).toHaveLength(1);expect(await stock(branch)).toBe(5);
  const retry=await invoke('post_sale',{input:e.postRequest});expect(retry.saleNumber).toBe(e.sale.saleNumber);expect(retry.items).toEqual(e.sale.items);expect(await stock(branch)).toBe(5);
  await page.getByRole('button',{name:'Receipt Preview'}).click();await expect(page.getByRole('dialog')).toContainText(e.sale.saleNumber);await page.screenshot({path:'docs/phase-4/receipt-native.png'});e.salePassed=true;
 }else if(stage==='hold'){
  if(e.holdIntent)throw new Error('Held cart already attempted; do not duplicate.');expect(await stock(branch)).toBe(5);await add();await page.getByLabel('Sale notes').fill('DEV PHASE4 durable held cart only');await capture();e.holdIntent=true;save();await page.getByRole('button',{name:'Hold Sale'}).click();await expect(page.getByText('Held sale persisted. No stock reserved.')).toBeVisible();const input=(await captured('hold_sale')).input;e.held=await invoke('get_canonical_sale',{id:input.id});expect(e.held.status).toBe('HELD');expect(await stock(branch)).toBe(5);e.holdPassed=true;
 }else if(stage==='restart-sale'){
  expect(await stock(branch)).toBe(5);const sale=await invoke('get_canonical_sale',{id:e.sale.id});expect(sale.saleNumber).toBe(e.sale.saleNumber);expect(sale.items).toEqual(e.sale.items);const held=await invoke('get_canonical_sale',{id:e.held.id});expect(held.cart).toEqual(e.held.cart);expect(held.status).toBe('HELD');
  await route('/sales/history');await expect(page.getByRole('table',{name:'Canonical sales history'})).toContainText(e.sale.saleNumber);
  await route('/sales/'+e.held.id);await page.getByRole('button',{name:'Resume Cart'}).click();await expect(page.getByLabel('Quantity '+pkg,{exact:true})).toHaveValue('1');await expect(page.getByLabel('Sale notes')).toHaveValue('DEV PHASE4 durable held cart only');await page.getByRole('button',{name:'Hold Sale'}).click();await expect(page.getByText('Held sale persisted. No stock reserved.')).toBeVisible();e.heldAfterResume=await invoke('get_canonical_sale',{id:e.held.id});expect(e.heldAfterResume.cart).toEqual(e.held.cart);expect(await stock(branch)).toBe(5);e.restartSalePassed=true;
 }else if(stage==='return'){
  if(e.returnIntent)throw new Error('Return already attempted; inspect before retry.');await route('/sales/'+e.sale.id);await page.getByRole('button',{name:'Return Items'}).click();await page.getByLabel('Return quantity '+e.sale.items[0].productNameSnapshot).fill('1');await page.getByLabel('Return reason').fill('DEV PHASE4 controlled return of original unit');await page.getByRole('button',{name:'Calculate Refund'}).click();await expect(page.getByText(/Validated refund:/)).toBeVisible();await capture();e.returnIntent=true;save();await page.getByRole('button',{name:'Confirm Return'}).click();await expect(page.getByRole('dialog')).toHaveCount(0);e.returnRequest=(await captured('post_sale_return')).input;e.returned=await invoke('get_canonical_sale',{id:e.sale.id});save();
  expect(e.returned.refundedMinor).toBe(e.sale.totalMinor);expect(e.returned.returns).toHaveLength(1);expect(e.returned.returns[0].status).toBe('POSTED');expect(e.returned.returns[0].allocations[0].originalAllocationId).toBe(e.sale.items[0].allocations[0].id);expect(await stock(branch)).toBe(6);const retry=await invoke('post_sale_return',{input:e.returnRequest});expect(retry.returns).toEqual(e.returned.returns);expect(await stock(branch)).toBe(6);e.returnPassed=true;
 }else if(stage==='final'){
  expect(await stock(branch)).toBe(6);const sale=await invoke('get_canonical_sale',{id:e.sale.id});expect(sale.items).toEqual(e.returned.items);expect(sale.returns).toEqual(e.returned.returns);expect(sale.saleNumber).toBe(e.sale.saleNumber);const held=await invoke('get_canonical_sale',{id:e.held.id});expect(held.status).toBe('HELD');expect(held.cart).toEqual(e.held.cart);
  await route('/sales/history');await expect(page.getByRole('table',{name:'Canonical sales history'})).toContainText(e.sale.saleNumber);await page.screenshot({path:'docs/phase-4/history-native.png'});
  for(const [width,height]of [[1366,768],[1440,900],[1920,1080]]){await page.setViewportSize({width,height});await route('/sales');await add();await expect(page.getByRole('button',{name:'Payment / Post',exact:true})).toBeInViewport();expect(await page.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth)).toBe(true);await page.screenshot({path:`docs/phase-4/native-${width}.png`});}
  await route('/catalog');await expect(page.getByText('25061 results · local SQLite')).toBeVisible();await route('/inventory');await expect(page.getByRole('table',{name:'Canonical inventory overview'})).toContainText('6');e.finalPassed=true;
 }
 expect(e.errors).toEqual([]);save();console.log(JSON.stringify({stage,passed:true,errors:e.errors}));
}catch(error){e.failure={stage,error:String(error)};try{e.lastCalls=await page.evaluate(()=>window.phase4Calls||[]);}catch{/* preserve original error */}save();throw error;}finally{await browser.close();}
