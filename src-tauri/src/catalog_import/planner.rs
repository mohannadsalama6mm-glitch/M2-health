use super::*;
use crate::db::{
    catalog_validation::{egp_to_minor, normalize, required},
    connection::AppDb,
};
use rusqlite::Connection;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    time::Instant,
};
pub fn snapshot(c: &Connection) -> Result<String, AppError> {
    let mut bytes = Vec::new();
    for table in [
        "products",
        "product_packages",
        "manufacturers",
        "categories",
        "routes",
        "barcodes",
        "product_price_history",
        "catalog_import_rows",
    ] {
        let mut stmt = c.prepare(&format!("SELECT * FROM {table} ORDER BY 1,2"))?;
        let n = stmt.column_count();
        let mut rows = stmt.query([])?;
        bytes.extend_from_slice(table.as_bytes());
        while let Some(row) = rows.next()? {
            for i in 0..n {
                let s = format!("{:?}", row.get_ref(i)?);
                bytes.extend_from_slice(&(s.len() as u64).to_le_bytes());
                bytes.extend_from_slice(s.as_bytes());
            }
        }
    }
    Ok(source::hash(&bytes))
}
pub fn identity(fields: &[String]) -> String {
    source::hash(
        json(&fields[..6].iter().map(|s| normalize(s)).collect::<Vec<_>>())
            .expect("strings serialize")
            .as_bytes(),
    )
}
pub fn name_key(fields: &[String]) -> String {
    if normalize(&fields[0]).is_empty() {
        format!("ar:{}", normalize(&fields[1]))
    } else {
        format!("en:{}", normalize(&fields[0]))
    }
}
pub fn build(db: &AppDb, root: &Path, reports: &Path) -> Result<Plan, AppError> {
    let start = Instant::now();
    let (profile, fields) = source::read(root)?;
    let c = db.lock()?;
    // One read transaction gives a consistent plan even with another process writing.
    c.execute_batch("BEGIN DEFERRED")?;
    let result = build_on(&c, profile, fields, reports, start);
    c.execute_batch("ROLLBACK")?;
    result
}
fn build_on(
    c: &Connection,
    profile: Profile,
    fields: Vec<Vec<String>>,
    reports: &Path,
    start: Instant,
) -> Result<Plan, AppError> {
    let snapshot = snapshot(c)?;
    let mut known = HashSet::new();
    {
        let mut q = c.prepare(
            "SELECT fingerprint FROM catalog_import_rows WHERE source_id='egyptian-drugs'",
        )?;
        for r in q.query_map([], |r| r.get::<_, String>(0))? {
            known.insert(r?);
        }
    }
    let mut existing = HashSet::new();
    let mut existing_names = HashSet::new();
    {
        let mut q=c.prepare("SELECT coalesce(p.commercial_name_en,''),coalesce(p.commercial_name_ar,''),coalesce(p.scientific_name,''),coalesce(m.name,''),coalesce(c.name,''),coalesce(r.name,'') FROM products p LEFT JOIN manufacturers m ON m.id=p.manufacturer_id LEFT JOIN categories c ON c.id=p.category_id LEFT JOIN routes r ON r.id=p.route_id")?;
        for r in q.query_map([], |r| {
            (0..6)
                .map(|i| r.get::<_, String>(i))
                .collect::<Result<Vec<_>, _>>()
        })? {
            let f = r?;
            existing.insert(identity(&f));
            existing_names.insert(name_key(&f));
        }
    }
    let mut groups: HashMap<String, HashSet<String>> = HashMap::new();
    for f in &fields {
        groups
            .entry(name_key(f))
            .or_default()
            .insert(source::hash(json(f)?.as_bytes()));
    }
    let mut seen = HashSet::new();
    let mut rows = Vec::new();
    let mut counts = BTreeMap::new();
    for (i, f) in fields.into_iter().enumerate() {
        let fingerprint = source::hash(json(&f)?.as_bytes());
        let identity = identity(&f);
        let mut warnings = Vec::new();
        for (index, label) in [
            (2, "missing_scientific_name"),
            (3, "missing_manufacturer"),
            (4, "missing_class"),
        ] {
            if normalize(&f[index]).is_empty() {
                warnings.push(label.to_string());
            }
        }
        if normalize(&f[5]) == "unknown" {
            warnings.push("unknown_route".into());
        }
        if !normalize(&f[2]).is_empty() {
            warnings.push("ingredient_relations_deferred".into());
        }
        let price = egp_to_minor(&f[6]);
        let parts: Vec<_> = f[6].trim().split('.').collect();
        let ambiguous = parts.len() == 2
            && !parts[0].is_empty()
            && parts.iter().all(|s| s.bytes().all(|b| b.is_ascii_digit()))
            && parts[1].len() > 2;
        let disposition = if !seen.insert(fingerprint.clone()) {
            "exact_duplicate_source"
        } else if known.contains(&fingerprint) {
            "already_imported"
        } else if (normalize(&f[0]).is_empty() && normalize(&f[1]).is_empty())
            || f[..6]
                .iter()
                .any(|s| !normalize(s).is_empty() && required(s).is_err())
        {
            "invalid_required_data"
        } else if ambiguous {
            "ambiguous_price"
        } else if price.is_err() {
            "invalid_price"
        } else if existing.contains(&identity) {
            "duplicate_existing_catalog"
        } else if groups.get(&name_key(&f)).is_some_and(|g| g.len() > 1)
            || existing_names.contains(&name_key(&f))
        {
            "possible_duplicate"
        } else {
            "ready"
        };
        *counts.entry(disposition.to_string()).or_default() += 1;
        if !matches!(
            disposition,
            "ready" | "already_imported" | "exact_duplicate_source"
        ) {
            *counts.entry("review_required".into()).or_default() += 1;
        }
        for w in &warnings {
            *counts.entry(w.clone()).or_default() += 1;
        }
        rows.push(Row {
            number: i + 1,
            fields: f,
            fingerprint,
            identity,
            disposition: disposition.into(),
            warnings,
            price_minor: price.ok(),
        });
    }
    let id = uuid::Uuid::new_v4().to_string();
    let mut report = Report {
        id: id.clone(),
        mode: "dry_run".into(),
        status: "validated".into(),
        profile: Some(profile),
        counts,
        rows,
        report_path: reports
            .join(format!("{id}-dry-run.json"))
            .to_string_lossy()
            .into_owned(),
        duration_ms: start.elapsed().as_millis() as u64,
        ..Default::default()
    };
    for (index, table) in [(3, "manufacturers"), (4, "categories"), (5, "routes")] {
        let mut q = c.prepare(&format!("SELECT normalized_name FROM {table}"))?;
        let known = q
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<Result<HashSet<_>, _>>()?;
        let values = report
            .rows
            .iter()
            .filter(|r| r.disposition == "ready")
            .map(|r| normalize(&r.fields[index]))
            .filter(|s| !s.is_empty())
            .collect::<HashSet<_>>();
        report
            .lookup_created
            .insert(table.into(), values.difference(&known).count());
        report
            .lookup_reused
            .insert(table.into(), values.intersection(&known).count());
    }
    save(&report)?;
    Ok(Plan { report, snapshot })
}
