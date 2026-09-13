pub mod commands;
pub mod importer;
pub mod planner;
pub mod source;
use crate::db::catalog_validation::invalid;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub fn json<T: Serialize>(v: &T) -> Result<String, AppError> {
    serde_json::to_string(v).map_err(|_| invalid("Cannot serialize import report."))
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub path: String,
    pub hash: String,
    pub size: u64,
    pub total_rows: usize,
    pub columns: Vec<String>,
    pub profiled_at: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub number: usize,
    pub fields: Vec<String>,
    pub fingerprint: String,
    pub identity: String,
    pub disposition: String,
    pub warnings: Vec<String>,
    pub price_minor: Option<i64>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub id: String,
    pub mode: String,
    pub status: String,
    pub profile: Option<Profile>,
    pub counts: BTreeMap<String, usize>,
    pub lookup_created: BTreeMap<String, usize>,
    pub lookup_reused: BTreeMap<String, usize>,
    pub imported: usize,
    pub processed: usize,
    pub duration_ms: u64,
    pub rows_per_second: f64,
    pub report_path: String,
    pub backup_path: Option<String>,
    pub error: Option<String>,
    pub rows: Vec<Row>,
}
#[derive(Clone)]
pub struct Plan {
    pub report: Report,
    pub snapshot: String,
}
pub fn save(report: &Report) -> Result<(), AppError> {
    use std::{fs, io::Write, path::Path};
    let path = Path::new(&report.report_path);
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| invalid("Invalid report path."))?,
    )?;
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)?;
    f.write_all(json(report)?.as_bytes())?;
    f.sync_all()?;
    // Windows rename cannot replace an existing file. Reports are immutable snapshots.
    fs::rename(temporary, path)?;
    Ok(())
}
