"""Read-only Phase 3C comparison with the verified pre-phase online backup."""
import hashlib
import json
import os
import sqlite3
from pathlib import Path

root = Path(__file__).resolve().parents[1]
baseline = json.loads((root / 'docs/phase-3c-before.json').read_text(encoding='utf-8'))
backup_path = Path(baseline['backupPath'])
database = Path(os.environ['LOCALAPPDATA']) / 'com.m2health.pharmacy/m2-health.db'
before = sqlite3.connect(backup_path.as_uri() + '?mode=ro', uri=True)
after = sqlite3.connect(database.as_uri() + '?mode=ro', uri=True)
tables = [r[0] for r in before.execute("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")]
unchanged = {}
for table in tables:
    if table in ['inventory_movements', '_schema_version']:
        continue
    old = before.execute(f'SELECT * FROM "{table}" ORDER BY 1').fetchall()
    new = after.execute(f'SELECT * FROM "{table}" ORDER BY 1').fetchall()
    unchanged[table] = old == new
old_movements = before.execute('SELECT * FROM inventory_movements ORDER BY id').fetchall()
original_movements_intact = all(after.execute('SELECT * FROM inventory_movements WHERE id=?', (row[0],)).fetchone() == row for row in old_movements)
old_versions = before.execute('SELECT * FROM _schema_version ORDER BY version').fetchall()
versions = after.execute('SELECT * FROM _schema_version ORDER BY version').fetchall()
new_tables = ['stock_count_sessions', 'stock_count_lines', 'stock_count_results', 'stock_count_completion_requests', 'stock_count_sequences']
all_tables = tables + new_tables
def records(sql):
    cursor = after.execute(sql)
    names = [column[0] for column in cursor.description]
    return [dict(zip(names, row)) for row in cursor.fetchall()]
source_hash = hashlib.sha256((root / 'src/data/karem/egyptian-drugs.csv').read_bytes()).hexdigest()
result = {
    'integrity': after.execute('PRAGMA integrity_check').fetchone()[0],
    'foreignKeyErrors': after.execute('PRAGMA foreign_key_check').fetchall(),
    'versions': versions, 'priorVersionsUnchanged': versions[:5] == old_versions,
    'unchangedBaselineTables': unchanged, 'originalMovementsUnchanged': original_movements_intact,
    'csvUnchanged': source_hash == baseline['sourceHash'], 'csvHash': source_hash,
    'countsBefore': {t: before.execute(f'SELECT count(*) FROM "{t}"').fetchone()[0] for t in tables},
    'countsAfter': {t: after.execute(f'SELECT count(*) FROM "{t}"').fetchone()[0] for t in all_tables},
    'stockByPackage': records('SELECT branch_id,product_package_id,sum(quantity_delta) quantity FROM inventory_movements GROUP BY branch_id,product_package_id'),
    'sessions': records('SELECT * FROM stock_count_sessions'),
    'lines': records('SELECT * FROM stock_count_line_view'),
    'requests': records('SELECT * FROM stock_count_completion_requests'),
    'corrections': records("SELECT m.*,l.session_id FROM inventory_movements m JOIN stock_count_lines l ON l.id=m.reference_id WHERE m.movement_type='STOCK_COUNT_CORRECTION'"),
}
assert result['integrity'] == 'ok' and not result['foreignKeyErrors']
assert all(unchanged.values()) and original_movements_intact and result['csvUnchanged']
assert result['priorVersionsUnchanged'] and [v[0] for v in versions] == [1, 2, 3, 4, 5, 6]
assert len(result['sessions']) == 1 and result['sessions'][0]['status'] == 'COMPLETED'
assert len(result['corrections']) == 1 and result['corrections'][0]['quantity_delta'] == 1
assert result['corrections'][0]['branch_id'] == '461dd52b-1136-409b-a019-9f5429b4ed09'
assert result['countsAfter']['inventory_movements'] == len(old_movements) + 1
(root / 'docs/phase-3c-after.json').write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding='utf-8')
print(json.dumps({k: result[k] for k in ['integrity', 'foreignKeyErrors', 'priorVersionsUnchanged', 'unchangedBaselineTables', 'originalMovementsUnchanged', 'csvUnchanged', 'countsAfter', 'stockByPackage']}, indent=2))
