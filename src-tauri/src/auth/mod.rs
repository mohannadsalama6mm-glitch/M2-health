//! Local/offline identity foundation. Operational authorization begins in Phase 7B.
pub mod commands;
pub mod models;

use crate::{
    db::{
        catalog_validation::{id, invalid, normalize},
        connection::AppDb,
    },
    error::{AppError, ErrorCode},
    inventory,
};
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use models::*;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

const SESSION_LIFETIME: i64 = 8 * 60 * 60;
const ADMIN: &str = "ADMIN";
#[derive(Clone)]
struct Session {
    user_id: String,
    credential_revision: i64,
    expires_at: i64,
}
#[derive(Default)]
struct Failure {
    count: u32,
    blocked_until: i64,
}
#[derive(Default)]
pub struct AuthState {
    sessions: Mutex<HashMap<String, Session>>,
    failures: Mutex<HashMap<String, Failure>>,
}
#[derive(Clone, Debug)]
pub struct Principal {
    pub user_id: String,
    pub roles: Vec<String>,
    pub branch_ids: Vec<String>,
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
fn token_digest(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}
fn argon() -> Result<Argon2<'static>, AppError> {
    let p = Params::new(19_456, 2, 1, Some(32)).map_err(|_| {
        AppError::new(
            ErrorCode::Internal,
            "Password security configuration is unavailable.",
        )
    })?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, p))
}
fn username(value: &str) -> Result<String, AppError> {
    let value = normalize(value);
    if !(3..=64).contains(&value.len())
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err(invalid("Username must be 3–64 characters using letters, numbers, dots, hyphens, or underscores."));
    }
    Ok(value)
}
fn display(value: &str) -> Result<String, AppError> {
    let v = value.trim();
    if v.is_empty() || v.chars().count() > 160 || v.contains('\0') {
        return Err(invalid("Display name must be 1–160 characters."));
    }
    Ok(v.into())
}
fn password(value: &str, user: &str) -> Result<(), AppError> {
    let count = value.chars().count();
    let lower = value.to_lowercase();
    if !(12..=128).contains(&count) || value.contains('\0') || lower.contains(user) {
        return Err(invalid(
            "Password must be 12–128 characters and must not contain the username.",
        ));
    }
    Ok(())
}
fn hash_password(value: &str) -> Result<String, AppError> {
    argon()?
        .hash_password(value.as_bytes())
        .map(|v| v.to_string())
        .map_err(|_| AppError::new(ErrorCode::Internal, "Password could not be secured."))
}
fn verify(value: &str, hash: &str) -> bool {
    PasswordHash::new(hash).ok().is_some_and(|h| {
        argon()
            .ok()
            .is_some_and(|a| a.verify_password(value.as_bytes(), &h).is_ok())
    })
}
fn dummy_hash() -> &'static str {
    static D: OnceLock<String> = OnceLock::new();
    D.get_or_init(|| hash_password("not-a-real-password-123").expect("Argon2 configuration"))
}
fn invalid_credentials() -> AppError {
    AppError::new(ErrorCode::Validation, "Invalid username or password.")
}
fn event(
    c: &Connection,
    user: Option<&str>,
    kind: &str,
    name: Option<&str>,
    detail: &str,
) -> Result<(), AppError> {
    c.execute("INSERT INTO auth_events(id,user_id,event_type,username_normalized,detail) VALUES(?1,?2,?3,?4,?5)",params![Uuid::new_v4().to_string(),user,kind,name,detail])?;
    Ok(())
}

fn security_event(
    c: &Connection,
    actor: &str,
    subject: Option<&str>,
    kind: &str,
    detail: &str,
) -> Result<(), AppError> {
    c.execute(
        "INSERT INTO security_events(id,actor_user_id,subject_user_id,event_type,detail) VALUES(?1,?2,?3,?4,?5)",
        params![Uuid::new_v4().to_string(), actor, subject, kind, detail],
    )?;
    Ok(())
}

impl AuthState {
    fn issue(&self, user_id: String, credential_revision: i64) -> Result<(String, i64), AppError> {
        let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let expires = now() + SESSION_LIFETIME;
        self.sessions
            .lock()
            .map_err(|_| {
                AppError::new(ErrorCode::Internal, "Authentication state is unavailable.")
            })?
            .insert(
                token_digest(&token),
                Session {
                    user_id,
                    credential_revision,
                    expires_at: expires,
                },
            );
        Ok((token, expires))
    }
    pub fn logout(&self, token: &str) {
        if let Ok(mut s) = self.sessions.lock() {
            s.remove(&token_digest(token));
        }
    }
    pub fn revoke_user(&self, user: &str) {
        if let Ok(mut s) = self.sessions.lock() {
            s.retain(|_, v| v.user_id != user);
        }
    }
}

pub fn status(db: &AppDb) -> Result<AuthStatus, AppError> {
    let c = db.lock()?;
    let initialized = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM auth_bootstrap WHERE singleton=1)",
        [],
        |r| r.get(0),
    )?;
    Ok(AuthStatus {
        initialized,
        session_lifetime_seconds: SESSION_LIFETIME,
    })
}
fn role_rows(c: &Connection, user_id: &str) -> Result<Vec<Role>, AppError> {
    let mut q=c.prepare("SELECT r.id,r.code,r.name,r.description,r.is_system FROM roles r JOIN user_roles ur ON ur.role_id=r.id WHERE ur.user_id=?1 ORDER BY r.code")?;
    let rows = q
        .query_map([user_id], |r| {
            Ok(Role {
                id: r.get(0)?,
                code: r.get(1)?,
                name: r.get(2)?,
                description: r.get(3)?,
                is_system: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
fn branch_rows(c: &Connection, user_id: &str) -> Result<Vec<BranchMembership>, AppError> {
    let mut q=c.prepare("SELECT b.id,b.code,b.name,m.is_primary FROM user_branch_memberships m JOIN branches b ON b.id=m.branch_id WHERE m.user_id=?1 ORDER BY m.is_primary DESC,b.code")?;
    let rows = q
        .query_map([user_id], |r| {
            Ok(BranchMembership {
                branch_id: r.get(0)?,
                branch_code: r.get(1)?,
                branch_name: r.get(2)?,
                is_primary: r.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
fn user_on(c: &Connection, value: &str) -> Result<User, AppError> {
    id(value)?;
    let mut u=c.query_row("SELECT id,username,display_name,is_active,revision,created_at,updated_at FROM users WHERE id=?1",[value],|r|Ok(User{id:r.get(0)?,username:r.get(1)?,display_name:r.get(2)?,is_active:r.get(3)?,revision:r.get(4)?,created_at:r.get(5)?,updated_at:r.get(6)?,roles:vec![],branches:vec![]})).optional()?.ok_or_else(||AppError::new(ErrorCode::NotFound,"User not found."))?;
    u.roles = role_rows(c, &u.id)?;
    u.branches = branch_rows(c, &u.id)?;
    Ok(u)
}
pub fn user(db: &AppDb, value: &str) -> Result<User, AppError> {
    let c = db.lock()?;
    user_on(&c, value)
}
pub fn roles(db: &AppDb) -> Result<Vec<Role>, AppError> {
    let c = db.lock()?;
    let mut q = c.prepare("SELECT id,code,name,description,is_system FROM roles ORDER BY code")?;
    let rows = q
        .query_map([], |r| {
            Ok(Role {
                id: r.get(0)?,
                code: r.get(1)?,
                name: r.get(2)?,
                description: r.get(3)?,
                is_system: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
pub fn principal(db: &AppDb, state: &AuthState, token: &str) -> Result<Principal, AppError> {
    if token.len() != 64 {
        return Err(AppError::new(
            ErrorCode::Validation,
            "Session is invalid or expired.",
        ));
    }
    let digest = token_digest(token);
    let session = state
        .sessions
        .lock()
        .map_err(|_| AppError::new(ErrorCode::Internal, "Authentication state is unavailable."))?
        .get(&digest)
        .cloned();
    let Some(session) = session else {
        return Err(AppError::new(
            ErrorCode::Validation,
            "Session is invalid or expired.",
        ));
    };
    if session.expires_at <= now() {
        state.logout(token);
        return Err(AppError::new(
            ErrorCode::Validation,
            "Session is invalid or expired.",
        ));
    }
    let c = db.lock()?;
    let current = c
        .query_row(
            "SELECT is_active,credential_revision FROM users WHERE id=?1",
            [&session.user_id],
            |r| Ok((r.get::<_, bool>(0)?, r.get::<_, i64>(1)?)),
        )
        .optional()?;
    let Some((active, rev)) = current else {
        state.logout(token);
        return Err(AppError::new(
            ErrorCode::Validation,
            "Session is invalid or expired.",
        ));
    };
    if !active || rev != session.credential_revision {
        state.logout(token);
        return Err(AppError::new(
            ErrorCode::Validation,
            "Session is invalid or expired.",
        ));
    }
    let roles = role_rows(&c, &session.user_id)?
        .into_iter()
        .map(|r| r.code)
        .collect();
    let branch_ids = branch_rows(&c, &session.user_id)?
        .into_iter()
        .map(|branch| branch.branch_id)
        .collect();
    Ok(Principal {
        user_id: session.user_id,
        roles,
        branch_ids,
    })
}
pub fn require_admin(db: &AppDb, state: &AuthState, token: &str) -> Result<Principal, AppError> {
    let p = principal(db, state, token)?;
    if !p.roles.iter().any(|r| r == ADMIN) {
        return Err(AppError::new(
            ErrorCode::Validation,
            "Administrator access is required.",
        ));
    }
    Ok(p)
}

pub fn bootstrap(
    db: &AppDb,
    state: &AuthState,
    input: &BootstrapInput,
) -> Result<LoginResult, AppError> {
    let name = username(&input.username)?;
    let display = display(&input.display_name)?;
    password(&input.password, &name)?;
    let hash = hash_password(&input.password)?;
    let id = Uuid::new_v4().to_string();
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM auth_bootstrap) OR EXISTS(SELECT 1 FROM users)",
        [],
        |r| r.get(0),
    )?;
    if exists {
        return Err(AppError::new(
            ErrorCode::Conflict,
            "Initial administrator setup is already complete.",
        ));
    }
    tx.execute(
        "INSERT INTO users(id,username,username_normalized,display_name) VALUES(?1,?2,?3,?4)",
        params![id, name, name, display],
    )?;
    tx.execute(
        "INSERT INTO user_credentials(user_id,password_hash) VALUES(?1,?2)",
        params![id, hash],
    )?;
    tx.execute("INSERT INTO user_roles(user_id,role_id,assigned_by) SELECT ?1,id,?1 FROM roles WHERE code='ADMIN'",[&id])?;
    tx.execute(
        "INSERT INTO auth_bootstrap(singleton,initialized_by) VALUES(1,?1)",
        [&id],
    )?;
    event(
        &tx,
        Some(&id),
        "BOOTSTRAP",
        Some(&name),
        "Initial administrator created.",
    )?;
    tx.commit()?;
    drop(c);
    let (token, expires_at) = state.issue(id.clone(), 1)?;
    Ok(LoginResult {
        token,
        expires_at,
        user: user(db, &id)?,
    })
}
pub fn authenticate_password(
    db: &AppDb,
    state: &AuthState,
    input: &LoginInput,
) -> Result<LoginResult, AppError> {
    let name = username(&input.username).map_err(|_| invalid_credentials())?;
    let blocked = state
        .failures
        .lock()
        .map_err(|_| AppError::new(ErrorCode::Internal, "Authentication state is unavailable."))?
        .get(&name)
        .is_some_and(|f| f.blocked_until > now());
    if blocked {
        let _ = verify(&input.password, dummy_hash());
        return Err(invalid_credentials());
    }
    let c = db.lock()?;
    let row=c.query_row("SELECT u.id,u.is_active,u.credential_revision,x.password_hash FROM users u JOIN user_credentials x ON x.user_id=u.id WHERE u.username_normalized=?1",[&name],|r|Ok((r.get::<_,String>(0)?,r.get::<_,bool>(1)?,r.get::<_,i64>(2)?,r.get::<_,String>(3)?))).optional()?;
    let valid = row
        .as_ref()
        .is_some_and(|(_, active, _, hash)| *active && verify(&input.password, hash));
    if !valid {
        if row.is_none() {
            let _ = verify(&input.password, dummy_hash());
        }
        drop(c);
        let mut failures = state.failures.lock().map_err(|_| {
            AppError::new(ErrorCode::Internal, "Authentication state is unavailable.")
        })?;
        let f = failures.entry(name.clone()).or_default();
        f.count += 1;
        if f.count >= 5 {
            f.blocked_until = now() + 30_i64.saturating_mul(1_i64 << (f.count - 5).min(4));
        }
        drop(failures);
        let c = db.lock()?;
        event(
            &c,
            row.as_ref().map(|v| v.0.as_str()),
            "LOGIN_FAILURE",
            Some(&name),
            "Authentication failed.",
        )?;
        return Err(invalid_credentials());
    }
    let (user_id, _, revision, _) = row.unwrap();
    state
        .failures
        .lock()
        .map_err(|_| AppError::new(ErrorCode::Internal, "Authentication state is unavailable."))?
        .remove(&name);
    event(
        &c,
        Some(&user_id),
        "LOGIN_SUCCESS",
        Some(&name),
        "Local session issued.",
    )?;
    drop(c);
    let (token, expires_at) = state.issue(user_id.clone(), revision)?;
    Ok(LoginResult {
        token,
        expires_at,
        user: user(db, &user_id)?,
    })
}

fn validate_roles_branches(
    c: &Connection,
    role_codes: &[String],
    branch_ids: &[String],
    primary: &Option<String>,
) -> Result<(), AppError> {
    let mut seen = std::collections::HashSet::new();
    for code in role_codes {
        if !seen.insert(code)
            || !c.query_row(
                "SELECT EXISTS(SELECT 1 FROM roles WHERE code=?1)",
                [code],
                |r| r.get::<_, bool>(0),
            )?
        {
            return Err(invalid("Role assignment is invalid or duplicated."));
        }
    }
    let mut branches = std::collections::HashSet::new();
    for branch in branch_ids {
        id(branch)?;
        if !branches.insert(branch)
            || !c.query_row(
                "SELECT EXISTS(SELECT 1 FROM branches WHERE id=?1 AND is_active=1)",
                [branch],
                |r| r.get::<_, bool>(0),
            )?
        {
            return Err(invalid("Branch assignment is invalid or duplicated."));
        }
    }
    if primary.as_ref().is_some_and(|p| !branches.contains(p)) {
        return Err(invalid(
            "Primary branch must be included in branch assignments.",
        ));
    }
    Ok(())
}
pub fn create(db: &AppDb, input: &CreateUser, actor: &str) -> Result<User, AppError> {
    let name = username(&input.username)?;
    let display = display(&input.display_name)?;
    password(&input.password, &name)?;
    let hash = hash_password(&input.password)?;
    let id = Uuid::new_v4().to_string();
    let mut c = db.lock()?;
    validate_roles_branches(
        &c,
        &input.role_codes,
        &input.branch_ids,
        &input.primary_branch_id,
    )?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute(
        "INSERT INTO users(id,username,username_normalized,display_name) VALUES(?1,?2,?3,?4)",
        params![id, name, name, display],
    )?;
    tx.execute(
        "INSERT INTO user_credentials(user_id,password_hash) VALUES(?1,?2)",
        params![id, hash],
    )?;
    for code in &input.role_codes {
        tx.execute("INSERT INTO user_roles(user_id,role_id,assigned_by) SELECT ?1,id,?2 FROM roles WHERE code=?3",params![id,actor,code])?;
    }
    for branch in &input.branch_ids {
        tx.execute("INSERT INTO user_branch_memberships(user_id,branch_id,is_primary,assigned_by) VALUES(?1,?2,?3,?4)",params![id,branch,input.primary_branch_id.as_ref()==Some(branch),actor])?;
    }
    security_event(&tx, actor, Some(&id), "USER_CREATED", "Local user created.")?;
    tx.commit()?;
    drop(c);
    user(db, &id)
}
pub fn update(db: &AppDb, input: &UpdateUser, actor: &str) -> Result<User, AppError> {
    id(&input.id)?;
    let name = username(&input.username)?;
    let display = display(&input.display_name)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let changed=tx.execute("UPDATE users SET username=?2,username_normalized=?2,display_name=?3,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND revision=?4",params![input.id,name,display,input.expected_revision])?;
    if changed == 0 {
        return Err(AppError::new(
            ErrorCode::Conflict,
            "User changed elsewhere. Reload and retry.",
        ));
    }
    security_event(
        &tx,
        actor,
        Some(&input.id),
        "USER_UPDATED",
        "User profile updated.",
    )?;
    tx.commit()?;
    drop(c);
    user(db, &input.id)
}
pub fn set_active(db: &AppDb, input: &SetUserActive, actor: &str) -> Result<User, AppError> {
    id(&input.id)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let changed=tx.execute("UPDATE users SET is_active=?2,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND revision=?3",params![input.id,input.active,input.expected_revision]).map_err(|e|if e.to_string().contains("final_active_admin_required"){invalid("The final active administrator cannot be deactivated.")}else{e.into()})?;
    if changed == 0 {
        return Err(AppError::new(
            ErrorCode::Conflict,
            "User changed elsewhere. Reload and retry.",
        ));
    }
    event(
        &tx,
        Some(&input.id),
        if input.active {
            "USER_ACTIVATED"
        } else {
            "USER_DEACTIVATED"
        },
        None,
        &format!("Changed by {actor}."),
    )?;
    security_event(
        &tx,
        actor,
        Some(&input.id),
        if input.active {
            "USER_ACTIVATED"
        } else {
            "USER_DEACTIVATED"
        },
        "User active state changed.",
    )?;
    tx.commit()?;
    drop(c);
    user(db, &input.id)
}
pub fn reset_password(db: &AppDb, input: &ResetPassword, actor: &str) -> Result<(), AppError> {
    id(&input.id)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let name: String = tx
        .query_row(
            "SELECT username_normalized FROM users WHERE id=?1",
            [&input.id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "User not found."))?;
    password(&input.password, &name)?;
    let hash = hash_password(&input.password)?;
    tx.execute("UPDATE user_credentials SET password_hash=?2,password_changed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE user_id=?1",params![input.id,hash])?;
    tx.execute("UPDATE users SET credential_revision=credential_revision+1,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[&input.id])?;
    event(
        &tx,
        Some(&input.id),
        "PASSWORD_RESET",
        None,
        &format!("Reset by {actor}."),
    )?;
    security_event(
        &tx,
        actor,
        Some(&input.id),
        "PASSWORD_RESET",
        "Password reset by administrator.",
    )?;
    tx.commit()?;
    Ok(())
}
pub fn assign_role(db: &AppDb, input: &UserRoleInput, actor: &str) -> Result<User, AppError> {
    id(&input.user_id)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let role: String = tx
        .query_row(
            "SELECT id FROM roles WHERE code=?1",
            [&input.role_code],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| invalid("Role does not exist."))?;
    tx.execute(
        "INSERT INTO user_roles(user_id,role_id,assigned_by) VALUES(?1,?2,?3)",
        params![input.user_id, role, actor],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            AppError::new(ErrorCode::Conflict, "User already has this role.")
        } else {
            e.into()
        }
    })?;
    event(
        &tx,
        Some(&input.user_id),
        "ROLE_ASSIGNED",
        None,
        &input.role_code,
    )?;
    security_event(
        &tx,
        actor,
        Some(&input.user_id),
        "ROLE_ASSIGNED",
        &input.role_code,
    )?;
    tx.commit()?;
    drop(c);
    user(db, &input.user_id)
}
pub fn remove_role(db: &AppDb, input: &UserRoleInput, actor: &str) -> Result<User, AppError> {
    id(&input.user_id)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let role: String = tx
        .query_row(
            "SELECT id FROM roles WHERE code=?1",
            [&input.role_code],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| invalid("Role does not exist."))?;
    let changed = tx
        .execute(
            "DELETE FROM user_roles WHERE user_id=?1 AND role_id=?2",
            params![input.user_id, role],
        )
        .map_err(|e| {
            if e.to_string().contains("final_active_admin_required") {
                invalid("The final active administrator must retain the ADMIN role.")
            } else {
                e.into()
            }
        })?;
    if changed == 0 {
        return Err(AppError::new(
            ErrorCode::NotFound,
            "User does not have this role.",
        ));
    }
    event(
        &tx,
        Some(&input.user_id),
        "ROLE_REMOVED",
        None,
        &format!("{} by {actor}", input.role_code),
    )?;
    security_event(
        &tx,
        actor,
        Some(&input.user_id),
        "ROLE_REMOVED",
        &input.role_code,
    )?;
    tx.commit()?;
    drop(c);
    user(db, &input.user_id)
}
pub fn set_branches(db: &AppDb, input: &SetUserBranches, actor: &str) -> Result<User, AppError> {
    id(&input.user_id)?;
    let mut c = db.lock()?;
    validate_roles_branches(&c, &[], &input.branch_ids, &input.primary_branch_id)?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if !tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id=?1)",
        [&input.user_id],
        |r| r.get::<_, bool>(0),
    )? {
        return Err(AppError::new(ErrorCode::NotFound, "User not found."));
    }
    tx.execute(
        "DELETE FROM user_branch_memberships WHERE user_id=?1",
        [&input.user_id],
    )?;
    for branch in &input.branch_ids {
        tx.execute("INSERT INTO user_branch_memberships(user_id,branch_id,is_primary,assigned_by) VALUES(?1,?2,?3,?4)",params![input.user_id,branch,input.primary_branch_id.as_ref()==Some(branch),actor])?;
    }
    event(
        &tx,
        Some(&input.user_id),
        "BRANCH_MEMBERSHIP_CHANGED",
        None,
        "Branch memberships replaced.",
    )?;
    security_event(
        &tx,
        actor,
        Some(&input.user_id),
        "BRANCH_MEMBERSHIP_CHANGED",
        "Branch memberships replaced.",
    )?;
    tx.commit()?;
    drop(c);
    user(db, &input.user_id)
}
pub fn users(db: &AppDb, q: &UserQuery) -> Result<UserPage, AppError> {
    let c = db.lock()?;
    let search = q.search.as_deref().unwrap_or("").trim();
    if search.chars().count() > 200 || search.contains('\0') {
        return Err(invalid("User search must be at most 200 characters."));
    }
    if let Some(code) = &q.role_code {
        if !c.query_row(
            "SELECT EXISTS(SELECT 1 FROM roles WHERE code=?1)",
            [code],
            |r| r.get::<_, bool>(0),
        )? {
            return Err(invalid("Role filter is invalid."));
        }
    }
    let (limit, offset) = inventory::page(q.limit, q.offset)?;
    let filter="(?1='' OR instr(lower(u.username||' '||u.display_name),lower(?1))>0) AND (?2 IS NULL OR u.is_active=?2) AND (?3 IS NULL OR EXISTS(SELECT 1 FROM user_roles ur JOIN roles r ON r.id=ur.role_id WHERE ur.user_id=u.id AND r.code=?3))";
    let total = c.query_row(
        &format!("SELECT count(*) FROM users u WHERE {filter}"),
        params![search, q.is_active, q.role_code],
        |r| r.get(0),
    )?;
    let mut stmt=c.prepare(&format!("SELECT u.id FROM users u WHERE {filter} ORDER BY u.is_active DESC,u.username_normalized,u.id LIMIT ?4 OFFSET ?5"))?;
    let ids = stmt
        .query_map(
            params![search, q.is_active, q.role_code, limit, offset],
            |r| r.get::<_, String>(0),
        )?
        .collect::<Result<Vec<_>, _>>()?;
    let items = ids
        .iter()
        .map(|v| user_on(&c, v))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(UserPage {
        items,
        total,
        limit,
        offset,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expired_session_is_rejected_and_removed() {
        let db = AppDb::in_memory().unwrap();
        let state = AuthState::default();
        let login = bootstrap(
            &db,
            &state,
            &BootstrapInput {
                username: "expiry.admin".into(),
                display_name: "Expiry Administrator".into(),
                password: "Strong!ExpiryPass2026".into(),
            },
        )
        .unwrap();
        let digest = token_digest(&login.token);
        state
            .sessions
            .lock()
            .unwrap()
            .get_mut(&digest)
            .unwrap()
            .expires_at = now() - 1;
        assert!(principal(&db, &state, &login.token).is_err());
        assert!(!state.sessions.lock().unwrap().contains_key(&digest));
    }
}
