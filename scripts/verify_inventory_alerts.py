"""Read-only comparison of Phase 3D with its pre-migration online backup."""
import hashlib
import json
import os
import sqlite3
from pathlib import Path

root = Path(__file__).resolve().parents[1]
folder = root / 'docs/phase-3d'
baseline = json.loads((folder / 'before.json').read_text(encoding='utf8'))
smoke = json.loads((folder / 'native.json').read_text(encoding='utf8'))
before = sqlite3.connect(Path(baseline['backupPath']).as_uri() + '?mode=ro', uri=True)
after = sqlite3.connect((Path(os.environ['LOCALAPPDATA']) / 'com.m2health.pharmacy/m2-health.db').as_uri() + '?mode=ro', uri=True)
after.execute('PRAGMA query_only=ON')
after.execute('BEGIN')
def rows(c, table):
    return c.execute(f'SELECT * FROM "{table}" ORDER BY 1,2').fetchall()
def records(sql):
    q = after.execute(sql)
    return [dict(zip([x[0] for x in q.description], row)) for row in q]
tables = [r[0] for r in before.execute("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")]
unchanged = {}
preserved = {}
for table in tables:
    old = rows(before, table)
    new = rows(after, table)
    unchanged[table] = old == new
    if table == 'inventory_document_sequences':
        # Existing global numbering counters may advance for the new DEV documents.
        continue
    new_set = set(new)
    preserved[table] = all(row in new_set for row in old)
assert all(preserved.values()), preserved
allowed_growth = {'_schema_version', 'branches', 'inventory_batches', 'inventory_movements', 'inventory_levels', 'inventory_document_sequences', 'inventory_documents', 'inventory_document_lines', 'inventory_document_movements', 'inventory_post_requests'}
assert all(ok for t, ok in unchanged.items() if t not in allowed_growth), unchanged
versions = rows(after, '_schema_version')
assert [(r[0], r[1]) for r in versions] == [(1,'branches'),(2,'catalog_core'),(3,'catalog_import'),(4,'inventory_core'),(5,'inventory_workflows'),(6,'stock_counts'),(7,'catalog_source_actions'),(8,'inventory_alerts')]
branch = smoke['branch']['id']
new_docs = [d for d in records('SELECT * FROM inventory_documents') if d['id'] not in {r[0] for r in rows(before,'inventory_documents')}]
assert len(new_docs) == 3 and all(d['branch_id'] == branch and d['status'] == 'POSTED' for d in new_docs)
expected_sequences = {(r[0],r[1]):r[2] for r in rows(before,'inventory_document_sequences')}
for d in new_docs:
    prefix, day, _number = d['document_number'].split('-')
    key = (day,prefix)
    expected_sequences[key] = expected_sequences.get(key,0) + 1
assert {(r[0],r[1]):r[2] for r in rows(after,'inventory_document_sequences')} == expected_sequences
for table in ['inventory_batches','inventory_movements','inventory_levels']:
    old_ids = {r[0] for r in rows(before, table)}
    additions = [r for r in records(f'SELECT * FROM {table}') if r['id'] not in old_ids]
    assert all(r['branch_id'] == branch for r in additions), table
stock = records('SELECT branch_id,product_package_id,sum(quantity_delta) quantity FROM inventory_movements GROUP BY branch_id,product_package_id ORDER BY branch_id')
quantities = {r['branch_id']:r['quantity'] for r in stock}
assert quantities['f319a917-95d0-47bf-b007-ab76b7c044ae'] == 8
assert quantities['461dd52b-1136-409b-a019-9f5429b4ed09'] == 2
assert quantities[branch] == 5
integrity = after.execute('PRAGMA integrity_check').fetchone()[0]
foreign = after.execute('PRAGMA foreign_key_check').fetchall()
assert integrity == 'ok' and not foreign
source_hash = hashlib.sha256((root/'src/data/karem/egyptian-drugs.csv').read_bytes()).hexdigest()
config_hash = hashlib.sha256((root/'config/catalog-source.json').read_bytes()).hexdigest()
assert source_hash == baseline['sourceHash'] and config_hash == baseline['configHash']
new_tables = ['inventory_alert_settings','inventory_alerts','inventory_alert_history']
alerts = records('SELECT id,branch_id,alert_type,status,fingerprint FROM inventory_alerts ORDER BY id')
assert len(alerts) == 4 and len({a['fingerprint'] for a in alerts}) == 4
assert all(a['branch_id'] == branch for a in alerts)
assert sum(a['status'] == 'ACTIVE' for a in alerts) == 1
result = {'integrity':integrity,'foreignKeyErrors':foreign,'versions':versions,'priorRowsPreserved':preserved,'unchangedTables':unchanged,'documentCountersAdvanceOnlyForSmoke':True,'sourceHash':source_hash,'configHash':config_hash,'csvAndConfigUnchanged':True,'countsBefore':baseline['counts'],'countsAfter':{t:len(rows(after,t)) for t in tables+new_tables},'stockByPackage':stock,'alerts':alerts,'newDocuments':new_docs,'nativeRestartPassed':bool(smoke.get('restart'))}
(folder/'after.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf8')
print(json.dumps({k:result[k] for k in ['integrity','foreignKeyErrors','csvAndConfigUnchanged','countsAfter','stockByPackage','nativeRestartPassed']},indent=2))


