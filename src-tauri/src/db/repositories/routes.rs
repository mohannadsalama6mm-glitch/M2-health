use crate::{
    db::{catalog_validation::*, connection::AppDb, models::catalog::*},
    error::{AppError, ErrorCode},
};
use rusqlite::{params, Connection, OptionalExtension, Row};
const COLUMNS: &str = "id, name, normalized_name, is_active, created_at, updated_at";
fn map(r: &Row<'_>) -> rusqlite::Result<Route> {
    Ok(Route {
        id: r.get(0)?,
        name: r.get(1)?,
        normalized_name: r.get(2)?,
        is_active: r.get(3)?,
        created_at: r.get(4)?,
        updated_at: r.get(5)?,
    })
}
pub(crate) fn get(c: &Connection, value: &str) -> Result<Route, AppError> {
    c.query_row(
        &format!("SELECT {COLUMNS} FROM routes WHERE id=?1"),
        [value],
        map,
    )
    .optional()?
    .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Route not found."))
}
pub fn create(db: &AppDb, input: &CreateRoute) -> Result<Route, AppError> {
    required(&input.name)?;

    let c = db.lock()?;
    let id = uuid::Uuid::new_v4().to_string();
    c.execute(
        "INSERT INTO routes(id,name,normalized_name) VALUES (?1,?2,?3)",
        params![id, input.name, normalize(&input.name)],
    )?;
    get(&c, &id)
}
pub fn list(db: &AppDb, include_inactive: bool) -> Result<Vec<Route>, AppError> {
    let c = db.lock()?;
    let mut s = c.prepare(&format!(
        "SELECT {COLUMNS} FROM routes WHERE (?1 OR is_active=1) ORDER BY normalized_name,id"
    ))?;
    let rows = s.query_map([include_inactive], map)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
pub fn set_active(db: &AppDb, value: &str, active: bool) -> Result<Route, AppError> {
    id(value)?;
    let c = db.lock()?;
    c.execute("UPDATE routes SET is_active=?1, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),julianday(updated_at)+0.001/86400)) WHERE id=?2",params![active,value])?;
    get(&c, value)
}
