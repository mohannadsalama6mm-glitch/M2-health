//! Minimal privacy-safe local support diagnostics for Early Access.

use crate::{
    commands::branches::DatabaseState,
    db::{connection::AppDb, migrations},
    error::{AppError, ErrorCode},
    licensing,
};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::State;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const RELEASE_CHANNEL: &str = "early-access";
pub const BUILD_ID: &str = match option_env!("M2_BUILD_ID") {
    Some(value) => value,
    None => "local",
};

pub struct DiagnosticsState {
    directory: PathBuf,
    log_path: PathBuf,
}

impl DiagnosticsState {
    pub fn new(data_directory: PathBuf) -> Self {
        let directory = data_directory.join("diagnostics");
        let log_path = directory.join("m2-health-support.log");
        Self {
            directory,
            log_path,
        }
    }

    pub fn record(&self, event: &str, context: &str) {
        if fs::create_dir_all(&self.directory).is_err() {
            return;
        }
        let timestamp = unix_time().unwrap_or_default();
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = writeln!(
                file,
                "timestamp={timestamp} event={} version={} build={} context={}",
                safe_token(event),
                APP_VERSION,
                safe_token(BUILD_ID),
                safe_context(context)
            );
        }
    }
}

fn safe_token(value: &str) -> String {
    value
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
        .take(80)
        .collect()
}

fn safe_context(value: &str) -> String {
    value
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, ' ' | '_' | '-' | '.')
        })
        .take(160)
        .collect()
}

fn unix_time() -> Result<u64, AppError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .map_err(|_| AppError::new(ErrorCode::Internal, "The system clock is unavailable."))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseInfo {
    version: &'static str,
    channel: &'static str,
    build_id: &'static str,
    schema_version: usize,
    updater_enabled: bool,
    signed_build: bool,
}

pub fn release_info() -> ReleaseInfo {
    ReleaseInfo {
        version: APP_VERSION,
        channel: RELEASE_CHANNEL,
        build_id: BUILD_ID,
        schema_version: migrations::MIGRATIONS.len(),
        updater_enabled: false,
        signed_build: false,
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticsExport {
    generated_at: u64,
    release: ReleaseInfo,
    license_state: String,
    license_can_operate: bool,
    database_schema_version: i64,
    foreign_keys_enabled: bool,
    integrity_check: String,
    recent_application_log: Vec<String>,
    privacy_notice: &'static str,
}

pub fn export(db: &AppDb, state: &DiagnosticsState) -> Result<String, AppError> {
    fs::create_dir_all(&state.directory)?;
    let connection = db.lock()?;
    let schema_version: i64 = connection.query_row(
        "SELECT coalesce(max(version),0) FROM _schema_version",
        [],
        |row| row.get(0),
    )?;
    let foreign_keys: i64 = connection.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;
    let integrity: String = connection.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    drop(connection);
    let license = licensing::status(db)?;
    let mut recent_log = fs::read_to_string(&state.log_path)
        .unwrap_or_default()
        .lines()
        .rev()
        .take(200)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    recent_log.reverse();
    let generated_at = unix_time()?;
    let report = DiagnosticsExport {
        generated_at,
        release: release_info(),
        license_state: license.state,
        license_can_operate: license.can_operate,
        database_schema_version: schema_version,
        foreign_keys_enabled: foreign_keys == 1,
        integrity_check: integrity,
        recent_application_log: recent_log,
        privacy_notice: "Contains release, schema, license-mode, integrity, and application lifecycle metadata only. No credentials or pharmacy records are exported.",
    };
    let path = state
        .directory
        .join(format!("m2-health-diagnostics-{generated_at}.json"));
    fs::write(
        &path,
        serde_json::to_vec_pretty(&report).map_err(|_| {
            AppError::new(
                ErrorCode::Internal,
                "The diagnostics report could not be encoded.",
            )
        })?,
    )?;
    state.record(
        "diagnostics_exported",
        "privacy safe support report created",
    );
    Ok(path.display().to_string())
}

#[tauri::command]
pub fn get_release_info() -> ReleaseInfo {
    release_info()
}

#[tauri::command]
pub fn export_diagnostics(
    database: State<'_, DatabaseState>,
    diagnostics: State<'_, DiagnosticsState>,
) -> Result<String, AppError> {
    export(database.db()?, &diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::AppDb;
    use tempfile::tempdir;

    #[test]
    fn export_contains_release_health_but_no_domain_rows_or_credentials() {
        let root = tempdir().unwrap();
        let db = AppDb::in_memory().unwrap();
        let state = DiagnosticsState::new(root.path().to_owned());
        state.record("application_started", "database ready");
        let path = export(&db, &state).unwrap();
        let report = fs::read_to_string(path).unwrap();
        assert!(report.contains(APP_VERSION));
        assert!(report.contains("databaseSchemaVersion"));
        assert!(report.contains("privacyNotice"));
        assert!(!report.to_ascii_lowercase().contains("password"));
        assert!(!report.contains("signedPayload"));
    }
}
