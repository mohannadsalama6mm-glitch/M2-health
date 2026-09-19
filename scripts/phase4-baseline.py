import hashlib, json, os, sqlite3
from pathlib import Path
from datetime import datetime, timezone
root=Path(__file__).resolve().parents[1]
folder=root/'docs/phase-4';folder.mkdir(exist_ok=True)
assert not (folder/'before.json').exists(), 'Baseline already exists; do not restart.'
dest=root/'backups/phase-4';dest.mkdir(parents=True,exist_ok=True)
path=dest/('before-'+datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')+'.sqlite3')
c=sqlite3.connect((Path(os.environ['LOCALAPPDATA'])/'com.m2health.pharmacy/m2-health.db').as_uri()+'?mode=ro',uri=True)
with sqlite3.connect(path) as b:c.backup(b)
tables=[r[0] for r in c.execute("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")]
data={'backupPath':str(path),'counts':{t:c.execute(f'SELECT count(*) FROM "{t}"').fetchone()[0] for t in tables},'stock':dict(c.execute('SELECT branch_id,sum(quantity_delta) FROM inventory_movements GROUP BY branch_id')),'hashes':{str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [root/'src/data/karem/egyptian-drugs.csv',root/'config/catalog-source.json',*sorted((root/'src-tauri/migrations').glob('00*.sql'))]}}
(folder/'before.json').write_text(json.dumps(data,indent=2),encoding='utf8');print(json.dumps(data,indent=2))
