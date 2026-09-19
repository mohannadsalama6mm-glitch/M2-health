"""Render a standalone, offline review of the generated plan; no database access."""
import json
from pathlib import Path

folder = Path(__file__).resolve().parents[1] / 'docs/catalog-v2'
summary = json.loads((folder / 'summary.json').read_text(encoding='utf8'))
plan = json.loads((folder / 'update-plan.json').read_text(encoding='utf8'))
# Raw padded fields and fingerprints stay in the complete machine-readable plan.
rows = [{k: v for k, v in r.items() if k not in ['rawFields', 'sourceHash', 'sourceVersion']}
        for r in plan['rows']]
payload = json.dumps({'summary': summary, 'rows': rows}, ensure_ascii=False).replace('<', '\\u003c')
html = '''<!doctype html><html lang="en"><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>M² Health · Canonical v2 review</title>
<style>
:root{color-scheme:light;--bg:#f4f8f7;--paper:#fff;--ink:#16332e;--muted:#536e67;--line:#d4e2dd;--accent:#007d62}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--ink);font:15px/1.6 system-ui,sans-serif}
main{max-width:1280px;padding:36px 32px;margin:auto}header{border-bottom:2px solid var(--accent);padding-bottom:24px}
.eyebrow{color:var(--accent);font-weight:700;letter-spacing:.12em;font-size:12px}h1{font-size:36px;line-height:1.2;margin:12px 0}h2{font-size:21px;margin:0 0 12px}
p{margin:8px 0}.muted{color:var(--muted)}.status{display:inline-block;padding:4px 12px;background:#d9eee7;color:#005c47;font-weight:700;border-radius:4px}
.stats{display:flex;gap:28px;flex-wrap:wrap;padding:24px 0;border-bottom:1px solid var(--line)}.stat strong{display:block;font-size:32px;line-height:1.2;font-variant-numeric:tabular-nums}
section{margin-top:28px}details{border-bottom:1px solid var(--line);padding:12px 0}summary{cursor:pointer;font-weight:650}
.toolbar{display:flex;gap:12px;flex-wrap:wrap;margin:16px 0}input,select,button{font:inherit;border:1px solid var(--line);border-radius:5px;padding:9px 12px;background:var(--paper);color:var(--ink)}input{flex:1;min-width:200px}button{cursor:pointer}button:hover{border-color:var(--accent)}button:disabled{opacity:.45;cursor:default}
.table-wrap{overflow:auto;background:var(--paper);border:1px solid var(--line)}table{border-collapse:collapse;width:100%;text-align:left}th,td{padding:12px;border-bottom:1px solid var(--line);vertical-align:top}th{background:#eaf2ef;font-size:12px;text-transform:uppercase;letter-spacing:.04em}td:first-child{width:65px}td:nth-child(2){min-width:220px}td:nth-child(3){font-size:12px;white-space:nowrap}td:nth-child(4){font-size:13px;max-width:330px;overflow-wrap:anywhere}
.pager{display:flex;gap:12px;align-items:center;margin:14px 0}.pager span{flex:1}a{color:var(--accent)}pre{white-space:pre-wrap;overflow-wrap:anywhere;font-size:13px;background:#eef4f1;padding:16px;border-radius:4px}dialog{border:1px solid var(--line);border-radius:8px;max-width:900px;width:90%;max-height:85vh;padding:24px}dialog::backdrop{background:#142b2777}.dialog-head{display:flex;justify-content:space-between;gap:20px;align-items:center}.routes{columns:3;list-style:none;padding:0}.routes li{break-inside:avoid}footer{margin-top:32px;color:var(--muted);font-size:13px}@media(max-width:650px){main{padding:20px 16px}h1{font-size:28px}.stats{gap:20px}.stat strong{font-size:26px}.routes{columns:2}}
</style><main>
<header><div class="eyebrow">M² HEALTH / SOURCE ADOPTION</div><h1>Canonical v2 review</h1>
<span class="status">NOT APPLIED</span><p>25,088 source records reviewed. No Catalog or stock changes.</p>
<p class="muted">The approved v2 bytes are preserved. Parsing issues need review before any update policy is approved.</p></header>
<div class="stats" id="stats"></div>
<section><h2>What needs attention</h2><p><b>26 records have the wrong column count.</b> Another 29 have strict quoting issues; 12 more retain ambiguous quoted text. One price uses three decimal places. These 68 records are held for review.</p>
<p>Twelve records form six normalized-name duplicate groups. Four groups are exact repeated rows. No uncertain identities have been merged.</p>
<p>There are <b>zero confirmed price changes or new products</b>. Of 387 literal metadata differences, 348 are shared lookup display variants and 39 require parsing or identity review.</p>
<details><summary>Complete source profile and route distribution</summary><p id="profile"></p><ul class="routes" id="routes"></ul><p class="muted">Field statistics cover 25,062 seven-column records. The 26 malformed records are preserved separately.</p></details>
<details><summary>Matching and preservation policy</summary><p id="policy"></p><p>Historical Phase 2D provenance was verified for all 25,061 imported rows. Schema versions 001–006, Catalog, price history, Inventory documents and the completed count session are preserved. Main stock remains 8; development branch stock remains 2.</p><p>Comparison values trim field-edge whitespace. Raw decoded fields, fingerprints, original line spans, match candidates and field-level deltas are retained in the full JSON plan.</p></details></section>
<section><h2>Row review</h2><p class="muted">Search all source records or select a classification. View a row for exact changes, warnings and candidate identifiers.</p>
<div class="toolbar"><input id="search" aria-label="Search source records" placeholder="Search English / Arabic name or source row…"><select id="filter" aria-label="Classification"><option value="attention">Needs attention (80)</option><option value="all">All records (25,088)</option><option>UNCHANGED</option><option>NEW</option><option>CHANGED</option><option>POSSIBLE_DUPLICATE</option><option>REVIEW_REQUIRED</option></select></div>
<div class="table-wrap"><table><thead><tr><th>Row</th><th>Source name</th><th>Classification</th><th>Review notes</th><th>Details</th></tr></thead><tbody id="body"></tbody></table></div>
<div class="pager"><span id="range" role="status"></span><button id="prev">Previous</button><button id="next">Next</button></div></section>
<footer>Offline review artifact · No apply control or database connection.<br><a href="update-plan.json">Complete JSON update plan</a> · <a href="summary.json">Profile and comparison summary</a> · <a href="REPORT.md">A–Q report</a></footer>
<dialog id="detail"><div class="dialog-head"><h2 id="detail-title">Source record</h2><button id="close">Close</button></div><pre id="detail-body"></pre></dialog>
</main><script id="data" type="application/json">PAYLOAD</script><script>
const data=JSON.parse(document.getElementById('data').textContent), $=id=>document.getElementById(id);
for(const [label,key] of [['Unchanged','UNCHANGED'],['New','NEW'],['Changed','CHANGED'],['Possible duplicates','POSSIBLE_DUPLICATE'],['Review required','REVIEW_REQUIRED']]){const el=document.createElement('div');el.className='stat';const n=document.createElement('strong');n.textContent=(data.summary.classifications[key]||0).toLocaleString();el.append(n,document.createTextNode(label));$('stats').append(el)}
const p=data.summary.profile;$('profile').textContent=`${p.bytes.toLocaleString()} bytes · 7 columns · Missing science: ${p.missing.scientific_name.toLocaleString()} · Missing manufacturer: ${p.missing.manufacturer} · Missing class: ${p.missing.drug_class}. UTF-8 valid; no replacement or NUL characters. Arabic script occurs in ${p.rowsWithArabicScript.toLocaleString()} valid records.`;
for(const [name,n] of Object.entries(p.routes)){const li=document.createElement('li');li.textContent=name+': '+n.toLocaleString();$('routes').append(li)}$('policy').textContent=data.summary.matchingPolicy;
let page=0, matches=[];const size=50;
function render(){const query=$('search').value.trim().toLowerCase(),filter=$('filter').value;matches=data.rows.filter(r=>(filter==='all'||(filter==='attention'?r.classification!=='UNCHANGED':r.classification===filter))&&(!query||String(r.sourceRow)===query||Object.values(r.values).join(' ').toLowerCase().includes(query)));$('body').replaceChildren();for(const r of matches.slice(page*size,(page+1)*size)){const tr=document.createElement('tr');for(const value of [r.sourceRow,r.values.commercial_name_en||'(Malformed fragment)',r.classification,r.warnings.join('; ')||'No update proposed']){const td=document.createElement('td');td.textContent=value;tr.append(td)}const td=document.createElement('td'),button=document.createElement('button');button.textContent='View';button.setAttribute('aria-label','View source row '+r.sourceRow);button.onclick=()=>{$('detail-title').textContent='Source row '+r.sourceRow;$('detail-body').textContent=JSON.stringify(r,null,2);$('detail').showModal()};td.append(button);tr.append(td);$('body').append(tr)}$('range').textContent=matches.length?`${page*size+1}–${Math.min((page+1)*size,matches.length)} of ${matches.length.toLocaleString()} records`:'No records in this classification';$('prev').disabled=page===0;$('next').disabled=(page+1)*size>=matches.length}
for(const id of ['search','filter'])$(id).addEventListener('input',()=>{page=0;render()});$('prev').onclick=()=>{page--;render()};$('next').onclick=()=>{page++;render()};$('close').onclick=()=>$('detail').close();render();
</script></html>'''
(folder / 'review.html').write_text(html.replace('PAYLOAD', payload), encoding='utf8')
print('Rendered docs/catalog-v2/review.html')
