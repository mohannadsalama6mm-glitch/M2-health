//! Phase 9A.1 device-scoped trial and signed-license foundation.
//!
//! License enforcement never deletes pharmacy data. Expired, suspended, and
//! revoked installations retain authenticated read access and backup recovery,
//! while operational mutations are denied at the central IPC boundary.

use crate::{
    commands::branches::DatabaseState,
    db::connection::AppDb,
    error::{AppError, ErrorCode},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ring::signature::{UnparsedPublicKey, ED25519};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;
use uuid::Uuid;

pub const TRIAL_SECONDS: i64 = 7 * 24 * 60 * 60;
pub const DEFAULT_OFFLINE_GRACE_SECONDS: i64 = 3 * 24 * 60 * 60;
const CLOCK_SKEW_SECONDS: i64 = 5 * 60;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LicenseClaims {
    version: u8,
    license_id: String,
    device_id: String,
    state: String,
    issued_at: i64,
    expires_at: Option<i64>,
    revalidate_after: i64,
    offline_grace_days: i64,
}

#[derive(Clone, Debug)]
struct StoredLicense {
    device_id: String,
    license_id: Option<String>,
    kind: String,
    activated_at: Option<i64>,
    expires_at: Option<i64>,
    last_validated_at: Option<i64>,
    next_revalidation_at: Option<i64>,
    offline_grace_seconds: i64,
    max_observed_at: i64,
    signed_payload: Option<String>,
    signature: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct LicenseSnapshot(StoredLicense);

pub(crate) fn snapshot(connection: &Connection) -> Result<LicenseSnapshot, AppError> {
    Ok(LicenseSnapshot(row(connection)?))
}

pub(crate) fn restore_snapshot(
    connection: &Connection,
    snapshot: &LicenseSnapshot,
) -> Result<(), AppError> {
    let current_device: String = connection.query_row(
        "SELECT device_id FROM local_device_identity WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    if current_device != snapshot.0.device_id {
        return Err(AppError::new(
            ErrorCode::Validation,
            "The backup belongs to a different device identity.",
        ));
    }
    let value = &snapshot.0;
    connection.execute(
        "UPDATE local_license_state SET license_id=?1,license_kind=?2,activated_at=?3,expires_at=?4,last_validated_at=?5,next_revalidation_at=?6,offline_grace_seconds=?7,max_observed_at=?8,signed_payload=?9,signature=?10,updated_at=max(updated_at,?8) WHERE singleton=1 AND device_id=?11",
        params![value.license_id,value.kind,value.activated_at,value.expires_at,value.last_validated_at,value.next_revalidation_at,value.offline_grace_seconds,value.max_observed_at,value.signed_payload,value.signature,value.device_id],
    )?;
    Ok(())
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LicenseStatus {
    pub state: String,
    pub device_id: String,
    pub activated_at: Option<i64>,
    pub expires_at: Option<i64>,
    pub last_validated_at: Option<i64>,
    pub next_revalidation_at: Option<i64>,
    pub offline_grace_ends_at: Option<i64>,
    pub days_remaining: Option<i64>,
    pub can_operate: bool,
    pub read_only: bool,
    pub requires_activation: bool,
    pub reason: String,
}

fn now() -> Result<i64, AppError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs() as i64)
        .map_err(|_| {
            AppError::new(
                ErrorCode::Internal,
                "The system clock is unavailable. Correct the clock and retry.",
            )
        })
}

fn row(connection: &Connection) -> Result<StoredLicense, AppError> {
    connection
        .query_row(
            "SELECT device_id,license_id,license_kind,activated_at,expires_at,last_validated_at,next_revalidation_at,offline_grace_seconds,max_observed_at,signed_payload,signature FROM local_license_state WHERE singleton=1",
            [],
            |r| {
                Ok(StoredLicense {
                    device_id: r.get(0)?,
                    license_id: r.get(1)?,
                    kind: r.get(2)?,
                    activated_at: r.get(3)?,
                    expires_at: r.get(4)?,
                    last_validated_at: r.get(5)?,
                    next_revalidation_at: r.get(6)?,
                    offline_grace_seconds: r.get(7)?,
                    max_observed_at: r.get(8)?,
                    signed_payload: r.get(9)?,
                    signature: r.get(10)?,
                })
            },
        )
        .map_err(Into::into)
}

fn ceil_days(seconds: i64) -> i64 {
    (seconds.max(0) + 86_399) / 86_400
}

fn evaluate(value: &StoredLicense, requested_now: i64) -> LicenseStatus {
    let observed = requested_now.max(value.max_observed_at);
    let base = |state: &str, can_operate: bool, read_only: bool, reason: &str| LicenseStatus {
        state: state.to_owned(),
        device_id: value.device_id.clone(),
        activated_at: value.activated_at,
        expires_at: value.expires_at,
        last_validated_at: value.last_validated_at,
        next_revalidation_at: value.next_revalidation_at,
        offline_grace_ends_at: value
            .next_revalidation_at
            .map(|at| at + value.offline_grace_seconds),
        days_remaining: value
            .expires_at
            .map(|at| ceil_days(at.saturating_sub(observed))),
        can_operate,
        read_only,
        requires_activation: state == "UNACTIVATED",
        reason: reason.to_owned(),
    };

    match value.kind.as_str() {
        "UNACTIVATED" => base(
            "UNACTIVATED",
            false,
            false,
            "Start the seven-day trial or enter a signed manual activation code.",
        ),
        "TRIAL" => match value.expires_at {
            Some(expiry) if observed < expiry => base(
                "TRIAL",
                true,
                false,
                "The local Early Access trial is active.",
            ),
            _ => base(
                "EXPIRED",
                false,
                true,
                "The trial expired. Existing pharmacy data remains available read-only.",
            ),
        },
        "ACTIVE" => {
            if value.expires_at.is_some_and(|expiry| observed >= expiry) {
                return base(
                    "EXPIRED",
                    false,
                    true,
                    "The license expired. Existing pharmacy data remains available read-only.",
                );
            }
            if let Some(revalidate_at) = value.next_revalidation_at {
                if observed > revalidate_at {
                    let grace_end = revalidate_at + value.offline_grace_seconds;
                    if observed <= grace_end {
                        return base(
                            "OFFLINE_GRACE",
                            true,
                            false,
                            "License revalidation is overdue; the device is operating within its offline grace period.",
                        );
                    }
                    return base(
                        "EXPIRED",
                        false,
                        true,
                        "The offline grace period ended. Revalidate or renew to resume operations.",
                    );
                }
            }
            base(
                "ACTIVE",
                true,
                false,
                "The signed manual license is active.",
            )
        }
        "SUSPENDED" => base(
            "SUSPENDED",
            false,
            true,
            "The license is suspended. Existing pharmacy data remains available read-only.",
        ),
        "REVOKED" => base(
            "REVOKED",
            false,
            true,
            "This device was revoked. Existing pharmacy data remains available read-only.",
        ),
        _ => base(
            "INVALID",
            false,
            true,
            "The cached license state is invalid. Existing data remains available read-only.",
        ),
    }
}

fn status_at(db: &AppDb, requested_now: i64) -> Result<LicenseStatus, AppError> {
    let connection = db.lock()?;
    let current = row(&connection)?;
    if requested_now > current.max_observed_at {
        connection.execute(
            "UPDATE local_license_state SET max_observed_at=?1,updated_at=?1 WHERE singleton=1",
            [requested_now],
        )?;
    }
    Ok(evaluate(&current, requested_now))
}

pub fn status(db: &AppDb) -> Result<LicenseStatus, AppError> {
    status_at(db, now()?)
}

fn event(connection: &Connection, kind: &str, detail: &str, at: i64) -> Result<(), AppError> {
    connection.execute(
        "INSERT INTO local_license_events(id,event_type,occurred_at,detail) VALUES(?1,?2,?3,?4)",
        params![Uuid::new_v4().to_string(), kind, at, detail],
    )?;
    Ok(())
}

fn start_trial_at(db: &AppDb, requested_now: i64) -> Result<LicenseStatus, AppError> {
    let mut connection = db.lock()?;
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let changed = tx.execute(
        "UPDATE local_license_state SET license_kind='TRIAL',activated_at=?1,expires_at=?2,last_validated_at=?1,next_revalidation_at=NULL,offline_grace_seconds=?3,max_observed_at=max(max_observed_at,?1),signed_payload=NULL,signature=NULL,updated_at=?1 WHERE singleton=1 AND license_kind='UNACTIVATED'",
        params![requested_now, requested_now + TRIAL_SECONDS, DEFAULT_OFFLINE_GRACE_SECONDS],
    )?;
    if changed != 1 {
        return Err(AppError::new(
            ErrorCode::Conflict,
            "The trial can be started only once on this device.",
        ));
    }
    event(
        &tx,
        "TRIAL_STARTED",
        "Seven-day Early Access trial started.",
        requested_now,
    )?;
    tx.commit()?;
    drop(connection);
    status_at(db, requested_now)
}

pub fn start_trial(db: &AppDb) -> Result<LicenseStatus, AppError> {
    start_trial_at(db, now()?)
}

fn parse_and_verify(
    activation_code: &str,
    public_key: &[u8],
    device_id: &str,
    requested_now: i64,
) -> Result<(LicenseClaims, String, String), AppError> {
    let (payload_text, signature_text) =
        activation_code.trim().split_once('.').ok_or_else(|| {
            AppError::new(
                ErrorCode::Validation,
                "The activation code format is invalid.",
            )
        })?;
    if payload_text.is_empty() || signature_text.is_empty() || signature_text.contains('.') {
        return Err(AppError::new(
            ErrorCode::Validation,
            "The activation code format is invalid.",
        ));
    }
    let payload = URL_SAFE_NO_PAD.decode(payload_text).map_err(|_| {
        AppError::new(
            ErrorCode::Validation,
            "The activation code payload is invalid.",
        )
    })?;
    let signature = URL_SAFE_NO_PAD.decode(signature_text).map_err(|_| {
        AppError::new(
            ErrorCode::Validation,
            "The activation code signature is invalid.",
        )
    })?;
    UnparsedPublicKey::new(&ED25519, public_key)
        .verify(&payload, &signature)
        .map_err(|_| {
            AppError::new(
                ErrorCode::Validation,
                "The activation code signature could not be verified.",
            )
        })?;
    let claims: LicenseClaims = serde_json::from_slice(&payload).map_err(|_| {
        AppError::new(
            ErrorCode::Validation,
            "The activation code claims are invalid.",
        )
    })?;
    if claims.version != 1
        || claims.device_id != device_id
        || claims.license_id.trim().is_empty()
        || !matches!(claims.state.as_str(), "ACTIVE" | "SUSPENDED" | "REVOKED")
        || claims.issued_at > requested_now + CLOCK_SKEW_SECONDS
        || claims.revalidate_after < claims.issued_at
        || !(0..=30).contains(&claims.offline_grace_days)
        || claims
            .expires_at
            .is_some_and(|expiry| expiry <= claims.issued_at)
    {
        return Err(AppError::new(
            ErrorCode::Validation,
            "The activation code is not valid for this device or release policy.",
        ));
    }
    Ok((claims, payload_text.to_owned(), signature_text.to_owned()))
}

fn activate_at_with_key(
    db: &AppDb,
    activation_code: &str,
    public_key: &[u8],
    requested_now: i64,
    revalidation: bool,
) -> Result<LicenseStatus, AppError> {
    let mut connection = db.lock()?;
    let device_id: String = connection.query_row(
        "SELECT device_id FROM local_license_state WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    let (claims, payload, signature) =
        parse_and_verify(activation_code, public_key, &device_id, requested_now)?;
    let previous_id: Option<String> = connection
        .query_row(
            "SELECT license_id FROM local_license_state WHERE singleton=1",
            [],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    if revalidation
        && previous_id
            .as_deref()
            .is_some_and(|id| id != claims.license_id)
    {
        return Err(AppError::new(
            ErrorCode::Validation,
            "Revalidation must use the currently activated license.",
        ));
    }
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute(
        "UPDATE local_license_state SET license_id=?1,license_kind=?2,activated_at=coalesce(activated_at,?3),expires_at=?4,last_validated_at=?3,next_revalidation_at=?5,offline_grace_seconds=?6,max_observed_at=max(max_observed_at,?3),signed_payload=?7,signature=?8,updated_at=?3 WHERE singleton=1",
        params![claims.license_id,claims.state,requested_now,claims.expires_at,claims.revalidate_after,claims.offline_grace_days*86_400,payload,signature],
    )?;
    event(
        &tx,
        if revalidation {
            "LICENSE_REVALIDATED"
        } else {
            "LICENSE_ACTIVATED"
        },
        "A signed device-scoped license envelope was verified.",
        requested_now,
    )?;
    tx.commit()?;
    drop(connection);
    status_at(db, requested_now)
}

fn configured_public_key() -> Result<Vec<u8>, AppError> {
    let encoded = option_env!("M2_LICENSE_PUBLIC_KEY").unwrap_or("").trim();
    if encoded.is_empty() {
        return Err(AppError::new(
            ErrorCode::Validation,
            "Manual activation is not configured for this Early Access build. Start the trial or contact support.",
        ));
    }
    let key = URL_SAFE_NO_PAD.decode(encoded).map_err(|_| {
        AppError::new(
            ErrorCode::Internal,
            "This build has an invalid license verification key. Contact support.",
        )
    })?;
    if key.len() != 32 {
        return Err(AppError::new(
            ErrorCode::Internal,
            "This build has an invalid license verification key. Contact support.",
        ));
    }
    Ok(key)
}

pub fn activate(
    db: &AppDb,
    activation_code: &str,
    revalidation: bool,
) -> Result<LicenseStatus, AppError> {
    activate_at_with_key(
        db,
        activation_code,
        &configured_public_key()?,
        now()?,
        revalidation,
    )
}

pub fn enforce_operational_write(db: &AppDb) -> Result<(), AppError> {
    let current = status(db)?;
    if current.can_operate {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorCode::Forbidden,
            "This installation is read-only until its trial or license is activated or renewed. Existing data was not changed.",
        ))
    }
}

#[tauri::command]
pub fn get_license_status(state: State<'_, DatabaseState>) -> Result<LicenseStatus, AppError> {
    status(state.db()?)
}

#[tauri::command]
pub fn start_license_trial(state: State<'_, DatabaseState>) -> Result<LicenseStatus, AppError> {
    start_trial(state.db()?)
}

#[tauri::command]
pub fn activate_manual_license(
    state: State<'_, DatabaseState>,
    activation_code: String,
) -> Result<LicenseStatus, AppError> {
    activate(state.db()?, &activation_code, false)
}

#[tauri::command]
pub fn revalidate_manual_license(
    state: State<'_, DatabaseState>,
    activation_code: String,
) -> Result<LicenseStatus, AppError> {
    activate(state.db()?, &activation_code, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repositories::branches;
    use ring::signature::{Ed25519KeyPair, KeyPair};
    use tempfile::tempdir;

    fn db() -> AppDb {
        let db = AppDb::in_memory().unwrap();
        db.lock()
            .unwrap()
            .execute(
                "UPDATE local_license_state SET max_observed_at=0 WHERE singleton=1",
                [],
            )
            .unwrap();
        db
    }

    fn signed_code(
        device: &str,
        state: &str,
        issued: i64,
        revalidate: i64,
        expires: Option<i64>,
    ) -> (String, Vec<u8>) {
        let seed = [42_u8; 32];
        let pair = Ed25519KeyPair::from_seed_unchecked(&seed).unwrap();
        let claims = LicenseClaims {
            version: 1,
            license_id: "license-early-access-1".into(),
            device_id: device.into(),
            state: state.into(),
            issued_at: issued,
            expires_at: expires,
            revalidate_after: revalidate,
            offline_grace_days: 3,
        };
        let payload = serde_json::to_vec(&claims).unwrap();
        let signature = pair.sign(&payload);
        (
            format!(
                "{}.{}",
                URL_SAFE_NO_PAD.encode(payload),
                URL_SAFE_NO_PAD.encode(signature.as_ref())
            ),
            pair.public_key().as_ref().to_vec(),
        )
    }

    fn device(db: &AppDb) -> String {
        db.lock()
            .unwrap()
            .query_row("SELECT device_id FROM local_device_identity", [], |r| {
                r.get(0)
            })
            .unwrap()
    }

    #[test]
    fn first_activation_starts_exactly_one_seven_day_trial() {
        let db = db();
        let status = start_trial_at(&db, 1_000_000).unwrap();
        assert_eq!(status.state, "TRIAL");
        assert_eq!(status.expires_at, Some(1_000_000 + TRIAL_SECONDS));
        assert_eq!(status.days_remaining, Some(7));
        assert!(start_trial_at(&db, 1_000_001).is_err());
    }

    #[test]
    fn trial_remaining_time_and_expiry_are_deterministic() {
        let db = db();
        start_trial_at(&db, 1_000_000).unwrap();
        assert_eq!(
            status_at(&db, 1_000_000 + 6 * 86_400)
                .unwrap()
                .days_remaining,
            Some(1)
        );
        let expired = status_at(&db, 1_000_000 + TRIAL_SECONDS).unwrap();
        assert_eq!(expired.state, "EXPIRED");
        assert!(expired.read_only);
        assert!(!expired.can_operate);
    }

    #[test]
    fn signed_manual_license_uses_existing_device_identity() {
        let db = db();
        let device = device(&db);
        let (code, key) = signed_code(&device, "ACTIVE", 2_000_000, 2_086_400, None);
        let status = activate_at_with_key(&db, &code, &key, 2_000_000, false).unwrap();
        assert_eq!(status.state, "ACTIVE");
        assert_eq!(status.device_id, device);
        assert!(status.can_operate);
    }

    #[test]
    fn revoked_device_is_read_only() {
        let db = db();
        let (code, key) = signed_code(&device(&db), "REVOKED", 2_000_000, 2_086_400, None);
        let status = activate_at_with_key(&db, &code, &key, 2_000_000, false).unwrap();
        assert_eq!(status.state, "REVOKED");
        assert!(status.read_only);
    }

    #[test]
    fn overdue_revalidation_enters_then_leaves_offline_grace() {
        let db = db();
        let (code, key) = signed_code(&device(&db), "ACTIVE", 2_000_000, 2_086_400, None);
        activate_at_with_key(&db, &code, &key, 2_000_000, false).unwrap();
        assert_eq!(status_at(&db, 2_086_401).unwrap().state, "OFFLINE_GRACE");
        assert_eq!(
            status_at(&db, 2_086_400 + DEFAULT_OFFLINE_GRACE_SECONDS + 1)
                .unwrap()
                .state,
            "EXPIRED"
        );
    }

    #[test]
    fn fresh_signed_envelope_revalidates_the_same_license() {
        let db = db();
        let device = device(&db);
        let (initial, key) = signed_code(&device, "ACTIVE", 2_000_000, 2_086_400, None);
        activate_at_with_key(&db, &initial, &key, 2_000_000, false).unwrap();
        let (fresh, _) = signed_code(&device, "ACTIVE", 2_086_000, 2_172_800, None);
        let status = activate_at_with_key(&db, &fresh, &key, 2_086_000, true).unwrap();
        assert_eq!(status.state, "ACTIVE");
        assert_eq!(status.next_revalidation_at, Some(2_172_800));
    }

    #[test]
    fn corrupt_or_wrong_device_cache_is_rejected() {
        let db = db();
        let (code, key) = signed_code(
            "00000000-0000-4000-8000-000000000000",
            "ACTIVE",
            2_000_000,
            2_086_400,
            None,
        );
        assert!(activate_at_with_key(&db, &code, &key, 2_000_000, false).is_err());
        assert!(activate_at_with_key(&db, &(code + "x"), &key, 2_000_000, false).is_err());
    }

    #[test]
    fn license_expiry_never_deletes_pharmacy_data() {
        let db = db();
        branches::ensure_default_branch(&db).unwrap();
        start_trial_at(&db, 1_000_000).unwrap();
        let before: i64 = db
            .lock()
            .unwrap()
            .query_row("SELECT count(*) FROM branches", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            status_at(&db, 1_000_000 + TRIAL_SECONDS).unwrap().state,
            "EXPIRED"
        );
        let after: i64 = db
            .lock()
            .unwrap()
            .query_row("SELECT count(*) FROM branches", [], |r| r.get(0))
            .unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn device_and_trial_persist_when_database_reopens() {
        let root = tempdir().unwrap();
        let path = root.path().join("license.db");
        let original_device;
        {
            let db = AppDb::open(&path).unwrap();
            db.lock()
                .unwrap()
                .execute(
                    "UPDATE local_license_state SET max_observed_at=0 WHERE singleton=1",
                    [],
                )
                .unwrap();
            original_device = device(&db);
            start_trial_at(&db, 1_000_000).unwrap();
        }
        let reopened = AppDb::open(&path).unwrap();
        assert_eq!(device(&reopened), original_device);
        assert_eq!(status_at(&reopened, 1_000_001).unwrap().state, "TRIAL");
    }

    #[test]
    fn clock_rollback_cannot_extend_a_trial() {
        let db = db();
        start_trial_at(&db, 1_000_000).unwrap();
        status_at(&db, 1_000_000 + TRIAL_SECONDS).unwrap();
        assert_eq!(status_at(&db, 900_000).unwrap().state, "EXPIRED");
    }
}
