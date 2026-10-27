use crate::{
    authorization::AuthorizationState,
    commands::branches::DatabaseState,
    db::{connection::AppDb, migrations},
    error::{AppError, ErrorCode},
};
use rusqlite::{params, Connection, OpenFlags, MAIN_DB};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::State;
use uuid::Uuid;

#[derive(Clone)]
pub struct DesktopState {
    backup_dir: PathBuf,
}
impl DesktopState {
    pub fn new(backup_dir: PathBuf) -> Self {
        Self { backup_dir }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalSettings {
    pub pharmacy_name: String,
    pub pharmacy_phone: String,
    pub pharmacy_address: String,
    pub registration_number: String,
    pub timezone: String,
    pub receipt_header: String,
    pub receipt_footer: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditQuery {
    pub search: Option<String>,
    pub event_type: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEvent {
    pub id: String,
    pub actor_user_id: String,
    pub actor_name: String,
    pub subject_user_id: Option<String>,
    pub subject_name: Option<String>,
    pub event_type: String,
    pub detail: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditPage {
    pub items: Vec<AuditEvent>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRecord {
    pub id: String,
    pub created_at: String,
    pub size_bytes: u64,
    pub schema_version: i64,
    pub integrity: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupStatus {
    pub managed_location: String,
    pub backups: Vec<BackupRecord>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub safety_backup_id: String,
    pub restart_required: bool,
}

fn valid_text(value: &str, max: usize, required: bool, label: &str) -> Result<String, AppError> {
    let value = value.trim();
    if (required && value.is_empty()) || value.chars().count() > max || value.contains('\0') {
        return Err(AppError::new(
            ErrorCode::Validation,
            format!("{label} is invalid."),
        ));
    }
    Ok(value.to_owned())
}

pub fn get_settings(db: &AppDb) -> Result<LocalSettings, AppError> {
    db.lock()?.query_row("SELECT pharmacy_name,pharmacy_phone,pharmacy_address,registration_number,timezone,receipt_header,receipt_footer,updated_at FROM local_settings WHERE singleton=1",[],|r| Ok(LocalSettings { pharmacy_name:r.get(0)?, pharmacy_phone:r.get(1)?, pharmacy_address:r.get(2)?, registration_number:r.get(3)?, timezone:r.get(4)?, receipt_header:r.get(5)?, receipt_footer:r.get(6)?, updated_at:r.get(7)? })).map_err(Into::into)
}

pub fn update_settings(
    db: &AppDb,
    value: &LocalSettings,
    actor: &str,
) -> Result<LocalSettings, AppError> {
    let name = valid_text(&value.pharmacy_name, 160, true, "Pharmacy name")?;
    let phone = valid_text(&value.pharmacy_phone, 64, false, "Pharmacy phone")?;
    let address = valid_text(&value.pharmacy_address, 500, false, "Pharmacy address")?;
    let registration = valid_text(
        &value.registration_number,
        100,
        false,
        "Registration number",
    )?;
    let header = valid_text(&value.receipt_header, 160, true, "Receipt header")?;
    let footer = valid_text(&value.receipt_footer, 500, false, "Receipt footer")?;
    if !matches!(value.timezone.as_str(), "Africa/Cairo" | "UTC") {
        return Err(AppError::new(
            ErrorCode::Validation,
            "Time zone is invalid.",
        ));
    }
    db.lock()?.execute("UPDATE local_settings SET pharmacy_name=?1,pharmacy_phone=?2,pharmacy_address=?3,registration_number=?4,timezone=?5,receipt_header=?6,receipt_footer=?7,updated_by_user_id=?8,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE singleton=1",params![name,phone,address,registration,value.timezone,header,footer,actor])?;
    get_settings(db)
}

fn date(value: Option<&str>, label: &str) -> Result<Option<String>, AppError> {
    let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    if value.len() != 10
        || value.as_bytes()[4] != b'-'
        || value.as_bytes()[7] != b'-'
        || !value
            .chars()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
    {
        return Err(AppError::new(
            ErrorCode::Validation,
            format!("{label} must use YYYY-MM-DD."),
        ));
    }
    Ok(Some(value.to_owned()))
}

pub fn audit_events(db: &AppDb, q: &AuditQuery) -> Result<AuditPage, AppError> {
    let search = valid_text(q.search.as_deref().unwrap_or(""), 200, false, "Search")?;
    let event_type = valid_text(
        q.event_type.as_deref().unwrap_or(""),
        100,
        false,
        "Event type",
    )?;
    let from = date(q.from_date.as_deref(), "From date")?;
    let to = date(q.to_date.as_deref(), "To date")?;
    if from > to && to.is_some() {
        return Err(AppError::new(
            ErrorCode::Validation,
            "From date must not be after to date.",
        ));
    }
    let limit = q.limit.unwrap_or(50);
    let offset = q.offset.unwrap_or(0);
    if !(1..=200).contains(&limit) || offset < 0 {
        return Err(AppError::new(
            ErrorCode::Validation,
            "Audit pagination is invalid.",
        ));
    }
    let filter="(?1='' OR instr(lower(e.event_type||' '||e.detail||' '||a.username||' '||a.display_name||' '||coalesce(s.username,'')||' '||coalesce(s.display_name,'')),lower(?1))>0) AND (?2='' OR e.event_type=?2) AND (?3 IS NULL OR substr(e.created_at,1,10)>=?3) AND (?4 IS NULL OR substr(e.created_at,1,10)<=?4)";
    let c = db.lock()?;
    let total=c.query_row(&format!("SELECT count(*) FROM security_events e JOIN users a ON a.id=e.actor_user_id LEFT JOIN users s ON s.id=e.subject_user_id WHERE {filter}"),params![search,event_type,from,to],|r|r.get(0))?;
    let mut stmt=c.prepare(&format!("SELECT e.id,e.actor_user_id,a.display_name,e.subject_user_id,s.display_name,e.event_type,e.detail,e.created_at FROM security_events e JOIN users a ON a.id=e.actor_user_id LEFT JOIN users s ON s.id=e.subject_user_id WHERE {filter} ORDER BY e.created_at DESC,e.id DESC LIMIT ?5 OFFSET ?6"))?;
    let items = stmt
        .query_map(params![search, event_type, from, to, limit, offset], |r| {
            Ok(AuditEvent {
                id: r.get(0)?,
                actor_user_id: r.get(1)?,
                actor_name: r.get(2)?,
                subject_user_id: r.get(3)?,
                subject_name: r.get(4)?,
                event_type: r.get(5)?,
                detail: r.get(6)?,
                created_at: r.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(AuditPage { items, total })
}

fn backup_path(state: &DesktopState, id: &str) -> Result<PathBuf, AppError> {
    if id.contains('/')
        || id.contains('\\')
        || !id.starts_with("m2-health-")
        || !id.ends_with(".db")
    {
        return Err(AppError::new(
            ErrorCode::Validation,
            "Backup identity is invalid.",
        ));
    }
    let path = state.backup_dir.join(id);
    if !path.is_file() {
        return Err(AppError::new(
            ErrorCode::NotFound,
            "Backup file was not found.",
        ));
    }
    Ok(path)
}

const REQUIRED_SCHEMA_OBJECTS: &[(i64, &str)] = &[
    (1, "branches"),
    (2, "products"),
    (4, "inventory_movements"),
    (5, "inventory_documents"),
    (6, "stock_count_sessions"),
    (8, "inventory_alerts"),
    (9, "sales"),
    (10, "suppliers"),
    (10, "customers"),
    (11, "purchase_orders"),
    (12, "purchase_receipts"),
    (13, "purchase_returns"),
    (15, "finance_journals"),
    (16, "users"),
    (16, "user_branch_memberships"),
    (17, "security_events"),
    (18, "sync_outbox"),
    (19, "sync_conflicts"),
    (22, "local_settings"),
    (23, "local_license_state"),
];

fn validate_connection(c: &Connection, expected_version: Option<usize>) -> Result<usize, AppError> {
    let integrity: String = c
        .query_row("PRAGMA quick_check", [], |r| r.get(0))
        .map_err(|_| {
            AppError::new(
                ErrorCode::Validation,
                "The selected backup failed its integrity check.",
            )
        })?;
    if integrity != "ok" {
        return Err(AppError::new(
            ErrorCode::Validation,
            "The selected backup failed its integrity check.",
        ));
    }
    let foreign_key_errors: i64 = c
        .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })
        .map_err(|_| {
            AppError::new(
                ErrorCode::Validation,
                "The selected backup failed its relational integrity check.",
            )
        })?;
    if foreign_key_errors != 0 {
        return Err(AppError::new(
            ErrorCode::Validation,
            "The selected backup failed its relational integrity check.",
        ));
    }
    let mut stmt = c
        .prepare("SELECT version,name FROM _schema_version ORDER BY version")
        .map_err(|_| {
            AppError::new(
                ErrorCode::Validation,
                "The selected file is not an M² Health backup.",
            )
        })?;
    let applied = stmt
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    if applied.is_empty()
        || expected_version.is_some_and(|expected| applied.len() != expected)
        || applied.len() > migrations::MIGRATIONS.len()
        || applied.iter().enumerate().any(|(i, (v, n))| {
            migrations::MIGRATIONS
                .get(i)
                .is_none_or(|m| m.version != *v || m.name != n)
        })
    {
        return Err(AppError::new(
            ErrorCode::Validation,
            "The backup schema is not compatible with this version of M² Health.",
        ));
    }
    for (_, object) in REQUIRED_SCHEMA_OBJECTS
        .iter()
        .filter(|(version, _)| *version <= applied.len() as i64)
    {
        let exists: i64 = c.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type IN ('table','view') AND name=?1",
            [object],
            |r| r.get(0),
        )?;
        if exists != 1 {
            return Err(AppError::new(
                ErrorCode::Validation,
                "The selected backup is missing required M² Health schema objects.",
            ));
        }
    }
    Ok(applied.len())
}

fn inspect_backup(path: &Path) -> Result<BackupRecord, AppError> {
    let c = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| {
        AppError::new(
            ErrorCode::Validation,
            "The selected file is not a readable SQLite backup.",
        )
    })?;
    let schema_version = validate_connection(&c, None)?;
    let meta = fs::metadata(path)?;
    let id = path
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or_else(|| AppError::new(ErrorCode::Validation, "Backup identity is invalid."))?
        .to_owned();
    let created_at = id
        .strip_prefix("m2-health-backup-")
        .or_else(|| id.strip_prefix("m2-health-safety-"))
        .and_then(|v| v.split('-').next())
        .unwrap_or("unknown")
        .to_owned();
    Ok(BackupRecord {
        id,
        created_at,
        size_bytes: meta.len(),
        schema_version: schema_version as i64,
        integrity: "ok".into(),
    })
}

fn list_backups(state: &DesktopState) -> Result<Vec<BackupRecord>, AppError> {
    fs::create_dir_all(&state.backup_dir)?;
    let mut rows = Vec::new();
    for entry in fs::read_dir(&state.backup_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|v| v.to_str()) == Some("db") {
            if let Ok(row) = inspect_backup(&path) {
                rows.push(row)
            }
        }
    }
    rows.sort_by(|a, b| b.id.cmp(&a.id));
    Ok(rows)
}

pub fn create_backup(
    db: &AppDb,
    state: &DesktopState,
    kind: &str,
) -> Result<BackupRecord, AppError> {
    fs::create_dir_all(&state.backup_dir)?;
    let stamp: String =
        db.lock()?
            .query_row("SELECT strftime('%Y%m%dT%H%M%SZ','now')", [], |r| r.get(0))?;
    let id = format!("m2-health-{kind}-{stamp}-{}.db", Uuid::new_v4().simple());
    let path = state.backup_dir.join(&id);
    db.lock()?.backup(MAIN_DB, &path, None)?;
    match inspect_backup(&path) {
        Ok(row) => Ok(row),
        Err(error) => {
            let _ = fs::remove_file(&path);
            Err(error)
        }
    }
}

pub fn backup_status(state: &DesktopState) -> Result<BackupStatus, AppError> {
    Ok(BackupStatus {
        managed_location: state.backup_dir.display().to_string(),
        backups: list_backups(state)?,
    })
}

pub fn restore_backup(
    db: &AppDb,
    state: &DesktopState,
    id: &str,
    confirmation: &str,
    reason: &str,
) -> Result<RestoreResult, AppError> {
    restore_backup_with_validation(db, state, id, confirmation, reason, |connection| {
        validate_connection(connection, Some(migrations::MIGRATIONS.len())).map(|_| ())
    })
}

fn restore_backup_with_validation<F>(
    db: &AppDb,
    state: &DesktopState,
    id: &str,
    confirmation: &str,
    reason: &str,
    validate_restored: F,
) -> Result<RestoreResult, AppError>
where
    F: FnOnce(&Connection) -> Result<(), AppError>,
{
    if confirmation != "RESTORE" {
        return Err(AppError::new(
            ErrorCode::Validation,
            "Type RESTORE to confirm replacement of local data.",
        ));
    }
    valid_text(reason, 500, true, "Restore reason")?;
    let source = backup_path(state, id)?;
    inspect_backup(&source)?;
    let safety = create_backup(db, state, "safety")?;
    let license_snapshot = {
        let connection = db.lock()?;
        crate::licensing::snapshot(&connection)?
    };
    let restore_result = (|| {
        let mut c = db.lock()?;
        c.restore(MAIN_DB, &source, None::<fn(rusqlite::backup::Progress)>)?;
        migrations::run(&mut c)?;
        validate_restored(&c)?;
        crate::licensing::restore_snapshot(&c, &license_snapshot)?;
        Ok::<(), AppError>(())
    })();
    if restore_result.is_err() {
        let safety_path = state.backup_dir.join(&safety.id);
        let rollback_result = (|| {
            let mut c = db.lock()?;
            c.restore(
                MAIN_DB,
                &safety_path,
                None::<fn(rusqlite::backup::Progress)>,
            )?;
            migrations::run(&mut c)?;
            validate_connection(&c, Some(migrations::MIGRATIONS.len()))?;
            Ok::<(), AppError>(())
        })();
        if rollback_result.is_err() {
            return Err(AppError::new(
                ErrorCode::Database,
                format!(
                    "Restore and automatic rollback both failed. Safety backup {} remains available for recovery.",
                    safety.id
                ),
            ));
        }
        return Err(AppError::new(
            ErrorCode::Database,
            format!(
                "Restore failed. Automatic rollback from safety backup {} was verified and existing data was retained.",
                safety.id
            ),
        ));
    }
    Ok(RestoreResult {
        safety_backup_id: safety.id,
        restart_required: true,
    })
}

#[tauri::command]
pub fn get_local_settings(state: State<'_, DatabaseState>) -> Result<LocalSettings, AppError> {
    get_settings(state.db()?)
}
#[tauri::command]
pub fn update_local_settings(
    state: State<'_, DatabaseState>,
    authz: State<'_, AuthorizationState>,
    input: LocalSettings,
) -> Result<LocalSettings, AppError> {
    update_settings(state.db()?, &input, &authz.actor()?)
}
#[tauri::command]
pub fn list_security_audit_events(
    state: State<'_, DatabaseState>,
    query: AuditQuery,
) -> Result<AuditPage, AppError> {
    audit_events(state.db()?, &query)
}
#[tauri::command]
pub fn get_local_backup_status(state: State<'_, DesktopState>) -> Result<BackupStatus, AppError> {
    backup_status(&state)
}
#[tauri::command]
pub fn create_local_backup(
    db: State<'_, DatabaseState>,
    state: State<'_, DesktopState>,
) -> Result<BackupRecord, AppError> {
    create_backup(db.db()?, &state, "backup")
}
#[tauri::command]
pub fn validate_local_backup(
    state: State<'_, DesktopState>,
    id: String,
) -> Result<BackupRecord, AppError> {
    inspect_backup(&backup_path(&state, &id)?)
}
#[tauri::command]
pub fn restore_local_backup(
    db: State<'_, DatabaseState>,
    state: State<'_, DesktopState>,
    id: String,
    confirmation: String,
    reason: String,
) -> Result<RestoreResult, AppError> {
    restore_backup(db.db()?, &state, &id, &confirmation, &reason)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{
        self,
        models::{BootstrapInput, CreateUser},
        AuthState,
    };
    use tempfile::tempdir;

    fn bootstrap(db: &AppDb) -> String {
        auth::bootstrap(
            db,
            &AuthState::default(),
            &BootstrapInput {
                username: "desktop.admin".into(),
                display_name: "Desktop Admin".into(),
                password: "Desktop!Strong2026".into(),
            },
        )
        .unwrap()
        .user
        .id
    }

    fn changed_settings(name: &str) -> LocalSettings {
        LocalSettings {
            pharmacy_name: name.into(),
            pharmacy_phone: "01000000000".into(),
            pharmacy_address: "Cairo".into(),
            registration_number: "REG-1".into(),
            timezone: "Africa/Cairo".into(),
            receipt_header: name.into(),
            receipt_footer: "Thank you".into(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn settings_are_migrated_validated_and_persisted() {
        let db = AppDb::in_memory().unwrap();
        let actor = bootstrap(&db);
        assert_eq!(
            get_settings(&db).unwrap().pharmacy_name,
            "M² Health Pharmacy"
        );

        let saved = update_settings(&db, &changed_settings("North Pharmacy"), &actor).unwrap();
        assert_eq!(saved.pharmacy_name, "North Pharmacy");
        assert_eq!(get_settings(&db).unwrap().receipt_footer, "Thank you");

        let mut invalid = changed_settings("");
        invalid.timezone = "local".into();
        assert_eq!(
            update_settings(&db, &invalid, &actor).unwrap_err().code,
            ErrorCode::Validation
        );
    }

    #[test]
    fn audit_query_returns_immutable_security_events() {
        let db = AppDb::in_memory().unwrap();
        let state = AuthState::default();
        let admin = auth::bootstrap(
            &db,
            &state,
            &BootstrapInput {
                username: "audit.admin".into(),
                display_name: "Audit Admin".into(),
                password: "Audit!Strong2026".into(),
            },
        )
        .unwrap();
        auth::create(
            &db,
            &CreateUser {
                username: "audit.viewer".into(),
                display_name: "Audit Viewer".into(),
                password: "Viewer!Strong2026".into(),
                role_codes: vec!["VIEWER".into()],
                branch_ids: Vec::new(),
                primary_branch_id: None,
            },
            &admin.user.id,
        )
        .unwrap();

        let page = audit_events(
            &db,
            &AuditQuery {
                event_type: Some("USER_CREATED".into()),
                ..AuditQuery::default()
            },
        )
        .unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].actor_name, "Audit Admin");
        assert_eq!(page.items[0].subject_name.as_deref(), Some("Audit Viewer"));

        let error = db
            .lock()
            .unwrap()
            .execute("DELETE FROM security_events", [])
            .unwrap_err();
        assert!(error.to_string().contains("immutable"));
    }

    #[test]
    fn restore_requires_confirmation_and_keeps_a_safety_backup() {
        let root = tempdir().unwrap();
        let db_path = root.path().join("isolated-test.db");
        let db = AppDb::open(&db_path).unwrap();
        let actor = bootstrap(&db);
        let state = DesktopState::new(root.path().join("backups"));

        update_settings(&db, &changed_settings("Before Backup"), &actor).unwrap();
        let backup = create_backup(&db, &state, "backup").unwrap();
        assert_eq!(backup.schema_version, migrations::MIGRATIONS.len() as i64);
        crate::licensing::start_trial(&db).unwrap();
        update_settings(&db, &changed_settings("After Backup"), &actor).unwrap();

        assert_eq!(
            restore_backup(&db, &state, &backup.id, "NO", "test restore")
                .unwrap_err()
                .code,
            ErrorCode::Validation
        );
        assert_eq!(get_settings(&db).unwrap().pharmacy_name, "After Backup");

        let result = restore_backup(&db, &state, &backup.id, "RESTORE", "test restore").unwrap();
        assert!(result.restart_required);
        assert!(result.safety_backup_id.starts_with("m2-health-safety-"));
        assert_eq!(get_settings(&db).unwrap().pharmacy_name, "Before Backup");
        assert_eq!(crate::licensing::status(&db).unwrap().state, "TRIAL");
        assert!(state.backup_dir.join(result.safety_backup_id).is_file());
    }

    #[test]
    fn failed_restore_reapplies_and_verifies_the_safety_backup() {
        let root = tempdir().unwrap();
        let db = AppDb::open(&root.path().join("isolated-rollback-test.db")).unwrap();
        let actor = bootstrap(&db);
        let state = DesktopState::new(root.path().join("backups"));

        update_settings(&db, &changed_settings("Backup Candidate"), &actor).unwrap();
        let backup = create_backup(&db, &state, "backup").unwrap();
        update_settings(&db, &changed_settings("Live Data"), &actor).unwrap();

        let error = restore_backup_with_validation(
            &db,
            &state,
            &backup.id,
            "RESTORE",
            "exercise verified rollback",
            |_| {
                Err(AppError::new(
                    ErrorCode::Database,
                    "Injected post-restore validation failure.",
                ))
            },
        )
        .unwrap_err();
        assert!(error.message.contains("Automatic rollback"));
        assert!(error.message.contains("was verified"));
        assert_eq!(get_settings(&db).unwrap().pharmacy_name, "Live Data");
        assert_eq!(backup_status(&state).unwrap().backups.len(), 2);
    }

    #[test]
    fn invalid_backup_identity_and_content_are_rejected() {
        let root = tempdir().unwrap();
        let state = DesktopState::new(root.path().to_path_buf());
        assert_eq!(
            backup_path(&state, "..\\development.db").unwrap_err().code,
            ErrorCode::Validation
        );

        let path = root.path().join("m2-health-backup-invalid.db");
        fs::write(&path, b"not sqlite").unwrap();
        assert_eq!(
            inspect_backup(&path).unwrap_err().code,
            ErrorCode::Validation
        );

        let forged = root.path().join("m2-health-backup-forged.db");
        let connection = Connection::open(&forged).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE _schema_version(version INTEGER PRIMARY KEY, name TEXT NOT NULL);\
                 INSERT INTO _schema_version(version,name) VALUES(1,'branches');",
            )
            .unwrap();
        drop(connection);
        let error = inspect_backup(&forged).unwrap_err();
        assert_eq!(error.code, ErrorCode::Validation);
        assert!(error.message.contains("required M² Health schema objects"));
    }
}
