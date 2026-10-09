use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub id: String,
    pub code: String,
    pub name: String,
    pub description: String,
    pub is_system: bool,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchMembership {
    pub branch_id: String,
    pub branch_code: String,
    pub branch_name: String,
    pub is_primary: bool,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub is_active: bool,
    pub revision: i64,
    pub created_at: String,
    pub updated_at: String,
    pub roles: Vec<Role>,
    pub branches: Vec<BranchMembership>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPage {
    pub items: Vec<User>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub initialized: bool,
    pub session_lifetime_seconds: i64,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResult {
    pub token: String,
    pub expires_at: i64,
    pub user: User,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BootstrapInput {
    pub username: String,
    pub display_name: String,
    pub password: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateUser {
    pub username: String,
    pub display_name: String,
    pub password: String,
    #[serde(default)]
    pub role_codes: Vec<String>,
    #[serde(default)]
    pub branch_ids: Vec<String>,
    pub primary_branch_id: Option<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateUser {
    pub id: String,
    pub expected_revision: i64,
    pub username: String,
    pub display_name: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetUserActive {
    pub id: String,
    pub expected_revision: i64,
    pub active: bool,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResetPassword {
    pub id: String,
    pub password: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UserRoleInput {
    pub user_id: String,
    pub role_code: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetUserBranches {
    pub user_id: String,
    #[serde(default)]
    pub branch_ids: Vec<String>,
    pub primary_branch_id: Option<String>,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UserQuery {
    pub search: Option<String>,
    pub is_active: Option<bool>,
    pub role_code: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
