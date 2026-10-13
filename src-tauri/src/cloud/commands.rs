use super::{
    config::{CloudConfig, CloudConfigState},
    engine, finalize_status,
    models::{
        CloudLoginInput, CloudSessionStatus, ResolveSyncConflictInput, SyncConflict,
        SyncCycleResult,
    },
    status,
    transport::{CloudAuthState, SupabaseTransport},
    CloudFoundationStatus,
};
use crate::{
    authorization::AuthorizationState,
    commands::branches::DatabaseState,
    error::{AppError, ErrorCode},
};
use tauri::State;

fn configured(state: &CloudConfigState) -> Result<&CloudConfig, AppError> {
    state
        .0
        .as_ref()
        .map_err(Clone::clone)?
        .as_ref()
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::Validation,
                "Cloud is not configured on this device.",
            )
        })
}

#[tauri::command]
pub fn list_sync_conflicts(
    state: State<'_, DatabaseState>,
    session_token: String,
) -> Result<Vec<SyncConflict>, AppError> {
    let _ = session_token;
    engine::list_conflicts(state.db()?)
}

#[tauri::command]
pub fn resolve_sync_conflict(
    state: State<'_, DatabaseState>,
    authorization: State<'_, AuthorizationState>,
    session_token: String,
    input: ResolveSyncConflictInput,
) -> Result<(), AppError> {
    let _ = session_token;
    engine::resolve_conflict(state.db()?, &authorization.actor()?, &input)
}

#[tauri::command]
pub fn get_cloud_foundation_status(
    state: State<'_, DatabaseState>,
    config: State<'_, CloudConfigState>,
    auth: State<'_, CloudAuthState>,
    authorization: State<'_, AuthorizationState>,
    session_token: String,
) -> Result<CloudFoundationStatus, AppError> {
    let _ = session_token;
    let db = state.db()?;
    let actor = authorization.actor()?;
    let cloud_session = auth.status_for(&actor);
    let endpoint_configured = config.0.as_ref().is_ok_and(Option::is_some);
    let configured_transport_enabled = config
        .0
        .as_ref()
        .ok()
        .and_then(Option::as_ref)
        .is_some_and(|value| value.transport_enabled);
    let cloud_identity_linked = cloud_session
        .subject_id
        .as_deref()
        .is_some_and(|subject| engine::verify_local_cloud_identity(db, &actor, subject).is_ok());
    Ok(finalize_status(
        status(db)?,
        endpoint_configured,
        configured_transport_enabled,
        engine::phase8c_gate(db)?,
        cloud_identity_linked,
        cloud_session.authenticated,
        cloud_session.subject_id,
    ))
}

#[tauri::command]
pub fn get_cloud_session_status(
    auth: State<'_, CloudAuthState>,
    authorization: State<'_, AuthorizationState>,
    session_token: String,
) -> Result<CloudSessionStatus, AppError> {
    let _ = session_token;
    Ok(auth.status_for(&authorization.actor()?))
}

#[tauri::command]
pub fn cloud_sign_in(
    config: State<'_, CloudConfigState>,
    auth: State<'_, CloudAuthState>,
    authorization: State<'_, AuthorizationState>,
    session_token: String,
    input: CloudLoginInput,
) -> Result<CloudSessionStatus, AppError> {
    let _ = session_token;
    auth.sign_in(configured(&config)?, &input, &authorization.actor()?)
}

#[tauri::command]
pub fn cloud_sign_out(
    auth: State<'_, CloudAuthState>,
    authorization: State<'_, AuthorizationState>,
    session_token: String,
) -> Result<(), AppError> {
    let _ = session_token;
    auth.sign_out_for(&authorization.actor()?);
    Ok(())
}

#[tauri::command]
pub fn run_cloud_sync_cycle(
    state: State<'_, DatabaseState>,
    config: State<'_, CloudConfigState>,
    auth: State<'_, CloudAuthState>,
    authorization: State<'_, AuthorizationState>,
    session_token: String,
) -> Result<SyncCycleResult, AppError> {
    let _ = session_token;
    let config = configured(&config)?;
    if !config.transport_enabled {
        return Err(AppError::new(
            ErrorCode::Forbidden,
            "Synchronization transport remains disabled until Phase 8C verification.",
        ));
    }
    let actor = authorization.actor()?;
    let cloud_status = auth.status_for(&actor);
    let cloud_subject = cloud_status
        .subject_id
        .ok_or_else(|| AppError::new(ErrorCode::Unauthorized, "Cloud sign-in is required."))?;
    engine::verify_local_cloud_identity(state.db()?, &actor, &cloud_subject)?;
    let device_id = status(state.db()?)?.device_id;
    let transport = SupabaseTransport::new(config, &auth, device_id);
    engine::run_cycle(state.db()?, &transport)
}
