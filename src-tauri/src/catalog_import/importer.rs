use super::*;
use crate::db::{catalog_validation::normalize, connection::AppDb};
use rusqlite::{params, Connection, TransactionBehavior, MAIN_DB};
use std::{collections::HashMap, path::Path, time::Instant};
const BATCH: usize = 250;
fn lookup(
    c: &Connection,
    table: &str,
    value: &str,
    cache: &mut HashMap<String, String>,
) -> Result<Option<String>, AppError> {
    let key = normalize(value);
    if key.is_empty() {
        return Ok(None);
    }
    if let Some(id) = cache.get(&key) {
        return Ok(Some(id.clone()));
    }
    let id = uuid::Uuid::new_v4().to_string();
    c.prepare_cached(&format!(
        "INSERT INTO {table}(id,name,normalized_name) VALUES (?1,?2,?3)"
    ))?
    .execute(params![id, value, key])?;
    cache.insert(key, id.clone());
    Ok(Some(id))
}
fn optional(s: &str) -> Option<&str> {
    if normalize(s).is_empty() {
        None
    } else {
        Some(s)
    }
}
fn insert(
    c: &Connection,
    row: &Row,
    run: &str,
    time: &str,
    maps: &mut [HashMap<String, String>; 3],
) -> Result<(), AppError> {
    let f = &row.fields;
    let manufacturer = lookup(c, "manufacturers", &f[3], &mut maps[0])?;
    let category = lookup(c, "categories", &f[4], &mut maps[1])?;
    let route = lookup(c, "routes", &f[5], &mut maps[2])?;
    let product = uuid::Uuid::new_v4().to_string();
    let package = uuid::Uuid::new_v4().to_string();
    c.prepare_cached("INSERT INTO products(id,commercial_name_en,commercial_name_ar,scientific_name,normalized_name_en,normalized_name_ar,normalized_scientific_name,manufacturer_id,category_id,route_id,notes) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)")?.execute(params![product,optional(&f[0]),optional(&f[1]),optional(&f[2]),optional(&f[0]).map(normalize),optional(&f[1]).map(normalize),optional(&f[2]).map(normalize),manufacturer,category,route,"Imported from Egyptian catalog. Default Package is an import-derived placeholder; package size and ingredient relations are unverified."])?;
    c.prepare_cached("INSERT INTO product_packages(id,product_id,package_label,is_default) VALUES (?1,?2,'Default Package',1)")?.execute(params![package,product])?;
    c.prepare_cached("INSERT INTO product_price_history(id,product_package_id,selling_price_minor,effective_from,reason) VALUES (?1,?2,?3,?4,'Initial Egyptian catalog import')")?.execute(params![uuid::Uuid::new_v4().to_string(),package,row.price_minor,time])?;
    c.prepare_cached("INSERT INTO catalog_import_rows(source_id,fingerprint,identity_key,run_id,source_row,product_id,package_id) VALUES ('egyptian-drugs',?1,?2,?3,?4,?5,?6)")?.execute(params![row.fingerprint,row.identity,run,row.number as i64,product,package])?;
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
    if plan
        .report
        .profile
        .as_ref()
        .is_none_or(|p| p.hash != fresh.hash)
    {
        return Err(invalid("Source changed. Run a fresh dry run."));
    }
    let mut c = db.lock()?;
    if planner::snapshot(&c)? != plan.snapshot {
        return Err(invalid("Catalog changed. Run a fresh dry run."));
    }
    let mut report = plan.report;
    report.mode = "apply".into();
    report.status = "applying".into();
    report.report_path = output
        .join(format!("{}-apply.json", report.id))
        .to_string_lossy()
        .into_owned();
    let backups = output.join("backups");
    std::fs::create_dir_all(&backups)?;
    let backup = backups.join(format!("catalog-before-{}.sqlite3", report.id));
    if backup.exists() {
        return Err(invalid(
            "This plan has already been applied or attempted. Run a fresh dry run.",
        ));
    }
    c.backup(MAIN_DB, &backup, None)?;
    report.backup_path = Some(backup.to_string_lossy().into_owned());
    // Backup uses SQLite's online backup API, including committed WAL content.
    let time: String = c.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now')", [], |r| {
        r.get(0)
    })?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if planner::snapshot(&tx)? != plan.snapshot {
        return Err(invalid(
            "Catalog changed during backup. Run a fresh dry run.",
        ));
    }
    tx.execute("INSERT INTO catalog_import_runs(id,source_hash,source_path,started_at,total_rows,status,report_path,backup_path) VALUES (?1,?2,?3,?4,?5,'applying',?6,?7)",params![report.id,fresh.hash,fresh.path,time,fresh.total_rows as i64,report.report_path,report.backup_path])?;
    tx.commit()?;
    let version: i64 = c.query_row("PRAGMA data_version", [], |r| r.get(0))?;
    let mut maps: [HashMap<String, String>; 3] = Default::default();
    for (i, table) in ["manufacturers", "categories", "routes"].iter().enumerate() {
        let mut q = c.prepare(&format!("SELECT normalized_name,id FROM {table}"))?;
        maps[i] = q
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<_, _>>()?;
    }
    let initial_lengths = maps.each_ref().map(|m| m.len());
    let indexes: Vec<_> = report
        .rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r.disposition == "ready")
        .map(|(i, _)| i)
        .collect();
    let result = (|| -> Result<(), AppError> {
        for batch in indexes.chunks(BATCH) {
            let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let current: i64 = tx.query_row("PRAGMA data_version", [], |r| r.get(0))?;
            if version != current {
                return Err(invalid("Another connection changed the Catalog. Committed batches are safe; run a new dry run to resume."));
            }
            let mut batch_maps = maps.clone();
            for index in batch {
                insert(
                    &tx,
                    &report.rows[*index],
                    &report.id,
                    &time,
                    &mut batch_maps,
                )?;
            }
            tx.execute(
                "UPDATE catalog_import_runs SET imported_rows=?2 WHERE id=?1",
                params![report.id, (report.imported + batch.len()) as i64],
            )?;
            tx.commit()?;
            maps = batch_maps;
            for index in batch {
                report.rows[*index].disposition = "imported".into();
            }
            report.imported += batch.len();
            report.processed = report.imported;
            progress(&report);
        }
        Ok(())
    })();
    report.status = if result.is_ok() { "complete" } else { "failed" }.into();
    report.error = result.err().map(|e| e.message);
    if report.status == "failed" {
        for row in &mut report.rows {
            if row.disposition == "ready" {
                row.disposition = "not_imported_after_failure".into();
            }
        }
    }
    for (i, table) in ["manufacturers", "categories", "routes"].iter().enumerate() {
        report
            .lookup_created
            .insert(table.to_string(), maps[i].len() - initial_lengths[i]);
    }
    report.processed = report.rows.len();
    report.duration_ms = start.elapsed().as_millis() as u64;
    report.rows_per_second = report.imported as f64 / start.elapsed().as_secs_f64().max(0.001);
    c.execute("UPDATE catalog_import_runs SET status=?2,finished_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![report.id,report.status])?;
    save(&report)?;
    progress(&report);
    Ok(report)
}
