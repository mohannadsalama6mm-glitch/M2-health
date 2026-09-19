/* global window, console, URL */
// Explicit single-session DEV smoke. Never automatically repeats creation or completion writes.
import { chromium, expect } from '@playwright/test';
import fs from 'node:fs';
import process from 'node:process';
const stage = process.argv[2];
const output = new URL('../docs/phase-3c-native-smoke.json', import.meta.url);
const evidence = fs.existsSync(output) ? JSON.parse(fs.readFileSync(output,'utf8')) : { errors: [] };
const browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
const page = browser.contexts()[0].pages()[0];page.setDefaultTimeout(12000);
page.on('pageerror',e=>evidence.errors.push(String(e)));
page.on('console',m=>{if(m.type()==='error')evidence.errors.push(m.text());});
const branchId='461dd52b-1136-409b-a019-9f5429b4ed09';
const mainBranch='f319a917-95d0-47bf-b007-ab76b7c044ae';
const productPackageId='40bad73e-196c-4a7e-81e7-b6dfb90e64af';
const invoke=(command,args)=>page.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
const stock=async(branch=branchId)=>(await invoke('get_package_stock',{branchId:branch,productPackageId})).quantity;
const save=()=>fs.writeFileSync(output,JSON.stringify(evidence,null,2));
const detail=()=>invoke('get_count_session',{id:evidence.sessionId,limit:20,offset:0});
try{
 if(stage==='count'){
  if(evidence.sessionId)throw new Error('A count session already exists in smoke evidence; inspect it instead of creating another.');
  await page.goto('http://127.0.0.1:1420/dashboard');
  await page.getByRole('combobox',{name:'Branch',exact:true}).selectOption(branchId);
  evidence.beforeStock=await stock();expect(evidence.beforeStock).toBe(1);expect(await stock(mainBranch)).toBe(8);
  await page.getByRole('link',{name:'Inventory',exact:true}).click();
  await page.getByRole('link',{name:'Stock count sessions',exact:true}).click();
  await page.getByRole('button',{name:'New count',exact:true}).click();
  await page.getByRole('textbox',{name:'Count notes',exact:true}).fill('DEV PHASE3C 20260914 controlled count: verify 1 system unit versus 2 physical package units.');
  await page.getByRole('button',{name:'Create draft count'}).click();
  await expect(page.getByText('Draft count — stock unchanged')).toBeVisible();
  evidence.sessionId=page.url().split('/').at(-1);evidence.draft=await detail();save();expect(await stock()).toBe(1);
  await page.getByRole('button',{name:'Start count',exact:true}).click();
  await expect(page.getByRole('button',{name:'Review variances'})).toBeVisible();
  evidence.started=await detail();save();expect(evidence.started.lines).toHaveLength(1);
  const line=evidence.started.lines[0];expect(line.expectedQuantity).toBe(1);expect(line.countedQuantity).toBeNull();expect(await stock()).toBe(1);
  await expect(page.getByLabel(`Physical count ${line.id}`,{exact:true})).toHaveValue('');
  await page.getByLabel(`Physical count ${line.id}`,{exact:true}).fill('2');
  await page.getByLabel(`Physical count ${line.id}`,{exact:true}).press('Enter');
  await expect(page.getByLabel('Count summary')).toContainText('1 counted');
  evidence.entered=await detail();save();expect(await stock()).toBe(1);
  await page.getByRole('button',{name:'Review variances'}).click();
  await expect(page.getByRole('button',{name:'Complete count',exact:true})).toBeEnabled();
  evidence.review=await detail();save();expect(evidence.review.lines[0]).toMatchObject({expectedQuantity:1,currentLive:1,countedQuantity:2,correctionDelta:1});
  await page.screenshot({path:'docs/phase-3c-native-review.png',fullPage:true});
  await page.getByRole('button',{name:'Complete count',exact:true}).click();
  await expect(page.getByRole('dialog')).toContainText('exactly 1 correction movements');
  await page.getByRole('button',{name:'Confirm count completion'}).click();
  await expect(page.getByText('Completed count is read-only')).toBeVisible();
  evidence.completed=await detail();evidence.finalStock=await stock();save();expect(evidence.finalStock).toBe(2);expect(await stock(mainBranch)).toBe(8);
  const movements=await invoke('list_stock_movements',{query:{branchId,productPackageId,limit:20,offset:0}});
  evidence.correction=movements.find(m=>m.id===evidence.completed.lines[0].movementId);expect(evidence.correction).toMatchObject({movementType:'STOCK_COUNT_CORRECTION',quantityDelta:1,referenceType:'stock_count_line',referenceId:line.id});
 }else if(stage==='restart'){
  await page.goto(`http://127.0.0.1:1420/inventory/count-sessions/${evidence.sessionId}`);
  await expect(page.getByText('Completed count is read-only')).toBeVisible();
  const d=await detail();expect(d.session.status).toBe('COMPLETED');expect(d.lines).toEqual(evidence.completed.lines);
  const requestId=await page.evaluate(({id,token})=>window.localStorage.getItem(`m2:count:${id}:${token}`),{id:evidence.sessionId,token:evidence.review.reviewToken});expect(requestId).toBeTruthy();
  const retry=await invoke('complete_count_session',{input:{sessionId:evidence.sessionId,expectedRevision:evidence.review.session.revision,requestId,reviewToken:evidence.review.reviewToken}});expect(retry.lines).toEqual(d.lines);
  expect(await stock()).toBe(2);expect(await stock(mainBranch)).toBe(8);
  await page.screenshot({path:'docs/phase-3c-native-completed.png',fullPage:true});
  const phase3b=JSON.parse(fs.readFileSync(new URL('../docs/phase-3b-native-smoke.json',import.meta.url),'utf8'));
  for(const stage of Object.values(phase3b.stages)){const document=await invoke('get_inventory_document',{id:stage.posted.id});expect(document.status).toBe('POSTED');expect(document.movements).toEqual(stage.posted.movements);}
  await page.getByRole('link',{name:'Products',exact:true}).click();await expect(page.getByText('25061 results · local SQLite')).toBeVisible({timeout:20000});
  evidence.restart={session:d.session,lines:d.lines,originalKeyRetry:true,devStock:await stock(),mainStock:await stock(mainBranch),catalogActiveCount:25061,phase3bDocumentsPreserved:true};
 }else throw new Error('Choose count or restart.');
 expect(evidence.errors).toEqual([]);save();console.log(`${stage}: PASS`);
}catch(error){evidence.failure={stage,message:String(error),url:page.url(),main:await page.locator('main').innerText().catch(()=>'')};save();throw error;}
finally{await browser.close();}
