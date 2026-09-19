"""Read-only Phase 3E preservation comparison against its online backup."""
import hashlib
import json
import os
import sqlite3
from pathlib import Path
root = Path(__file__).resolve().parents[1]
folder = root / 'docs/phase-3e'
baseline = json.loads((folder/'before.json').read_text(encoding='utf8'))
before = sqlite3.connect(Path(baseline['backupPath']).as_uri()+'?mode=ro', uri=True)
after = sqlite3.connect((Path(os.environ['LOCALAPPDATA'])/'com.m2health.pharmacy/m2-health.db').as_uri()+'?mode=ro', uri=True)
after.execute('PRAGMA query_only=ON')
after.execute('BEGIN')
def rows(c, table):
    return c.execute(f'SELECT * FROM "{table}" ORDER BY 1,2').fetchall()
tables = [r[0] for r in before.execute("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")]
unchanged = {t:rows(before,t)==rows(after,t) for t in tables}
allowed = {'inventory_batches','inventory_movements','inventory_levels','inventory_alerts'}
assert all(v for t,v in unchanged.items() if t not in allowed), unchanged
assert before.execute("SELECT type,name,tbl_name,sql FROM sqlite_master ORDER BY type,name").fetchall() == after.execute("SELECT type,name,tbl_name,sql FROM sqlite_master ORDER BY type,name").fetchall()
additions = {}
for t in ['inventory_batches','inventory_movements']:
    old=set(rows(before,t));new=set(rows(after,t));assert old<=new
    additions[t]=list(new-old);assert len(additions[t])==1
dev='0fc16eaf-cefa-4576-8201-9b03541036f8'
movement_id=json.loads((folder/'native.json').read_text())['openingMessage'].split(': ')[1]
q=after.execute('SELECT * FROM inventory_movements WHERE id=?',(movement_id,))
movement=dict(zip([c[0] for c in q.description],q.fetchone()))
assert movement['branch_id']==dev and movement['quantity_delta']==1 and movement['movement_type']=='OPENING'
assert movement['reason']=='DEV Phase3E controlled native opening +1'
assert after.execute('SELECT lot_number FROM inventory_batches WHERE id=?',(movement['batch_id'],)).fetchone()[0]=='DEV-PHASE3E-OPENING'
# Level editing intentionally updates updated_at; final configured values must match.
level_columns=[r[1] for r in after.execute('PRAGMA table_info(inventory_levels)') if r[1]!='updated_at']
columns=','.join(level_columns)
assert before.execute(f'SELECT {columns} FROM inventory_levels ORDER BY id').fetchall()==after.execute(f'SELECT {columns} FROM inventory_levels ORDER BY id').fetchall()
# Monitoring refresh may update evaluation timestamps, never identity/status/history.
alert_columns=[r[1] for r in after.execute('PRAGMA table_info(inventory_alerts)') if r[1] not in ['updated_at','last_detected_at']]
columns=','.join(alert_columns)
assert before.execute(f'SELECT {columns} FROM inventory_alerts ORDER BY id').fetchall()==after.execute(f'SELECT {columns} FROM inventory_alerts ORDER BY id').fetchall()
stock=dict(after.execute('SELECT branch_id,sum(quantity_delta) FROM inventory_movements GROUP BY branch_id'))
assert stock=={'f319a917-95d0-47bf-b007-ab76b7c044ae':8,'461dd52b-1136-409b-a019-9f5429b4ed09':2,dev:6}
integrity=after.execute('PRAGMA integrity_check').fetchone()[0]
foreign=after.execute('PRAGMA foreign_key_check').fetchall()
assert integrity=='ok' and not foreign
hashes={p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in ['src/data/karem/egyptian-drugs.csv','config/catalog-source.json','src-tauri/migrations/007_catalog_source_actions.sql']}
assert hashes['src/data/karem/egyptian-drugs.csv']==baseline['sourceHash']
assert hashes['config/catalog-source.json']==baseline['configHash']
assert hashes['src-tauri/migrations/007_catalog_source_actions.sql']=='df5d72f61f6a2c0f98fe3a6ad318ece16df060a491f7efda3f985cd3f7546074'
result={'integrity':integrity,'foreignKeyErrors':foreign,'schemaUnchanged':True,'versions':rows(after,'_schema_version'),'unchangedTables':unchanged,'countsBefore':baseline['counts'],'countsAfter':{t:len(rows(after,t)) for t in tables},'stock':stock,'intentionalOpening':movement,'levelValuesRestored':True,'alertIdentityStatusHistoryPreserved':True,'hashes':hashes}
(folder/'after.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf8')
print(json.dumps(result,indent=2))
