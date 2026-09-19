/* global window, console, URL */
// Explicit controlled DEV smoke only. Each stage is invoked separately; no automatic retries of writes.
import { chromium, expect } from '@playwright/test';
import fs from 'node:fs';
import process from 'node:process';
const stage = process.argv[2];
const output = new URL('../docs/phase-3b-native-smoke.json', import.meta.url);
const evidence = fs.existsSync(output) ? JSON.parse(fs.readFileSync(output, 'utf8')) : { stages: {}, errors: [] };
const browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
const page = browser.contexts()[0].pages()[0];
page.setDefaultTimeout(12000);
page.on('pageerror', e => evidence.errors.push(String(e)));
page.on('console', m => { if (m.type() === 'error') evidence.errors.push(m.text()); });
const branchId = 'f319a917-95d0-47bf-b007-ab76b7c044ae';
const packageId = '40bad73e-196c-4a7e-81e7-b6dfb90e64af';
const productId = '1e49bfb5-878c-458b-bce0-5849485282bf';
const invoke = (command, args) => page.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
const stock = async (branch = branchId) => (await invoke('get_package_stock', { branchId: branch, productPackageId: packageId })).quantity;
const saveEvidence = () => fs.writeFileSync(output, JSON.stringify(evidence, null, 2));
const detail = () => invoke('get_inventory_document', { id: page.url().split('/').at(-1) });
async function newLine(kind, quantity, batch) {
  await page.goto('http://127.0.0.1:1420/inventory/documents');
  await page.getByRole('button', { name: kind, exact: true }).click();
  if (kind === 'Transfer Stock') await page.getByLabel('Destination branch').selectOption(evidence.devBranch.id);
  await page.getByLabel('Reference', { exact: true }).fill(`DEV PHASE3B ${stage.toUpperCase()} 20260914`);
  await page.getByRole('textbox', { name: 'Notes', exact: true }).fill('Controlled Phase 3B development smoke; intentional immutable history.');
  await page.getByLabel('Search packages').fill('1 2 3');
  await page.getByLabel('Product', { exact: true }).selectOption(productId);
  await page.getByLabel('Package', { exact: true }).selectOption(packageId);
  if (batch) await page.getByLabel('Batch', { exact: true }).selectOption(batch);
  else await page.getByLabel('Lot number').fill('DEV-PHASE3B-20260914');
  await page.getByLabel('Quantity', { exact: true }).fill(String(quantity));
  await page.getByLabel('Line reason').fill(`DEV Phase 3B ${stage} smoke`);
  await page.getByRole('button', { name: 'Add line', exact: true }).click();
  await page.getByRole('button', { name: 'Save draft', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Mark ready' })).toBeEnabled();
  evidence.stages[stage] = { draft: await detail(), draftStock: await stock() }; saveEvidence();
}
async function post() {
  await page.getByRole('button', { name: 'Mark ready' }).click();
  await expect(page.getByRole('button', { name: 'Post document', exact: true })).toBeVisible();
  evidence.stages[stage].ready = await detail(); saveEvidence();
  await page.getByRole('button', { name: 'Post document', exact: true }).click();
  await page.getByRole('button', { name: 'Confirm', exact: true }).click();
  await expect(page.getByText('Posted document is read-only')).toBeVisible();
  evidence.stages[stage].posted = await detail(); evidence.stages[stage].postedStock = await stock(); saveEvidence();
}
try {
  if (new URL(page.url()).hostname !== '127.0.0.1') await page.goto('http://127.0.0.1:1420/dashboard');
  if (stage === 'inspect') {
    await page.reload(); await expect(page.getByRole('heading', { name: /Good morning/ })).toBeVisible();
    evidence.initialStock = await stock(); expect(evidence.initialStock).toBe(8);
    evidence.capabilities = await invoke('recovery_status'); console.log(JSON.stringify(evidence));
  } else if (stage === 'receipt') {
    if (evidence.stages.receipt) throw new Error('Receipt stage already recorded; inspect instead of repeating.');
    expect(await stock()).toBe(8); await newLine('Receive Stock', 2); expect(await stock()).toBe(8); await post(); expect(await stock()).toBe(10);
  } else if (stage === 'adjustment') {
    if (evidence.stages.adjustment) throw new Error('Adjustment already recorded; inspect instead of repeating.');
    expect(await stock()).toBe(10); await newLine('Adjust Stock', 1, evidence.stages.receipt.posted.movements[0].batchId); expect(await stock()).toBe(10); await post(); expect(await stock()).toBe(9);
  } else if (stage === 'transfer') {
    if (evidence.stages.transfer) throw new Error('Transfer already recorded; inspect instead of repeating.');
    const branches = await invoke('list_branches');
    evidence.devBranch = branches.find(b => b.code === 'DEV3B') || await invoke('create_branch', { input: { code: 'DEV3B', name: 'DEV Phase 3B Transfer Smoke', phone: '', address: 'Controlled development smoke only' } }); saveEvidence();
    expect(await stock()).toBe(9); expect(await stock(evidence.devBranch.id)).toBe(0);
    await newLine('Transfer Stock', 1, evidence.stages.receipt.posted.movements[0].batchId); expect(await stock()).toBe(9); await post(); expect(await stock()).toBe(8); expect(await stock(evidence.devBranch.id)).toBe(1);
    evidence.stages.transfer.destinationStock = await stock(evidence.devBranch.id);
  } else if (stage === 'restart') {
    await page.reload(); evidence.restart = { documents: [], mainStock: await stock(), destinationStock: await stock(evidence.devBranch.id) };
    expect(evidence.restart.mainStock).toBe(8); expect(evidence.restart.destinationStock).toBe(1);
    for (const saved of Object.values(evidence.stages)) {
      const current = await invoke('get_inventory_document', { id: saved.posted.id });
      expect(current.status).toBe('POSTED'); expect(current.movements).toEqual(saved.posted.movements);
      const requestId = await page.evaluate(({ id, revision }) => window.localStorage.getItem(`m2:inventory:post:${id}:${revision}:`), { id: current.id, revision: saved.ready.revision });
      expect(requestId).toBeTruthy();
      const retry = await invoke('post_inventory_document', { input: { documentId: current.id, expectedRevision: saved.ready.revision, requestId } });
      expect(retry.movements).toEqual(current.movements); evidence.restart.documents.push(current.id);
    }
    await page.goto('http://127.0.0.1:1420/inventory/documents'); await expect(page.getByText('3 documents · local SQLite')).toBeVisible();
    await page.screenshot({ path: 'docs/phase-3b-native-documents.png', fullPage: true });
    await page.getByRole('link', { name: 'Products', exact: true }).click();
    await expect(page.getByText('25061 results · local SQLite')).toBeVisible({ timeout: 20000 }); evidence.restart.catalogActiveCount = 25061;
    expect(await stock()).toBe(8); expect(await stock(evidence.devBranch.id)).toBe(1);
  } else throw new Error('Choose inspect, receipt, adjustment, transfer or restart.');
  expect(evidence.errors).toEqual([]); saveEvidence(); console.log(`${stage}: PASS`);
} catch (error) { evidence.failure = { stage, message: String(error), url: page.url(), main: await page.locator('main').innerText().catch(() => '') }; saveEvidence(); throw error; }
finally { await browser.close(); }
