"""Reproducible, read-only canonical v2 comparison. Never opens SQLite writable.

Writes review artifacts only. Baseline is create-once; --verify checks preservation.
Matching changes only ASCII case / Unicode whitespace, never medicine tokens.
"""
import argparse
import collections
import csv
import hashlib
import io
import json
import os
from pathlib import Path
import re
import sqlite3

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'docs/catalog-v2'
HEADERS = ['commercial_name_en', 'commercial_name_ar', 'scientific_name',
           'manufacturer', 'drug_class', 'route', 'price_egp']
V1 = '35867044249d327b81c8c87988d6328c802da39359dc768eb4e7f1e9528dee0c'
V2 = '5f31d97c3491dd75bae780bba8ce52bb8e33bf81a7f0f085884159f08d38de41'


def encoded(value):
    return json.dumps(value, ensure_ascii=False, separators=(',', ':')).encode('utf8')


def digest(value):
    return hashlib.sha256(encoded(value)).hexdigest()


def normalize(value):
    return ' '.join(value.split()).translate(str.maketrans('ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz'))


def price(value):
    value = value.strip()
    if not re.fullmatch(r'[0-9]+(?:\.[0-9]{1,2})?', value):
        return None
    whole, _, fraction = value.partition('.')
    result = int(whole) * 100 + int(fraction.ljust(2, '0') or '0')
    return result if result <= 9007199254740991 else None


def records(c, sql):
    cur = c.execute(sql)
    return [dict(zip([d[0] for d in cur.description], row)) for row in cur]


def snapshot(c, source_hash):
    tables = [r[0] for r in c.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")]
    data = {}
    for table in tables:
        quoted = '"' + table.replace('"', '""') + '"'
        rows = c.execute('SELECT * FROM ' + quoted).fetchall()
        # Sort serialized complete rows, including composite-key provenance tables.
        data[table] = {'count': len(rows), 'sha256': digest(sorted(encoded(list(r)).decode('utf8') for r in rows))}
    return {
        'tables': data,
        'schemaHash': digest(c.execute('SELECT type,name,tbl_name,sql FROM sqlite_master ORDER BY type,name').fetchall()),
        'versions': c.execute('SELECT * FROM _schema_version ORDER BY version').fetchall(),
        'branches': records(c, 'SELECT * FROM branches ORDER BY id'),
        'stock': records(c, 'SELECT branch_id,product_package_id,sum(quantity_delta) quantity FROM inventory_movements GROUP BY branch_id,product_package_id ORDER BY 1,2'),
        'integrity': c.execute('PRAGMA integrity_check').fetchall(),
        'foreignKeyErrors': c.execute('PRAGMA foreign_key_check').fetchall(),
        'sourceHash': source_hash,
    }


def write(name, value):
    (OUT / name).write_text(json.dumps(value, ensure_ascii=False, indent=2), encoding='utf8')


def compare(rows, products, historical, malformed=()):
    by_en, by_ar = collections.defaultdict(list), collections.defaultdict(list)
    for p in products:
        for index, mapping in [(0, by_en), (1, by_ar)]:
            if normalize(p['fields'][index]):
                mapping[normalize(p['fields'][index])].append(p)
    names = collections.Counter(normalize(r[0]) or normalize(r[1]) for r in rows if len(r) == 7)
    exact = collections.Counter(tuple(r) for r in rows)
    seen = set()
    plan = []
    for number, raw in enumerate(rows, 1):
        fingerprint = digest(raw)
        row = [v.strip() for v in raw]
        item = {'sourceVersion': 'v2', 'sourceHash': V2, 'sourceRow': number,
                'sourceRowFingerprint': fingerprint, 'logicalFingerprint': digest(row),
                'rawFields': raw, 'validColumnCount': len(row) == 7,
                'values': dict(zip(HEADERS, row)) if len(row) == 7 else {},
                'unparsedFields': row if len(row) != 7 else None, 'classification': 'REVIEW_REQUIRED',
                'matchBasis': None, 'candidateProductIds': [], 'changes': {},
                'historicalChanges': {}, 'warnings': [], 'proposedAction': 'HOLD',
                'applied': False}
        plan.append(item)
        if len(row) != 7:
            item['warnings'].append('malformed_row')
            continue
        key = normalize(row[0]) or normalize(row[1])
        pence = price(row[6])
        item['priceMinor'] = pence
        if number in malformed:
            item['warnings'].append('malformed_csv_quoting')
        for index in [2, 3, 4]:
            if not row[index]:
                item['warnings'].append('missing_' + HEADERS[index])
        if not key:
            item['warnings'].append('missing_commercial_name')
        if pence is None:
            item['warnings'].append('invalid_or_overprecision_price')
        if any('\ufffd' in v or '\0' in v for v in row):
            item['warnings'].append('text_integrity_review')
        if any(v.startswith('"') and v.endswith('"') for v in row):
            item['warnings'].append('padded_quote_interpretation_requires_review')
        candidates = by_en.get(normalize(row[0]), []) if row[0] else by_ar.get(normalize(row[1]), [])
        item['matchBasis'] = 'normalized_english_name' if row[0] else 'normalized_arabic_name'
        renamed = False
        if not candidates and row[1]:
            candidates = by_ar.get(normalize(row[1]), [])
            if candidates:
                renamed = True
                item['matchBasis'] = 'arabic_name_only_requires_identity_review'
        item['candidateProductIds'] = [p['id'] for p in candidates]
        if len(candidates) == 1:
            p = candidates[0]
            item['productId'] = p['id']
            item['packageId'] = p['package_id']
            item['importedFromV1'] = p['source_row'] is not None
            for index, field in enumerate(HEADERS[:6]):
                if row[index] != p['fields'][index].strip():
                    item['changes'][field] = {'current': p['fields'][index], 'v2': row[index]}
            if pence is not None and pence != p['price_minor']:
                item['changes']['price_egp'] = {'currentMinor': p['price_minor'], 'v2Minor': pence}
            old = historical.get(p['source_row'])
            if old:
                item['historicalSourceRow'] = old['number']
                item['historicalFingerprint'] = old['fingerprint']
                for index, field in enumerate(HEADERS):
                    different = price(old['fields'][index]) != pence if index == 6 else old['fields'][index].strip() != row[index]
                    if different:
                        item['historicalChanges'][field] = {'v1': old['fields'][index], 'v2': row[index]}
                local_edits = [HEADERS[i] for i in range(6) if old['fields'][i].strip() != p['fields'][i].strip()]
                if price(old['fields'][6]) != p['price_minor']:
                    local_edits.append('price_egp')
                item['catalogDisplayDifferencesFromV1'] = local_edits
                substantive_local = [HEADERS[i] for i in range(6) if normalize(old['fields'][i]) != normalize(p['fields'][i])]
                if price(old['fields'][6]) != p['price_minor']:
                    substantive_local.append('price_egp')
                item['localChangesSinceV1'] = substantive_local
                if substantive_local:
                    item['warnings'].append('local_catalog_changes_preserve')
            else:
                item['warnings'].append('non_v1_catalog_record_preserve')
        # Catalog lookups are shared by normalized key; their first imported display
        # spelling is not evidence of a local edit or a new medicine identity.
        display_only = [f for f, delta in item['changes'].items()
                        if f in HEADERS[3:6] and normalize(delta['current']) == normalize(delta['v2'])]
        item['lookupDisplayOnlyDifferences'] = display_only
        substantive = {f: d for f, d in item['changes'].items() if f not in display_only}
        item['substantiveChanges'] = substantive
        if tuple(raw) in seen:
            item['warnings'].append('repeated_exact_source_row')
        seen.add(tuple(raw))
        if names[key] > 1 or len(candidates) > 1 or exact[tuple(raw)] > 1:
            item['classification'] = 'POSSIBLE_DUPLICATE'
            item['warnings'].append('ambiguous_source_or_catalog_name')
        elif renamed or not key or pence is None or number in malformed or any(w in item['warnings'] for w in ['text_integrity_review', 'padded_quote_interpretation_requires_review']):
            item['classification'] = 'REVIEW_REQUIRED'
        elif len(candidates) == 1 and not substantive:
            item['classification'] = 'UNCHANGED'
            item['proposedAction'] = 'SKIP_UNCHANGED'
        elif item['warnings']:
            item['classification'] = 'REVIEW_REQUIRED'
        elif candidates:
            item['classification'] = 'CHANGED'
            item['proposedAction'] = 'REVIEW_METADATA_AND_APPEND_PRICE' if 'price_egp' in item['changes'] else 'REVIEW_METADATA'
        else:
            item['classification'] = 'NEW'
            item['proposedAction'] = 'REVIEW_INSERT'
    return plan


def main():
    args = argparse.ArgumentParser()
    args.add_argument('--verify', action='store_true')
    options = args.parse_args()
    OUT.mkdir(exist_ok=True)
    source = ROOT / 'src/data/karem/egyptian-drugs.csv'
    data = source.read_bytes()
    sha = hashlib.sha256(data).hexdigest()
    assert sha == V2 and len(data) == 26437838, 'Approved v2 bytes changed'
    home = Path(os.environ['LOCALAPPDATA']) / 'com.m2health.pharmacy'
    c = sqlite3.connect((home / 'm2-health.db').as_uri() + '?mode=ro', uri=True)
    c.execute('PRAGMA query_only=ON')
    c.execute('BEGIN')
    current = snapshot(c, sha)
    baseline_path = OUT / 'baseline.json'
    if baseline_path.exists():
        assert json.loads(baseline_path.read_text(encoding='utf8')) == json.loads(json.dumps(current)), 'Development DB or CSV differs from adoption baseline'
    else:
        assert not options.verify, 'Missing baseline'
        write('baseline.json', current)
    if options.verify:
        summary = json.loads((OUT / 'summary.json').read_text(encoding='utf8'))
        report_hash = hashlib.sha256(Path(summary['historicalRun']['report_path']).read_bytes()).hexdigest()
        assert report_hash == summary['historicalReportHash'], 'Historical import report changed'
        write('preservation.json', {'status': 'UNCHANGED', 'allTablesAndSchemaIdentical': True,
                                   'historicalImportReportUnchanged': True, 'snapshot': current})
        print('All development DB tables, schema and CSV unchanged.')
        return
    runs = records(c, 'SELECT * FROM catalog_import_runs ORDER BY started_at')
    run = next(r for r in runs if r['source_hash'] == V1)
    report_bytes = Path(run['report_path']).read_bytes()
    report = json.loads(report_bytes)
    assert report['profile']['hash'] == V1 and report['id'] == run['id']
    historical = {r['number']: r for r in report['rows']}
    provenance = records(c, 'SELECT * FROM catalog_import_rows')
    assert all(r['fingerprint'] == historical[r['source_row']]['fingerprint'] == digest(historical[r['source_row']]['fields']) for r in provenance if r['run_id'] == run['id'])
    text = data.decode('utf-8-sig', errors='strict')
    # Retain the complete physical span for a strict syntax check per logical row.
    # The supplied padded CSV requires tolerant decoding; syntax issues are held.
    reader = csv.reader(io.StringIO(text, newline=''), strict=False)
    lines = text.splitlines(keepends=True)
    parsed, malformed, spans, previous = [], [], [], 0
    for record in reader:
        span = ''.join(lines[previous:reader.line_num])
        spans.append([previous + 1, reader.line_num])
        previous = reader.line_num
        try:
            list(csv.reader(io.StringIO(span, newline=''), strict=True))
        except csv.Error:
            malformed.append(len(parsed))
        parsed.append(record)
    header, rows = parsed[0], parsed[1:]
    assert [s.strip() for s in header] == HEADERS and len(rows) == 25088
    products = records(c, '''SELECT p.id,p.commercial_name_en,p.commercial_name_ar,p.scientific_name,
        m.name manufacturer,g.name drug_class,r.name route,k.id package_id,i.source_row,
        (SELECT h.selling_price_minor FROM product_price_history h WHERE h.product_package_id=k.id
         AND h.effective_from<=strftime('%Y-%m-%dT%H:%M:%fZ','now')
         AND (h.effective_to IS NULL OR h.effective_to>strftime('%Y-%m-%dT%H:%M:%fZ','now'))
         ORDER BY h.effective_from DESC,h.created_at DESC,h.id DESC LIMIT 1) price_minor
        FROM products p LEFT JOIN manufacturers m ON m.id=p.manufacturer_id
        LEFT JOIN categories g ON g.id=p.category_id LEFT JOIN routes r ON r.id=p.route_id
        LEFT JOIN product_packages k ON k.product_id=p.id AND k.is_default=1
        LEFT JOIN catalog_import_rows i ON i.product_id=p.id AND i.package_id=k.id''')
    for p in products:
        p['fields'] = [p[field] or '' for field in HEADERS[:6]]
    plan = compare(rows, products, historical, malformed)
    historical_names = collections.defaultdict(list)
    for original in historical.values():
        historical_names[normalize(original['fields'][0]) or normalize(original['fields'][1])].append(original)
    for item, span in zip(plan, spans[1:]):
        item['physicalLineSpan'] = span
        if item['validColumnCount'] and not item['candidateProductIds']:
            key = normalize(item['values']['commercial_name_en']) or normalize(item['values']['commercial_name_ar'])
            item['historicalUnimportedCandidates'] = [
                {'sourceRow': r['number'], 'disposition': r['disposition'], 'fingerprint': r['fingerprint']}
                for r in historical_names.get(key, []) if r['disposition'] != 'imported']
    valid = [r for r in rows if len(r) == 7]
    trimmed = [[v.strip() for v in r] for r in valid]
    duplicates = collections.Counter(tuple(r) for r in valid)
    logical_duplicates = collections.Counter(tuple(r) for r in trimmed)
    names = collections.Counter(normalize(r[0]) or normalize(r[1]) for r in valid)
    overprecision = [i for i, r in enumerate(rows, 1) if len(r) == 7 and re.fullmatch(r'[0-9]+\.[0-9]{3,}', r[6].strip())]
    invalid = [i for i, r in enumerate(rows, 1) if len(r) == 7 and price(r[6]) is None and i not in overprecision]
    profile = {
        'sourceVersion': 'v2', 'sourceHash': sha, 'bytes': len(data), 'dataRows': len(rows), 'columns': HEADERS,
        'sevenColumnRecords': len(valid),
        'fieldStatisticsScope': 'Seven-column records only; 26 malformed records remain explicitly unassigned, not guessed or merged.',
        'paddedHeaders': sum(a != a.strip() for a in header),
        'rowsWithFieldPadding': sum(any(v != v.strip() for v in r) for r in valid),
        'missing': {HEADERS[i]: sum(not r[i].strip() for r in valid) for i in [2, 3, 4]},
        'routes': dict(sorted(collections.Counter(r[5].strip() for r in valid).items())),
        'exactDuplicateGroups': sum(n > 1 for n in duplicates.values()),
        'exactDuplicateExtraRows': sum(n - 1 for n in duplicates.values()),
        'trimmedDuplicateExtraRows': sum(n - 1 for n in logical_duplicates.values()),
        'normalizedNameDuplicateGroups': sum(n > 1 for n in names.values()),
        'normalizedNameDuplicateRows': sum(n for n in names.values() if n > 1),
        'invalidPriceRows': invalid, 'overprecisionPriceRows': overprecision,
        'malformedColumnRows': [i for i, r in enumerate(rows, 1) if len(r) != 7],
        'strictCsvSyntaxIssueRows': malformed,
        'malformedRows': sorted(set(malformed + [i for i, r in enumerate(rows, 1) if len(r) != 7])),
        'utf8Valid': True, 'replacementCharacters': text.count('\ufffd'), 'nulCharacters': text.count('\0'),
        'rowsWithArabicScript': sum(any('\u0600' <= ch <= '\u06ff' for ch in r[1]) for r in valid),
        'missingArabicNames': sum(not r[1].strip() for r in valid),
        'textIntegrityScope': 'Strict UTF-8, U+FFFD/NUL and Arabic-script presence; no linguistic or medical correction.',
    }
    counts = collections.Counter({key: 0 for key in ['UNCHANGED', 'NEW', 'CHANGED', 'POSSIBLE_DUPLICATE', 'REVIEW_REQUIRED']})
    counts.update(p['classification'] for p in plan)
    changes = collections.Counter(f for p in plan for f in p['changes'])
    summary = {'applyStatus': 'NOT APPLIED', 'profile': profile, 'parsedRowsFingerprint': digest(rows),
               'historicalRun': run, 'historicalReportHash': hashlib.sha256(report_bytes).hexdigest(),
               'historicalReportProfile': report['profile'], 'verifiedProvenanceRows': len(provenance),
               'baselineCounts': {k: v['count'] for k, v in current['tables'].items()},
               'classifications': dict(counts), 'fieldChangeCandidates': dict(changes),
               'rowsWithChanges': sum(bool(p['changes']) for p in plan),
               'priceChangeCandidates': sum('price_egp' in p['changes'] for p in plan),
               'metadataChangeCandidates': sum(any(f != 'price_egp' for f in p['changes']) for p in plan),
               'substantiveMetadataChangeCandidates': sum(bool(p.get('substantiveChanges')) for p in plan),
               'lookupDisplayOnlyRows': sum(bool(p.get('lookupDisplayOnlyDifferences')) for p in plan),
               'noCatalogMatchRows': sum(not p['candidateProductIds'] for p in plan),
               'validNoCatalogMatchRows': sum(p['validColumnCount'] and not p['candidateProductIds'] for p in plan),
               'skippedOrHeldRows': sum(p['proposedAction'] in ['HOLD', 'SKIP_UNCHANGED'] for p in plan),
               'localChangeCandidates': sum(bool(p.get('localChangesSinceV1')) for p in plan),
               'unmatchedImportedProducts': sorted({r['product_id'] for r in provenance} - {pid for p in plan for pid in p['candidateProductIds']}),
               'matchingPolicy': 'Unique ASCII-case/whitespace-normalized English name; Arabic-only candidates require review. Duplicate source names or multiple Catalog candidates are never merged. Strength/pack/punctuation are never stripped. Values are edge-trimmed only for comparison; normalized shared-lookup display variants are recorded separately as unchanged. Missing metadata on new/changed rows requires review. Counts of field impacts overlap primary classifications. All rows remain unapplied.',
               'fingerprintPolicy': 'SHA256 of compact UTF-8 JSON array of the seven original, untrimmed decoded fields, same serialization as historical importer; sourceHash anchors complete file bytes. logicalFingerprint uses edge-trimmed fields.'}
    assert sum(counts.values()) == 25088
    summary['unmatchedImportedDetails'] = [
        {'productId': p['id'], 'name': p['fields'][0], 'historicalSourceRow': p['source_row']}
        for p in products if p['id'] in summary['unmatchedImportedProducts']]
    write('summary.json', summary)
    write('update-plan.json', {'applyStatus': 'NOT APPLIED', 'sourceVersion': 'v2', 'sourceHash': sha, 'baselineSchemaHash': current['schemaHash'], 'rows': plan})
    write('profile.json', profile)
    c.rollback()
    c.close()
    print(json.dumps(summary, ensure_ascii=True, indent=2))


if __name__ == '__main__':
    main()
