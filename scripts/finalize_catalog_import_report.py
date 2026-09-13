"""Build a read-only verification summary from a completed import and its backup."""
import hashlib
import json
import sqlite3
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parents[1]
before = json.loads((root / 'docs/phase-2d-before.json').read_text(encoding='utf-8'))
after = json.loads((root / 'docs/phase-2d-restart.json').read_text(encoding='utf-8'))
native = json.loads((root / 'docs/phase-2d-native-verification.json').read_text(encoding='utf-8'))
run = after['runs'][0]
report_path = Path(run['report_path'])
report = json.loads(report_path.read_text(encoding='utf-8'))
backup_path = Path(run['backup_path'])
backup = sqlite3.connect(backup_path.as_uri() + '?mode=ro', uri=True)
db = sqlite3.connect(Path(after['database']).as_uri() + '?mode=ro', uri=True)
backup_counts = {t: backup.execute(f'SELECT count(*) FROM {t}').fetchone()[0] for t in before['counts']}
assert backup_counts == before['counts']
assert backup.execute('PRAGMA integrity_check').fetchone()[0] == 'ok'
for table in before['counts']:
    for old in backup.execute(f'SELECT * FROM {table}'):
        assert db.execute(f'SELECT * FROM {table} WHERE id=?', [old[0]]).fetchone() == old
ledger = {r[0] for r in db.execute('SELECT fingerprint FROM catalog_import_rows')}
dispositions = Counter(r['disposition'] for r in report['rows'])
review = [r for r in report['rows'] if r['disposition'] in ['possible_duplicate', 'ambiguous_price']]
assert len(review) == 5 and all(r['fingerprint'] not in ledger for r in review)
assert dispositions == {'imported': 25061, 'exact_duplicate_source': 4, 'possible_duplicate': 4, 'ambiguous_price': 1}
assert len(ledger) == 25061
assert after['counts']['catalog_import_runs'] == 1
assert native['secondDry']['counts']['already_imported'] == 25061
assert native['secondDry']['counts'].get('ready', 0) == 0
config = json.loads((root / 'config/catalog-source.json').read_text())
source = root / config['path']
source_hash = hashlib.sha256(source.read_bytes()).hexdigest()
assert source_hash == before['sourceHash'] == report['profile']['hash']
summary = {
    'verifiedAtUtc': datetime.now(timezone.utc).isoformat(),
    'status': report['status'],
    'source': {**report['profile'], 'path': str(source)},
    'ready': report['counts']['ready'],
    'importedSourceRows': report['imported'],
    'created': {t: after['counts'][t] - before['counts'].get(t, 0) for t in before['counts']},
    'lookupsCreated': report['lookupCreated'],
    'lookupsReusedFromExistingCatalog': report['lookupReused'],
    'exactDuplicates': 4, 'reviewRequired': 5, 'skipped': 9, 'failed': 0,
    'durationMs': report['durationMs'], 'rowsPerSecond': report['rowsPerSecond'],
    'backup': {'path': str(backup_path), 'sizeBytes': backup_path.stat().st_size,
               'timestampUtc': datetime.fromtimestamp(backup_path.stat().st_mtime, timezone.utc).isoformat(),
               'integrity': 'ok', 'countsMatchBefore': True, 'preexistingRowsUnchanged': True},
    'beforeCounts': before['counts'], 'finalCounts': after['counts'],
    'reportPath': str(report_path), 'reportSizeBytes': report_path.stat().st_size,
    'secondDryRunPath': native['secondDry']['reportPath'],
    'secondDryRunCounts': native['secondDry']['counts'],
    'reviewQueue': review, 'dispositions': dispositions,
    'sourceUnchanged': True, 'branchUnchanged': before['branches'] == after['branches'],
    'schemaVersion': after['schema'][-1]['version'],
}
(root / 'docs/phase-2d-final-summary.json').write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding='utf-8')
print(json.dumps({k:v for k,v in summary.items() if k not in ['reviewQueue','source','beforeCounts','finalCounts','secondDryRunCounts']}, ensure_ascii=False, indent=2))
