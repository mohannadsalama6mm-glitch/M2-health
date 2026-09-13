use crate::{
    db::{
        connection::AppDb,
        models::branch::{Branch, CreateBranch},
    },
    error::{AppError, ErrorCode},
};
use rusqlite::{params, Connection, OptionalExtension, Row, TransactionBehavior};
use uuid::Uuid;

const COLUMNS: &str = "id, code, name, phone, address, is_active, created_at, updated_at";
fn map(row: &Row<'_>) -> rusqlite::Result<Branch> {
    Ok(Branch {
        id: row.get(0)?,
        code: row.get(1)?,
        name: row.get(2)?,
        phone: row.get(3)?,
        address: row.get(4)?,
        is_active: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}
pub(crate) fn get(connection: &Connection, id: &str) -> Result<Branch, AppError> {
    connection
        .query_row(
            &format!("SELECT {COLUMNS} FROM branches WHERE id = ?1"),
            [id],
            map,
        )
        .optional()?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "The requested branch does not exist."))
}
pub fn get_branch(db: &AppDb, id: &str) -> Result<Branch, AppError> {
    if Uuid::parse_str(id).is_err() {
        return Err(AppError::new(
            ErrorCode::Validation,
            "Branch ID must be a UUID.",
        ));
    }
    let connection = db.lock()?;
    get(&connection, id)
}
pub fn list_branches(db: &AppDb) -> Result<Vec<Branch>, AppError> {
    let connection = db.lock()?;
    let mut stmt = connection.prepare(&format!(
        "SELECT {COLUMNS} FROM branches ORDER BY is_active DESC, created_at, id"
    ))?;
    let rows = stmt.query_map([], map)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
fn insert(connection: &Connection, input: &CreateBranch) -> Result<Branch, AppError> {
    let code = input.code.trim().to_ascii_uppercase();
    let name = input.name.trim();
    if code.is_empty()
        || code.len() > 32
        || !code
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        || name.is_empty()
        || name.chars().count() > 160
        || input.phone.chars().count() > 64
        || input.address.chars().count() > 500
    {
        return Err(AppError::new(ErrorCode::Validation, "Enter a branch name and a unique 1–32 character code using letters, numbers, hyphens or underscores. Check contact field lengths."));
    }
    let id = Uuid::new_v4().to_string();
    connection.execute(
        "INSERT INTO branches(id, code, name, phone, address) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, code, name, input.phone.trim(), input.address.trim()],
    )?;
    get(connection, &id)
}
pub fn create_branch(db: &AppDb, input: &CreateBranch) -> Result<Branch, AppError> {
    let connection = db.lock()?;
    insert(&connection, input)
}
pub fn ensure_default_branch(db: &AppDb) -> Result<Branch, AppError> {
    let mut connection = db.lock()?;
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let existing = tx
        .query_row(
            &format!(
                "SELECT {COLUMNS} FROM branches ORDER BY is_active DESC, created_at, id LIMIT 1"
            ),
            [],
            map,
        )
        .optional()?;
    // If only inactive branches exist, return one without silently reactivating it.
    let branch = match existing {
        Some(branch) => branch,
        None => insert(
            &tx,
            &CreateBranch {
                code: "MAIN".into(),
                name: "Main Pharmacy".into(),
                phone: String::new(),
                address: String::new(),
            },
        )?,
    };
    tx.commit()?;
    Ok(branch)
}
