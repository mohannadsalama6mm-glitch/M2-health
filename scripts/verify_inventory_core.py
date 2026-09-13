"""Read-only Inventory/Catalog audit; optional SQLite online backup before migration."""
import argparse
import hashlib
import json
import os
import sqlite3
import uuid
from pathlib import Path

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--output', required=True)
parser.add_argument('--backup', action='store_true')
args = parser.parse_args()
directory = Path(os.environ['LOCALAPPDATA']) / 'com.m2health.pharmacy'
database = directory / 'm2-health.db'
db = sqlite3.connect(database.as_uri() + '?mode=ro', uri=True)
catalog_tables = ['branches','products','product_packages','manufacturers','categories','routes','active_ingredients','product_active_ingredients','barcodes','product_price_history','catalog_import_runs','catalog_import_rows']
inventory_tables = ['inventory_batches','inventory_movements','inventory_transfers','inventory_levels']
existing = {r[0] for r in db.execute("SELECT name FROM sqlite_master WHERE type='table'")}
digest = {}
for table in catalog_tables:
    rows = db.execute(f'SELECT * FROM {table} ORDER BY 1,2').fetchall()
    digest[table] = hashlib.sha256(json.dumps(rows, ensure_ascii=False, separators=(',',':')).encode()).hexdigest()
result = {
    'database': str(database),
    'counts': {t: db.execute(f'SELECT count(*) FROM {t}').fetchone()[0] for t in catalog_tables + inventory_tables if t in existing},
    'catalogDigests': digest,
    'schemaVersions': db.execute('SELECT version,name FROM _schema_version ORDER BY version').fetchall(),
    'sourceHash': hashlib.sha256((root/'src/data/karem/egyptian-drugs.csv').read_bytes()).hexdigest(),
    'priorMigrationHashes': {str(p.name):hashlib.sha256(p.read_bytes()).hexdigest() for p in (root/'src-tauri/migrations').glob('00[123]_*.sql')},
    'inventoryUiHash': hashlib.sha256((root/'src/features/inventory/Inventory.tsx').read_bytes()).hexdigest(),
    'integrity': db.execute('PRAGMA integrity_check').fetchone()[0],
    'foreignKeyErrors': db.execute('PRAGMA foreign_key_check').fetchall(),
}
if 'inventory_movements' in existing:
    result['stockByPackage'] = db.execute('SELECT branch_id,product_package_id,sum(quantity_delta) FROM inventory_movements GROUP BY branch_id,product_package_id').fetchall()
    result['batches'] = db.execute('SELECT * FROM inventory_batches').fetchall()
    result['movements'] = db.execute('SELECT * FROM inventory_movements').fetchall()
if args.backup:
    backup = directory / 'backups' / f'before-phase-3a-{uuid.uuid4()}.sqlite3'
    backup.parent.mkdir(exist_ok=True)
    with sqlite3.connect(backup) as destination:
        db.backup(destination)
        assert destination.execute('PRAGMA integrity_check').fetchone()[0] == 'ok'
    result['backupPath'] = str(backup)
    result['backupBytes'] = backup.stat().st_size
Path(args.output).write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding='utf-8')
print(json.dumps({k:v for k,v in result.items() if k not in ['batches','movements','catalogDigests','priorMigrationHashes']}, ensure_ascii=False, indent=2))
