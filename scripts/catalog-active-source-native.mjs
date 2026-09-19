/* global window, console, URL */
// Exactly one authorized apply; verify/restart stages only read Catalog and stock.
import { chromium, expect } from '@playwright/test';
import fs from 'node:fs';
import process from 'node:process';
const stage=process.argv[2];
const output=new URL('../docs/catalog-active-source/native.json',import.meta.url);
const evidence=fs.existsSync(output)?JSON.parse(fs.readFileSync(output,'utf8')):{errors:[]};
const save=()=>fs.writeFileSync(output,JSON.stringify(evidence,null,2));
const browser=await chromium.connectOverCDP('http://127.0.0.1:9223');
const page=browser.contexts()[0].pages()[0];page.setDefaultTimeout(25000);
page.on('pageerror',e=>evidence.errors.push(String(e)));
page.on('console',m=>{if(m.type()==='error')evidence.errors.push(m.text());});
const invoke=(command,args)=>page.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
const list=query=>invoke('list_products',{query:{limit:10,offset:0,...query}});
try {
  await page.goto('http://127.0.0.1:1420/catalog');
  await expect(page.getByRole('heading',{name:'Products',exact:true})).toBeVisible();
  if(stage==='apply'){
    if(evidence.applyAttempted)throw new Error('This controlled apply was already attempted. Inspect persisted results; do not apply again.');
    const previous=await invoke('get_catalog_import_report');
    if(previous?.profile?.importPolicy==='conservative_catalog_v2' && previous.status==='complete')throw new Error('An active-source run already exists. Use verification only.');
    const profile=await invoke('profile_catalog_source');
    expect(profile.hash).toBe('5f31d97c3491dd75bae780bba8ce52bb8e33bf81a7f0f085884159f08d38de41');
    expect(profile.applyAllowed).toBe(true);expect(profile.totalRows).toBe(25088);
    evidence.beforeCatalog=await list({includeInactive:true});
    const plan=await invoke('dry_run_catalog_import');evidence.plan=plan;save();
    expect(plan.counts.ready).toBeGreaterThan(24000);
    expect(plan.counts.ready_insert??0).toBeLessThan(100);
    evidence.applyAttempted=true;evidence.applyCommandCalls=1;save();
    await invoke('apply_catalog_import',{planId:plan.id,confirmed:true});
    await expect.poll(async()=>{const r=await invoke('get_catalog_import_status');return r?.status;},{timeout:300000,intervals:[1000,2000]}).toMatch(/^(complete|failed)$/);
    evidence.apply=await invoke('get_catalog_import_status');save();expect(evidence.apply.status).toBe('complete');
    evidence.idempotency=await invoke('dry_run_catalog_import');save();
    expect(evidence.idempotency.counts.ready_insert??0).toBe(0);
    expect(evidence.idempotency.counts.ready_update??0).toBe(0);
    expect(evidence.idempotency.counts.already_imported).toBe(evidence.apply.reused+evidence.apply.imported+evidence.apply.updated);
  } else if(stage==='verify'||stage==='restart') {
    if(!evidence.apply?.id)throw new Error('Missing completed apply evidence.');
    if(stage==='restart'){
      const report=await invoke('get_catalog_import_report');expect(report.id).toBe(evidence.apply.id);expect(report.status).toBe('complete');
      evidence.persistedRun={id:report.id,reused:report.reused,imported:report.imported,status:report.status};
    }
    const all=await list({includeInactive:true});
    expect(all.total).toBe(evidence.beforeCatalog.total+evidence.apply.imported);
    const english=await list({search:'PANADOL'});expect(english.total).toBeGreaterThan(0);
    const sample=english.items[0];const detail=await invoke('get_product_detail',{id:sample.id});
    const searches={english:'PANADOL',arabic:sample.nameAr,scientific:sample.scientificName,manufacturer:detail.manufacturer.name};
    const search=page.getByRole('searchbox',{name:'Search Products',exact:true});
    const results={};
    for(const [key,value] of Object.entries(searches)){
      expect(value).toBeTruthy();await search.fill(value);
      const data=await list({search:value});expect(data.total).toBeGreaterThan(0);
      await expect(page.getByText(`${data.total} results · local SQLite`,{exact:true})).toBeVisible();
      results[key]={query:value,total:data.total};
    }
    await search.fill('');
    for(const [label,field,id] of [['Manufacturer','manufacturerId',detail.manufacturer.id],['Category','categoryId',detail.category.id],['Route','routeId',detail.route.id]]){
      await page.getByRole('combobox',{name:label,exact:true}).selectOption(id);
      const data=await list({[field]:id});expect(data.total).toBeGreaterThan(0);
      await expect(page.getByText(`${data.total} results · local SQLite`,{exact:true})).toBeVisible();
      results[label]={id,total:data.total};await page.getByRole('combobox',{name:label,exact:true}).selectOption('');
    }
    const next=page.getByRole('button',{name:'Next page',exact:true});await expect(next).toBeEnabled();await next.click();
    const second=await list({offset:10});await expect(page.getByRole('link',{name:`View ${second.items[0].nameEn}`,exact:true})).toBeVisible();
    await search.fill(sample.nameEn);await page.getByRole('link',{name:`View ${sample.nameEn}`,exact:true}).click();
    await expect(page.getByRole('heading',{name:sample.nameEn,exact:true})).toBeVisible();
    const current=detail.packages.find(p=>p.package.isDefault).currentPrice;
    expect(current.sellingPriceMinor).toBe(sample.sellingPriceMinor);
    await page.getByRole('tab',{name:'Packages & barcodes',exact:true}).click();
    await expect(page.locator('main')).toContainText((current.sellingPriceMinor/100).toFixed(2));
    await page.screenshot({path:`docs/catalog-active-source/catalog-${stage}.png`,fullPage:true});
    const productPackageId='40bad73e-196c-4a7e-81e7-b6dfb90e64af';
    const main=await invoke('get_package_stock',{branchId:'f319a917-95d0-47bf-b007-ab76b7c044ae',productPackageId});
    const dev=await invoke('get_package_stock',{branchId:'461dd52b-1136-409b-a019-9f5429b4ed09',productPackageId});expect(main.quantity).toBe(8);expect(dev.quantity).toBe(2);
    const count=await invoke('get_count_session',{id:'c668e88d-94ed-49ed-bd55-841fe6402cbd',limit:20,offset:0});expect(count.session.status).toBe('COMPLETED');
    const phase3b=JSON.parse(fs.readFileSync(new URL('../docs/phase-3b-native-smoke.json',import.meta.url),'utf8'));
    for(const v of Object.values(phase3b.stages)){const doc=await invoke('get_inventory_document',{id:v.posted.id});expect(doc.movements).toEqual(v.posted.movements);}
    evidence[stage]={searches:results,pagination:true,productDetails:true,currentPriceMinor:current.sellingPriceMinor,total:all.total,mainStock:main.quantity,devStock:dev.quantity,countStatus:count.session.status,documentsPreserved:true};
  } else throw new Error('Choose apply, verify or restart.');
  expect(evidence.errors).toEqual([]);evidence.lastError=null;save();console.log(JSON.stringify({stage,runId:evidence.apply?.id,status:evidence.apply?.status,checks:stage==='apply'?{reused:evidence.apply.reused,imported:evidence.apply.imported,updated:evidence.apply.updated}:evidence[stage]},null,2));
} catch(e){evidence.lastError=String(e);save();throw e;} finally {await browser.close();}
