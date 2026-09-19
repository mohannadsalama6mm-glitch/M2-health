"""Read-only Phase 3B evidence against the saved pre-phase baseline."""
import hashlib
import json
import os
import sqlite3
from pathlib import Path

root = Path(__file__).resolve().parents[1]
before = json.loads((root / 'docs/phase-3b-before.json').read_text(encoding='utf-8'))
path = Path(os.environ['LOCALAPPDATA']) / 'com.m2health.pharmacy/m2-health.db'
db = sqlite3.connect(path.as_uri() + '?mode=ro', uri=True)
db.row_factory = sqlite3.Row
tables = list(before['counts']) + ['inventory_documents', 'inventory_document_lines', 'inventory_document_movements', 'inventory_post_requests']
counts = {t: db.execute(f'SELECT count(*) FROM {t}').fetchone()[0] for t in tables}
preserved = {}
for table, digest in before['catalogDigests'].items():
    if table == 'branches':
        continue  # A single explicitly labelled DEV branch is authorized.
    rows = [tuple(r) for r in db.execute(f'SELECT * FROM {table} ORDER BY 1,2')]
    preserved[table] = hashlib.sha256(json.dumps(rows, ensure_ascii=False, separators=(',', ':')).encode()).hexdigest() == digest
old_rows = {}
for key, table in [('batches', 'inventory_batches'), ('movements', 'inventory_movements')]:
    old_rows[key] = all(tuple(db.execute(f'SELECT * FROM {table} WHERE id=?', (row[0],)).fetchone()) == tuple(row) for row in before[key])
source_hash = hashlib.sha256((root / 'src/data/karem/egyptian-drugs.csv').read_bytes()).hexdigest()
result = {
    'integrity': db.execute('PRAGMA integrity_check').fetchone()[0],
    'foreignKeyErrors': [tuple(r) for r in db.execute('PRAGMA foreign_key_check')],
    'schemaVersions': [dict(r) for r in db.execute('SELECT * FROM _schema_version ORDER BY version')],
    'countsBefore': before['counts'], 'countsAfter': counts,
    'catalogAndImportPreserved': preserved, 'originalInventoryRowsPreserved': old_rows,
    'csvUnchanged': source_hash == before['sourceHash'], 'csvHash': source_hash,
    'branches': [dict(r) for r in db.execute('SELECT * FROM branches ORDER BY created_at')],
    'stockByPackage': [dict(r) for r in db.execute('SELECT branch_id,product_package_id,sum(quantity_delta) quantity FROM inventory_movements GROUP BY branch_id,product_package_id')],
    'documents': [dict(r) for r in db.execute('SELECT * FROM inventory_documents ORDER BY created_at')],
    'documentMovements': [dict(r) for r in db.execute('SELECT dm.*,m.branch_id,m.batch_id,m.movement_type,m.quantity_delta FROM inventory_document_movements dm JOIN inventory_movements m ON m.id=dm.movement_id')],
    'postingRequests': [dict(r) for r in db.execute('SELECT * FROM inventory_post_requests')],
}
assert result['integrity'] == 'ok' and not result['foreignKeyErrors']
assert all(preserved.values()) and all(old_rows.values()) and result['csvUnchanged']
assert [r['version'] for r in result['schemaVersions']] == [1, 2, 3, 4, 5]
(root / 'docs/phase-3b-after.json').write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding='utf-8')
print(json.dumps({k: result[k] for k in ['integrity', 'foreignKeyErrors', 'countsAfter', 'catalogAndImportPreserved', 'originalInventoryRowsPreserved', 'csvUnchanged', 'stockByPackage']}, indent=2))
