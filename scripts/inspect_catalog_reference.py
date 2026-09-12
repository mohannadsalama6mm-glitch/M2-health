"""Read-only reference profiler. Never connects to the application database."""
import argparse
import collections
import csv
import decimal
import hashlib
import json
import re
from pathlib import Path

FIELDS = ["commercial_name_en", "commercial_name_ar", "scientific_name", "manufacturer", "drug_class", "route", "price_egp"]
PROJECT_ROOT = Path(__file__).resolve().parents[1]
SOURCE_CONFIG = PROJECT_ROOT / "config" / "catalog-source.json"

def canonical_source():
    config = json.loads(SOURCE_CONFIG.read_text(encoding="utf-8"))
    path = (PROJECT_ROOT / config["path"]).resolve()
    if not path.is_relative_to(PROJECT_ROOT) or not path.is_file():
        raise ValueError("Canonical Catalog source must be an existing file inside the project.")
    return path

def normalize(value):
    return re.sub(r"\s+", " ", value.strip()).translate(str.maketrans("ABCDEFGHIJKLMNOPQRSTUVWXYZ", "abcdefghijklmnopqrstuvwxyz"))

def inspect(path):
    with path.open(encoding="utf-8-sig", newline="") as stream:
        reader = csv.DictReader(stream)
        if reader.fieldnames != FIELDS:
            raise ValueError(f"Unexpected CSV fields: {reader.fieldnames}")
        rows, lines = [], []
        for row in reader:
            rows.append(row)
            lines.append(reader.line_num)
    missing = {f: sum(not r[f].strip() for r in rows) for f in FIELDS}
    duplicates = len(rows) - len({tuple(r[f] for f in FIELDS) for r in rows})
    names = collections.Counter(normalize(r["commercial_name_en"]) for r in rows)
    prices = []
    invalid_prices = []
    for line, row in zip(lines, rows):
        try:
            amount = decimal.Decimal(row["price_egp"])
            if not amount.is_finite() or amount < 0 or amount * 100 != (amount * 100).to_integral_value():
                raise ValueError()
            prices.append(amount)
        except (decimal.InvalidOperation, ValueError):
            invalid_prices.append(line)
    variants = {}
    for field in ["manufacturer", "drug_class", "route"]:
        groups = collections.defaultdict(set)
        for row in rows:
            if row[field].strip():
                groups[normalize(row[field])].add(row[field])
        variants[field] = [sorted(v) for v in groups.values() if len(v) > 1][:10]
    samples = [dict(sourceEndLine=lines[0], **rows[0])]
    for field in ["scientific_name", "manufacturer", "drug_class"]:
        for line, row in zip(lines, rows):
            if not row[field].strip():
                samples.append(dict(sourceEndLine=line, **row))
                break
    for predicate in [lambda r: r["route"] == "UNKNOWN", lambda r: decimal.Decimal(r["price_egp"]) % 1 != 0]:
        for line, row in zip(lines, rows):
            if predicate(row):
                samples.append(dict(sourceEndLine=line, **row))
                break
    resolved = path.resolve()
    source_path = resolved.relative_to(PROJECT_ROOT).as_posix() if resolved.is_relative_to(PROJECT_ROOT) else str(resolved)
    return dict(sourceFile=path.name, sourcePath=source_path, format="csv", sha256=hashlib.sha256(path.read_bytes()).hexdigest(), columns=FIELDS,
                rowCount=len(rows), missing=missing, maxLengths={f:max(len(r[f]) for r in rows) for f in FIELDS},
                exactDuplicateRows=duplicates, normalizedEnglishDuplicateGroups=sum(n>1 for n in names.values()),
                routes=dict(collections.Counter(r["route"] for r in rows)), lookupVariants=variants,
                scientificCombinations=sum("+" in r["scientific_name"] for r in rows),
                priceMin=str(min(prices)), priceMax=str(max(prices)), invalidPriceLines=invalid_prices,
                invalidPriceExamples=[dict(sourceEndLine=line, **row) for line,row in zip(lines, rows) if line in invalid_prices],
                representativeRows=samples)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("csv_path", type=Path, nargs="?", help="Defaults to config/catalog-source.json")
    parser.add_argument("--output", type=Path, help="Optional JSON report; never a DB import")
    args = parser.parse_args()
    source = args.csv_path or canonical_source()
    if args.output and args.output.resolve() == source.resolve():
        parser.error("The read-only source cannot be used as the report output.")
    report = json.dumps(inspect(source), ensure_ascii=False, indent=2)
    if args.output:
        args.output.write_text(report + "\n", encoding="utf-8")
    else:
        print(report)
