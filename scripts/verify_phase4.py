"""Read-only preservation checks and a verified pre-009 backup; never repairs data."""
import hashlib,json,os,sqlite3,sys
from pathlib import Path
from datetime import datetime,timezone
root=Path(__file__).resolve().parents[1];folder=root/'docs/phase-4'
baseline=json.loads((folder/'before.json').read_text(encoding='utf8'))
old=sqlite3.connect(Path(baseline['backupPath']).as_uri()+'?mode=ro',uri=True)
c=sqlite3.connect((Path(os.environ['LOCALAPPDATA'])/'com.m2health.pharmacy/m2-health.db').as_uri()+'?mode=ro',uri=True)
c.execute('PRAGMA query_only=ON');c.execute('BEGIN')
def rows(db,t):return db.execute(f'SELECT * FROM "{t}" ORDER BY 1,2').fetchall()
tables=list(baseline['counts'])
hashes={p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in baseline['hashes']}
assert hashes==baseline['hashes'],'Source or migrations 001–008 changed'
integrity=c.execute('PRAGMA integrity_check').fetchone()[0];fk=c.execute('PRAGMA foreign_key_check').fetchall();assert integrity=='ok' and not fk
if sys.argv[1]=='preapply':
 assert json.loads((folder/'checks.json').read_text())['automatedPassed']
 assert all(rows(old,t)==rows(c,t) for t in tables),'Development state changed; inspect before migration'
 c.rollback()
 path=root/'backups/phase-4'/('preapply-'+datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')+'.sqlite3')
 with sqlite3.connect(path) as b:c.backup(b);assert b.execute('PRAGMA integrity_check').fetchone()[0]=='ok';assert all(rows(old,t)==rows(b,t) for t in tables)
 (folder/'preapply.json').write_text(json.dumps({'backupPath':str(path),'verifiedAgainstBaseline':True,'schemaVersion':8,'hashes':hashes},indent=2),encoding='utf8')
 print(str(path));sys.exit(0)
smoke=json.loads((folder/'native.json').read_text())
assert smoke.get('finalPassed') and smoke.get('returnPassed') and smoke.get('restartSalePassed')
unchanged={t:rows(old,t)==rows(c,t) for t in tables}
allowed={'_schema_version','inventory_movements','inventory_alerts','inventory_alert_history'}
assert all(v for t,v in unchanged.items() if t not in allowed),unchanged
for t in ['inventory_movements','inventory_alert_history']:
 assert set(rows(old,t))<=set(rows(c,t)),t+' lost or altered prior rows'
for row in old.execute("SELECT type,name,tbl_name,sql FROM sqlite_master WHERE name NOT LIKE 'sqlite_%'"):
 assert c.execute('SELECT type,name,tbl_name,sql FROM sqlite_master WHERE name=?',(row[1],)).fetchone()==row,'Prior schema definition changed'
versions=rows(c,'_schema_version');assert versions[:8]==rows(old,'_schema_version');assert len(versions)==9 and versions[8][0:2]==(9,'sales')
alertcols='id,branch_id,alert_type,product_package_id,batch_id,status,fingerprint,first_detected_at,created_at'
assert old.execute(f'SELECT {alertcols} FROM inventory_alerts ORDER BY id').fetchall()==c.execute(f'SELECT {alertcols} FROM inventory_alerts ORDER BY id').fetchall()
stock=dict(c.execute('SELECT branch_id,sum(quantity_delta) FROM inventory_movements GROUP BY branch_id'));assert stock==baseline['stock']
added=list(set(rows(c,'inventory_movements'))-set(rows(old,'inventory_movements')));assert len(added)==2
new_ids=[r[0] for r in added];q=c.execute('SELECT id,branch_id,movement_type,quantity_delta,batch_id,reference_type,reference_id FROM inventory_movements WHERE id IN (?,?)',new_ids);movements=[dict(zip([x[0] for x in q.description],r)) for r in q]
assert all(r['branch_id']=='0fc16eaf-cefa-4576-8201-9b03541036f8' for r in movements)
assert sorted((r['movement_type'],r['quantity_delta']) for r in movements)==[('SALE',-1),('SALE_RETURN',1)]
assert len({r['batch_id'] for r in movements})==1
expected={'sales':2,'sale_items':1,'sale_item_allocations':1,'sale_payments':1,'sale_returns':1,'sale_return_items':1,'sale_return_allocations':1,'sale_refunds':1,'sale_voids':0,'sale_void_allocations':0,'sale_requests':2,'sale_sequences':1}
counts={t:len(rows(c,t)) for t in tables+list(expected)}
assert all(counts[t]==n for t,n in expected.items()),counts
assert dict(c.execute('SELECT status,count(*) FROM sales GROUP BY status'))=={'HELD':1,'POSTED':1}
assert c.execute('SELECT amount_minor FROM sale_payments').fetchone()[0]==c.execute('SELECT amount_minor FROM sale_refunds').fetchone()[0]==smoke['sale']['totalMinor']
result={'integrity':integrity,'foreignKeyViolations':fk,'versions':versions,'countsBefore':baseline['counts'],'countsAfter':counts,'unchangedTables':unchanged,'priorSchemaPreserved':True,'priorLedgerAndAlertHistoryPreserved':True,'alertIdentityAndStatusPreserved':True,'hashes':hashes,'stock':stock,'newMovements':movements,'nativePassed':True}
(folder/'after.json').write_text(json.dumps(result,indent=2,ensure_ascii=False),encoding='utf8');print(json.dumps(result,indent=2))
