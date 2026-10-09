use super::{models::*, *};
use crate::{commands::branches::DatabaseState, error::AppError};
use tauri::State;

#[tauri::command]
pub fn get_auth_status(state: State<'_, DatabaseState>) -> Result<AuthStatus, AppError> {
    status(state.db()?)
}
#[tauri::command]
pub fn bootstrap_admin(
    state: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    input: BootstrapInput,
) -> Result<LoginResult, AppError> {
    bootstrap(state.db()?, &auth, &input)
}
#[tauri::command]
pub fn login(
    state: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    input: LoginInput,
) -> Result<LoginResult, AppError> {
    authenticate_password(state.db()?, &auth, &input)
}
#[tauri::command]
pub fn current_session(
    state: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    session_token: String,
) -> Result<User, AppError> {
    principal(state.db()?, &auth, &session_token).and_then(|p| user(state.db()?, &p.user_id))
}
#[tauri::command]
pub fn logout(auth: State<'_, AuthState>, session_token: String) -> Result<(), AppError> {
    auth.logout(&session_token);
    Ok(())
}
#[tauri::command]
pub fn list_users(
    state: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    session_token: String,
    query: UserQuery,
) -> Result<UserPage, AppError> {
    require_admin(state.db()?, &auth, &session_token)?;
    users(state.db()?, &query)
}
#[tauri::command]
pub fn get_user(
    state: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    session_token: String,
    id: String,
) -> Result<User, AppError> {
    require_admin(state.db()?, &auth, &session_token)?;
    user(state.db()?, &id)
}
#[tauri::command]
pub fn create_user(
    state: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    session_token: String,
    input: CreateUser,
) -> Result<User, AppError> {
    let p = require_admin(state.db()?, &auth, &session_token)?;
    create(state.db()?, &input, &p.user_id)
}
#[tauri::command]
pub fn update_user(
    state: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    session_token: String,
    input: UpdateUser,
) -> Result<User, AppError> {
    let p = require_admin(state.db()?, &auth, &session_token)?;
    update(state.db()?, &input, &p.user_id)
}
#[tauri::command]
pub fn set_user_active(
    state: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    session_token: String,
    input: SetUserActive,
) -> Result<User, AppError> {
    let p = require_admin(state.db()?, &auth, &session_token)?;
    let value = set_active(state.db()?, &input, &p.user_id)?;
    if !input.active {
        auth.revoke_user(&input.id)
    };
    Ok(value)
}
#[tauri::command]
pub fn reset_user_password(
    state: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    session_token: String,
    input: ResetPassword,
) -> Result<(), AppError> {
    let p = require_admin(state.db()?, &auth, &session_token)?;
    reset_password(state.db()?, &input, &p.user_id)?;
    auth.revoke_user(&input.id);
    Ok(())
}
#[tauri::command]
pub fn list_roles(
    state: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    session_token: String,
) -> Result<Vec<Role>, AppError> {
    principal(state.db()?, &auth, &session_token)?;
    roles(state.db()?)
}
#[tauri::command]
pub fn assign_user_role(
    state: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    session_token: String,
    input: UserRoleInput,
) -> Result<User, AppError> {
    let p = require_admin(state.db()?, &auth, &session_token)?;
    let value = assign_role(state.db()?, &input, &p.user_id)?;
    auth.revoke_user(&input.user_id);
    Ok(value)
}
#[tauri::command]
pub fn remove_user_role(
    state: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    session_token: String,
    input: UserRoleInput,
) -> Result<User, AppError> {
    let p = require_admin(state.db()?, &auth, &session_token)?;
    let value = remove_role(state.db()?, &input, &p.user_id)?;
    auth.revoke_user(&input.user_id);
    Ok(value)
}
#[tauri::command]
pub fn set_user_branches(
    state: State<'_, DatabaseState>,
    auth: State<'_, AuthState>,
    session_token: String,
    input: SetUserBranches,
) -> Result<User, AppError> {
    let p = require_admin(state.db()?, &auth, &session_token)?;
    let value = set_branches(state.db()?, &input, &p.user_id)?;
    auth.revoke_user(&input.user_id);
    Ok(value)
}
