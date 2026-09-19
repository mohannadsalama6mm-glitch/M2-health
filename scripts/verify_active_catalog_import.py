"""Verify real import against the pre-task online backup. SQLite is read-only."""
import hashlib
import json
import os
from pathlib import Path
import sqlite3

root=Path(__file__).resolve().parents[1]
folder=root/'docs/catalog-active-source'
baseline=json.loads((folder/'before.json').read_text(encoding='utf8'))
before=sqlite3.connect(Path(baseline['backupPath']).as_uri()+'?mode=ro',uri=True)
after=sqlite3.connect((Path(os.environ['LOCALAPPDATA'])/'com.m2health.pharmacy/m2-health.db').as_uri()+'?mode=ro',uri=True)
after.execute('PRAGMA query_only=ON');after.execute('BEGIN')
tables=[r[0] for r in before.execute("SELECT name FROM sqlite_master WHERE type='table'")]
def records(c,sql):
    q=c.execute(sql);names=[x[0] for x in q.description];return [dict(zip(names,row)) for row in q]
def rows(c,table):return c.execute('SELECT * FROM "'+table.replace('"','""')+'" ORDER BY 1,2').fetchall()
unchanged={t:rows(before,t)==rows(after,t) for t in tables if t not in ['_schema_version','catalog_import_runs']}
# This source resolves entirely to existing products; any Catalog or operational
# modification is unexpected for this controlled apply.
assert all(unchanged.values()),str({t:ok for t,ok in unchanged.items() if not ok})
old_runs=rows(before,'catalog_import_runs')
for old in old_runs:assert after.execute('SELECT * FROM catalog_import_runs WHERE id=?',(old[0],)).fetchone()==old
new_runs=records(after,'SELECT r.*,s.source_version,s.reused_rows,s.updated_rows,s.malformed_rows,s.quoting_rows,s.ambiguous_rows,s.duplicate_rows,s.invalid_price_rows,s.failed_rows,s.backup_at,s.backup_size FROM catalog_import_runs r JOIN catalog_source_runs s ON s.run_id=r.id')
assert len(new_runs)==1 and new_runs[0]['status']=='complete',new_runs
run=new_runs[0]
assert run['source_hash']=='5f31d97c3491dd75bae780bba8ce52bb8e33bf81a7f0f085884159f08d38de41'
report=json.loads(Path(run['report_path']).read_text(encoding='utf8'))
actions=records(after,'SELECT action,count(*) count FROM catalog_source_rows GROUP BY action ORDER BY action')
assert sum(r['count'] for r in actions)==25088
assert run['imported_rows']==0 and run['updated_rows']==0 and run['failed_rows']==0
assert run['reused_rows']==report['reused']
versions=rows(after,'_schema_version');assert versions[:6]==rows(before,'_schema_version')
assert [(v[0],v[1]) for v in versions][-1]==(7,'catalog_source_actions')
integrity=after.execute('PRAGMA integrity_check').fetchall();foreign=after.execute('PRAGMA foreign_key_check').fetchall()
assert integrity==[('ok',)] and not foreign
sha=hashlib.sha256((root/'src/data/karem/egyptian-drugs.csv').read_bytes()).hexdigest();assert sha==run['source_hash']
all_tables=[r[0] for r in after.execute("SELECT name FROM sqlite_master WHERE type='table'")]
result={'integrity':'ok','foreignKeyErrors':foreign,'unchangedTables':unchanged,'historicalRunsUnchanged':True,
        'versions':versions,'sourceHash':sha,'sourceUnchanged':True,'run':run,'actions':actions,
        'counts':{t:after.execute('SELECT count(*) FROM "'+t+'"').fetchone()[0] for t in all_tables},
        'stock':records(after,'SELECT b.code,m.product_package_id,sum(m.quantity_delta) quantity FROM inventory_movements m JOIN branches b ON b.id=m.branch_id GROUP BY b.code,m.product_package_id'),
        'reportPath':run['report_path']}
result['distinctReusedLookups']={name:after.execute(
    'SELECT count(DISTINCT p.'+column+') FROM catalog_source_rows a JOIN products p ON p.id=a.product_id WHERE a.action=\'reused\' AND a.run_id=?',
    (run['id'],)).fetchone()[0] for name,column in [('manufacturers','manufacturer_id'),('categories','category_id'),('routes','route_id')]}
(folder/'after.json').write_text(json.dumps(result,indent=2,ensure_ascii=False),encoding='utf8')
(folder/'apply-report.json').write_text(json.dumps(report,indent=2,ensure_ascii=False),encoding='utf8')
(folder/'skipped-rows.json').write_text(json.dumps([
    {'sourceRow':r['number'],'reason':r['disposition'],'warnings':r['warnings'],'originalFields':r['fields'],'fingerprint':r['fingerprint']}
    for r in report['rows'] if r['disposition'].startswith('skipped_')],indent=2,ensure_ascii=False),encoding='utf8')
print(json.dumps(result,indent=2,ensure_ascii=True))
