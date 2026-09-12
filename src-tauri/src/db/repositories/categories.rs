use crate::{
    db::{catalog_validation::*, connection::AppDb, models::catalog::*},
    error::{AppError, ErrorCode},
};
use rusqlite::{params, Connection, OptionalExtension, Row};
const COLUMNS: &str = "id, name, normalized_name, description, is_active, created_at, updated_at";
fn map(r: &Row<'_>) -> rusqlite::Result<Category> {
    Ok(Category {
        id: r.get(0)?,
        name: r.get(1)?,
        normalized_name: r.get(2)?,
        description: r.get(3)?,
        is_active: r.get(4)?,
        created_at: r.get(5)?,
        updated_at: r.get(6)?,
    })
}
pub(crate) fn get(c: &Connection, value: &str) -> Result<Category, AppError> {
    c.query_row(
        &format!("SELECT {COLUMNS} FROM categories WHERE id=?1"),
        [value],
        map,
    )
    .optional()?
    .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Category not found."))
}
pub fn create(db: &AppDb, input: &CreateCategory) -> Result<Category, AppError> {
    required(&input.name)?;
    optional(&input.description)?;

    let c = db.lock()?;
    let id = uuid::Uuid::new_v4().to_string();
    c.execute(
        "INSERT INTO categories(id,name,normalized_name,description) VALUES (?1,?2,?3,?4)",
        params![id, input.name, normalize(&input.name), input.description],
    )?;
    get(&c, &id)
}
pub fn list(db: &AppDb, include_inactive: bool) -> Result<Vec<Category>, AppError> {
    let c = db.lock()?;
    let mut s = c.prepare(&format!(
        "SELECT {COLUMNS} FROM categories WHERE (?1 OR is_active=1) ORDER BY normalized_name,id"
    ))?;
    let rows = s.query_map([include_inactive], map)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
pub fn set_active(db: &AppDb, value: &str, active: bool) -> Result<Category, AppError> {
    id(value)?;
    let c = db.lock()?;
    c.execute("UPDATE categories SET is_active=?1, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),julianday(updated_at)+0.001/86400)) WHERE id=?2",params![active,value])?;
    get(&c, value)
}
