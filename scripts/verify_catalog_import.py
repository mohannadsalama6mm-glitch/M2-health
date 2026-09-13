"""Read-only verification of the development Catalog and canonical source."""
import csv
import hashlib
import json
import sqlite3
import os
from pathlib import Path
import argparse

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--output', required=True)
args = parser.parse_args()
config = json.loads((root / 'config/catalog-source.json').read_text())
source = root / config['path']
db = Path(os.environ['LOCALAPPDATA']) / 'com.m2health.pharmacy/m2-health.db'
connection = sqlite3.connect(db.as_uri() + '?mode=ro', uri=True)
connection.row_factory = sqlite3.Row
tables = ['branches', 'products', 'product_packages', 'manufacturers', 'categories', 'routes', 'active_ingredients', 'product_active_ingredients', 'barcodes', 'product_price_history']
existing = {r[0] for r in connection.execute("SELECT name FROM sqlite_master WHERE type='table'")}
tables += [t for t in ['catalog_import_runs', 'catalog_import_rows'] if t in existing]
result = {
    'sourceHash': hashlib.sha256(source.read_bytes()).hexdigest(),
    'sourceBytes': source.stat().st_size,
    'database': str(db),
    'counts': {t: connection.execute(f'SELECT count(*) FROM {t}').fetchone()[0] for t in tables},
    'schema': [dict(r) for r in connection.execute('SELECT * FROM _schema_version ORDER BY version')],
    'branches': [dict(r) for r in connection.execute('SELECT * FROM branches')],
    'integrity': connection.execute('PRAGMA integrity_check').fetchone()[0],
    'foreignKeyErrors': [tuple(r) for r in connection.execute('PRAGMA foreign_key_check')],
}
if 'catalog_import_runs' in existing:
    result['runs'] = [dict(r) for r in connection.execute('SELECT * FROM catalog_import_runs')]
    result['placeholderErrors'] = connection.execute("SELECT count(*) FROM catalog_import_rows i JOIN product_packages p ON p.id=i.package_id WHERE p.package_label!='Default Package' OR p.pack_size IS NOT NULL OR p.unit_name IS NOT NULL OR p.units_per_package IS NOT NULL").fetchone()[0]
    result['importedBarcodeCount'] = connection.execute('SELECT count(*) FROM barcodes b JOIN catalog_import_rows i ON i.package_id=b.product_package_id').fetchone()[0]
    result['samples'] = []
    with source.open(encoding='utf-8-sig', newline='') as f:
        rows = list(csv.DictReader(f))
    predicates = {
        'bilingual_combination': lambda r: '+' in r['scientific_name'],
        'missing_scientific': lambda r: not r['scientific_name'],
        'missing_manufacturer': lambda r: not r['manufacturer'],
        'unknown_route': lambda r: r['route'] == 'UNKNOWN',
        'two_decimals': lambda r: '.' in r['price_egp'] and len(r['price_egp'].split('.')[1]) == 2,
    }
    for label, predicate in predicates.items():
        for source_row in filter(predicate, rows):
            match = connection.execute('''SELECT p.*, m.name manufacturer_name,c.name category_name,r.name route_name,pk.package_label,h.selling_price_minor
                FROM products p JOIN catalog_import_rows i ON i.product_id=p.id
                JOIN product_packages pk ON pk.id=i.package_id
                LEFT JOIN manufacturers m ON m.id=p.manufacturer_id LEFT JOIN categories c ON c.id=p.category_id
                LEFT JOIN routes r ON r.id=p.route_id JOIN product_price_history h ON h.product_package_id=pk.id AND h.effective_to IS NULL
                WHERE p.commercial_name_en=?''', [source_row['commercial_name_en']]).fetchone()
            if match:
                from decimal import Decimal
                assert match['commercial_name_ar'] == source_row['commercial_name_ar']
                assert (match['scientific_name'] or '') == source_row['scientific_name']
                assert (match['manufacturer_name'] or '').split() == source_row['manufacturer'].split()
                assert match['selling_price_minor'] == int(Decimal(source_row['price_egp']) * 100)
                result['samples'].append({'case': label, 'source': source_row, 'stored': dict(match)})
                break
Path(args.output).write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding='utf-8')
print(json.dumps({k:v for k,v in result.items() if k not in ['samples','branches','schema']}, ensure_ascii=False, indent=2))
