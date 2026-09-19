//! Active-source reconciliation. A single transaction commits Catalog changes and
//! immutable row actions together; no Inventory repository or table is written.
use super::*;
use crate::db::{
    catalog_validation::{egp_to_minor, normalize, required},
    connection::AppDb,
    models::catalog::SetPackagePrice,
    repositories::price_history,
};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior, MAIN_DB};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    time::Instant,
};
pub const POLICY: &str = "conservative_catalog_v2";

struct Existing {
    product: String,
    package: Option<String>,
    active: bool,
    price: Option<i64>,
}
fn snapshot(c: &Connection) -> Result<String, AppError> {
    let catalog = planner::snapshot(c)?;
    let mut q = c.prepare("SELECT run_id,source_row,fingerprint,action FROM catalog_source_rows ORDER BY run_id,source_row")?;
    let rows = q
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(source::hash(json(&(catalog, rows))?.as_bytes()))
}
pub fn build(
    db: &AppDb,
    profile: Profile,
    fields: Vec<Vec<String>>,
    output: &Path,
) -> Result<Plan, AppError> {
    let c = db.lock()?;
    c.execute_batch("BEGIN DEFERRED")?;
    let result = build_on(&c, profile, fields, output);
    c.execute_batch("ROLLBACK")?;
    result
}
fn build_on(
    c: &Connection,
    profile: Profile,
    fields: Vec<Vec<String>>,
    output: &Path,
) -> Result<Plan, AppError> {
    let snap = snapshot(c)?;
    let mut existing: HashMap<String, Vec<Existing>> = HashMap::new();
    let mut existing_names = HashSet::new();
    let mut existing_arabic = HashSet::new();
    let mut q = c.prepare("SELECT p.id,k.id,p.is_active,coalesce(k.is_active,0),coalesce(p.commercial_name_en,''),coalesce(p.commercial_name_ar,''),coalesce(p.scientific_name,''),coalesce(m.name,''),coalesce(g.name,''),coalesce(r.name,''),h.selling_price_minor FROM products p LEFT JOIN product_packages k ON k.product_id=p.id AND k.is_default=1 LEFT JOIN manufacturers m ON m.id=p.manufacturer_id LEFT JOIN categories g ON g.id=p.category_id LEFT JOIN routes r ON r.id=p.route_id LEFT JOIN product_price_history h ON h.product_package_id=k.id AND h.effective_to IS NULL")?;
    for row in q.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, bool>(2)? && r.get::<_, bool>(3)?,
            (4..10)
                .map(|i| r.get::<_, String>(i))
                .collect::<Result<Vec<_>, _>>()?,
            r.get::<_, Option<i64>>(10)?,
        ))
    })? {
        let (product, package, active, f, price) = row?;
        if !normalize(&f[0]).is_empty() {
            existing_names.insert(normalize(&f[0]));
        }
        if !normalize(&f[1]).is_empty() {
            existing_arabic.insert(normalize(&f[1]));
        }
        existing
            .entry(planner::identity(&f))
            .or_default()
            .push(Existing {
                product,
                package,
                active,
                price,
            });
    }
    let mut known = HashSet::new();
    let mut q=c.prepare("SELECT a.fingerprint,a.product_id,a.package_id FROM catalog_source_rows a JOIN catalog_source_runs s ON s.run_id=a.run_id JOIN catalog_import_runs r ON r.id=s.run_id WHERE r.source_hash=?1 AND s.source_version=?2 AND a.action IN ('inserted','reused','updated')")?;
    for r in q.query_map(params![profile.hash, profile.source_version], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })? {
        known.insert(r?);
    }
    let malformed: HashSet<_> = profile.malformed_rows.iter().copied().collect();
    let quoting: HashSet<_> = profile.quoting_issue_rows.iter().copied().collect();
    let mut groups: HashMap<String, HashSet<String>> = HashMap::new();
    for (i, f) in fields.iter().enumerate() {
        if f.len() == 7 && !malformed.contains(&(i + 1)) && !quoting.contains(&(i + 1)) {
            if let Ok(price) = egp_to_minor(&f[6]) {
                groups
                    .entry(planner::name_key(f))
                    .or_default()
                    .insert(format!("{}:{price}", planner::identity(f)));
            }
        }
    }
    let mut seen = HashSet::new();
    let mut rows = Vec::new();
    let mut counts = BTreeMap::new();
    for (i, f) in fields.into_iter().enumerate() {
        let mut row = Row {
            number: i + 1,
            fingerprint: source::hash(json(&f)?.as_bytes()),
            identity: String::new(),
            disposition: String::new(),
            warnings: Vec::new(),
            price_minor: None,
            product_id: None,
            package_id: None,
            fields: f,
        };
        if malformed.contains(&row.number) || row.fields.len() != 7 {
            row.disposition = "skipped_malformed".into();
        } else if quoting.contains(&row.number) {
            row.disposition = "skipped_quoting".into();
        } else {
            let f = &row.fields;
            row.identity = planner::identity(f);
            for (index, label) in [
                (2, "missing_scientific_name"),
                (3, "missing_manufacturer"),
                (4, "missing_class"),
            ] {
                if normalize(&f[index]).is_empty() {
                    row.warnings.push(label.into());
                }
            }
            if normalize(&f[5]) == "unknown" {
                row.warnings.push("unknown_route".into());
            }
            if !normalize(&f[2]).is_empty() {
                row.warnings.push("ingredient_linking_deferred".into());
            }
            let price = egp_to_minor(&f[6]);
            if (normalize(&f[0]).is_empty() && normalize(&f[1]).is_empty())
                || f[..6]
                    .iter()
                    .any(|s| !normalize(s).is_empty() && required(s).is_err())
            {
                row.disposition = "skipped_invalid".into();
            } else if price.is_err() {
                row.disposition = "skipped_price".into();
            } else {
                row.price_minor = price.ok();
                let logical = format!("{}:{}", row.identity, row.price_minor.unwrap());
                if groups
                    .get(&planner::name_key(f))
                    .is_some_and(|g| g.len() > 1)
                {
                    row.disposition = "skipped_ambiguous".into();
                } else if !seen.insert(logical) {
                    row.disposition = "skipped_duplicate".into();
                } else if let Some(candidates) = existing.get(&row.identity) {
                    if candidates.len() != 1 || candidates[0].package.is_none() {
                        row.disposition = "skipped_ambiguous".into();
                    } else {
                        let target = &candidates[0];
                        row.product_id = Some(target.product.clone());
                        row.package_id = target.package.clone();
                        if target.price != row.price_minor && !target.active {
                            row.disposition = "skipped_ambiguous".into();
                            row.warnings.push("inactive_price_target".into());
                        } else if target.price != row.price_minor {
                            row.disposition = "ready_update".into();
                        } else if known.contains(&(
                            row.fingerprint.clone(),
                            target.product.clone(),
                            target.package.clone().unwrap(),
                        )) {
                            row.disposition = "already_imported".into();
                        } else {
                            row.disposition = "ready_reuse".into();
                        }
                    }
                } else if existing_names.contains(&normalize(&f[0]))
                    || existing_arabic.contains(&normalize(&f[1]))
                {
                    row.disposition = "skipped_ambiguous".into();
                } else {
                    row.disposition = "ready_insert".into();
                }
            }
        }
        *counts.entry(row.disposition.clone()).or_default() += 1;
        if row.disposition.starts_with("ready_") {
            *counts.entry("ready".into()).or_default() += 1;
        }
        if row.disposition.starts_with("skipped_") && row.disposition != "skipped_duplicate" {
            *counts.entry("review_required".into()).or_default() += 1;
        }
        for w in &row.warnings {
            *counts.entry(w.clone()).or_default() += 1;
        }
        rows.push(row);
    }
    let id = uuid::Uuid::new_v4().to_string();
    let report = Report {
        id: id.clone(),
        mode: "dry_run".into(),
        status: "validated".into(),
        profile: Some(profile),
        rows,
        counts,
        report_path: output
            .join(format!("{id}-dry-run.json"))
            .to_string_lossy()
            .into_owned(),
        ..Default::default()
    };
    save(&report)?;
    Ok(Plan {
        report,
        snapshot: snap,
    })
}
fn integrity(c: &Connection) -> Result<(), AppError> {
    let value: String = c.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if value != "ok" {
        return Err(invalid(
            "Database integrity check failed; no Catalog writes attempted.",
        ));
    }
    if c.prepare("PRAGMA foreign_key_check")?
        .query([])?
        .next()?
        .is_some()
    {
        return Err(invalid(
            "Foreign-key violations found; no Catalog writes attempted.",
        ));
    }
    Ok(())
}
fn lookup(
    c: &Connection,
    table: &str,
    value: &str,
    report: &mut Report,
) -> Result<Option<String>, AppError> {
    let key = normalize(value);
    if key.is_empty() {
        return Ok(None);
    }
    let found = c
        .query_row(
            &format!("SELECT id FROM {table} WHERE normalized_name=?1"),
            [&key],
            |r| r.get::<_, String>(0),
        )
        .optional()?;
    if let Some(id) = found {
        *report.lookup_reused.entry(table.into()).or_default() += 1;
        return Ok(Some(id));
    }
    let id = uuid::Uuid::new_v4().to_string();
    c.execute(
        &format!("INSERT INTO {table}(id,name,normalized_name) VALUES (?1,?2,?3)"),
        params![id, value, key],
    )?;
    *report.lookup_created.entry(table.into()).or_default() += 1;
    Ok(Some(id))
}
fn optional(value: &str) -> Option<&str> {
    if normalize(value).is_empty() {
        None
    } else {
        Some(value)
    }
}
fn insert(
    c: &Connection,
    row: &mut Row,
    report: &mut Report,
    time: &str,
) -> Result<String, AppError> {
    let f = &row.fields;
    let m = lookup(c, "manufacturers", &f[3], report)?;
    let g = lookup(c, "categories", &f[4], report)?;
    let r = lookup(c, "routes", &f[5], report)?;
    let product = uuid::Uuid::new_v4().to_string();
    let package = uuid::Uuid::new_v4().to_string();
    let price = uuid::Uuid::new_v4().to_string();
    c.prepare_cached("INSERT INTO products(id,commercial_name_en,commercial_name_ar,scientific_name,normalized_name_en,normalized_name_ar,normalized_scientific_name,manufacturer_id,category_id,route_id,notes) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)")?.execute(params![product,optional(&f[0]),optional(&f[1]),optional(&f[2]),optional(&f[0]).map(normalize),optional(&f[1]).map(normalize),optional(&f[2]).map(normalize),m,g,r,"Active canonical source import. Default Package is a placeholder; package size and ingredient relations remain unverified."])?;
    c.prepare_cached("INSERT INTO product_packages(id,product_id,package_label,is_default) VALUES (?1,?2,'Default Package',1)")?.execute(params![package,product])?;
    c.prepare_cached("INSERT INTO product_price_history(id,product_package_id,selling_price_minor,effective_from,reason) VALUES (?1,?2,?3,?4,'Initial active canonical source import')")?.execute(params![price,package,row.price_minor,time])?;
    row.product_id = Some(product);
    row.package_id = Some(package);
    Ok(price)
}
fn persist_run(c: &Connection, report: &Report) -> Result<(), AppError> {
    let p = report
        .profile
        .as_ref()
        .ok_or_else(|| invalid("Source profile missing."))?;
    c.execute("INSERT INTO catalog_import_runs(id,source_hash,source_path,started_at,finished_at,total_rows,imported_rows,status,report_path,backup_path) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",params![report.id,p.hash,p.path,report.started_at,report.finished_at,p.total_rows as i64,report.imported as i64,report.status,report.report_path,report.backup_path])?;
    let n = |key: &str| report.counts.get(key).copied().unwrap_or(0) as i64;
    let backup_size = i64::try_from(report.backup_size.unwrap_or(0))
        .map_err(|_| invalid("Backup file size is unsupported."))?;
    c.execute("INSERT INTO catalog_source_runs(run_id,source_version,reused_rows,updated_rows,malformed_rows,quoting_rows,ambiguous_rows,duplicate_rows,invalid_price_rows,failed_rows,backup_at,backup_size) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",params![report.id,p.source_version,report.reused as i64,report.updated as i64,n("skipped_malformed"),n("skipped_quoting"),n("skipped_ambiguous"),n("skipped_duplicate"),n("skipped_price"),report.failed as i64,report.started_at,backup_size])?;
    Ok(())
}
pub fn apply(
    db: &AppDb,
    root: &Path,
    output: &Path,
    plan: Plan,
    progress: impl Fn(&Report),
) -> Result<Report, AppError> {
    let start = Instant::now();
    let (fresh, _) = source::read(root)?;
    if !fresh.apply_allowed
        || fresh.import_policy.as_deref() != Some(POLICY)
        || fresh.source_version.as_deref() != Some("v2")
        || plan
            .report
            .profile
            .as_ref()
            .is_none_or(|p| p.hash != fresh.hash || p.import_policy != fresh.import_policy)
    {
        return Err(invalid("Source policy changed. Prepare a fresh import."));
    }
    let mut c = db.lock()?;
    if snapshot(&c)? != plan.snapshot {
        return Err(invalid("Catalog changed. Prepare a fresh import."));
    }
    integrity(&c)?;
    let mut report = plan.report;
    report.mode = "apply".into();
    report.status = "applying".into();
    report.report_path = output
        .join(format!("{}-apply.json", report.id))
        .to_string_lossy()
        .into_owned();
    let folder = output.join("backups");
    std::fs::create_dir_all(&folder)?;
    let path = folder.join(format!("catalog-before-{}.sqlite3", report.id));
    if path.exists() {
        return Err(invalid("This plan was already attempted."));
    }
    c.backup(MAIN_DB, &path, None)?;
    let backup = Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    integrity(&backup)?;
    drop(backup);
    report.backup_path = Some(path.to_string_lossy().into_owned());
    report.backup_size = Some(std::fs::metadata(path)?.len());
    let time: String = c.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now')", [], |r| {
        r.get(0)
    })?;
    report.started_at = Some(time.clone());
    save(&report)?;
    let original_rows = report.rows.clone();
    let result = (|| -> Result<(), AppError> {
        let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
        if snapshot(&tx)? != plan.snapshot {
            return Err(invalid(
                "Catalog changed during backup; import rolled back.",
            ));
        }
        // The parent run exists within this same transaction for every row action.
        persist_run(&tx, &report)?;
        let mut rows = std::mem::take(&mut report.rows);
        for row in &mut rows {
            let price_id = match row.disposition.as_str() {
                "ready_insert" => {
                    let id = insert(&tx, row, &mut report, &time)?;
                    report.imported += 1;
                    row.disposition = "inserted".into();
                    Some(id)
                }
                "ready_update" => {
                    let package = row
                        .package_id
                        .as_ref()
                        .ok_or_else(|| invalid("Missing price target."))?;
                    let cost =
                        price_history::current_on(&tx, package)?.and_then(|p| p.cost_price_minor);
                    let p = price_history::set_checked_on(
                        &tx,
                        &SetPackagePrice {
                            product_package_id: package.clone(),
                            selling_price_minor: row
                                .price_minor
                                .ok_or_else(|| invalid("Invalid source price."))?,
                            cost_price_minor: cost,
                            reason: Some("Active canonical source v2 update".into()),
                        },
                    )?;
                    report.updated += 1;
                    row.disposition = "updated".into();
                    Some(p.id)
                }
                "ready_reuse" | "already_imported" => {
                    report.reused += 1;
                    row.disposition = "reused".into();
                    None
                }
                _ => None,
            };
            tx.prepare_cached("INSERT INTO catalog_source_rows(run_id,source_row,fingerprint,identity_key,action,reason,product_id,package_id,price_history_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)")?.execute(params![report.id,row.number as i64,row.fingerprint,row.identity,row.disposition,json(&row.warnings)?,row.product_id,row.package_id,price_id])?;
        }
        report.rows = rows;
        tx.execute("UPDATE catalog_import_runs SET imported_rows=?2,status='complete',finished_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![report.id,report.imported as i64])?;
        tx.execute(
            "UPDATE catalog_source_runs SET reused_rows=?2,updated_rows=?3 WHERE run_id=?1",
            params![report.id, report.reused as i64, report.updated as i64],
        )?;
        tx.commit()?;
        Ok(())
    })();
    report.finished_at = Some(c.query_row(
        "SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now')",
        [],
        |r| r.get(0),
    )?);
    if let Err(e) = result {
        report.rows = original_rows;
        report.imported = 0;
        report.updated = 0;
        report.reused = 0;
        report.lookup_created.clear();
        report.lookup_reused.clear();
        report.failed = report.counts.get("ready").copied().unwrap_or(0);
        report.status = "failed".into();
        report.error = Some(e.message);
        for row in &mut report.rows {
            if row.disposition.starts_with("ready_") {
                row.disposition = "failed_rolled_back".into();
            }
        }
        let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
        persist_run(&tx, &report)?;
        tx.commit()?;
    } else {
        report.status = "complete".into();
    }
    report.processed = report.rows.len();
    report.duration_ms = start.elapsed().as_millis() as u64;
    report.rows_per_second = report.processed as f64 / start.elapsed().as_secs_f64().max(0.001);
    report.counts.insert("inserted".into(), report.imported);
    report.counts.insert("reused".into(), report.reused);
    report.counts.insert("updated".into(), report.updated);
    report.counts.insert("failed".into(), report.failed);
    save(&report)?;
    progress(&report);
    Ok(report)
}
