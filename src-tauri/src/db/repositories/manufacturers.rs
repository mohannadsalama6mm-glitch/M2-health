use crate::{
    db::{catalog_validation::*, connection::AppDb, models::catalog::*},
    error::{AppError, ErrorCode},
};
use rusqlite::{params, Connection, OptionalExtension, Row};
const COLUMNS: &str =
    "id, name, normalized_name, country, phone, email, website, is_active, created_at, updated_at";
fn map(r: &Row<'_>) -> rusqlite::Result<Manufacturer> {
    Ok(Manufacturer {
        id: r.get(0)?,
        name: r.get(1)?,
        normalized_name: r.get(2)?,
        country: r.get(3)?,
        phone: r.get(4)?,
        email: r.get(5)?,
        website: r.get(6)?,
        is_active: r.get(7)?,
        created_at: r.get(8)?,
        updated_at: r.get(9)?,
    })
}
pub(crate) fn get(c: &Connection, value: &str) -> Result<Manufacturer, AppError> {
    c.query_row(
        &format!("SELECT {COLUMNS} FROM manufacturers WHERE id=?1"),
        [value],
        map,
    )
    .optional()?
    .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Manufacturer not found."))
}
pub fn create(db: &AppDb, input: &CreateManufacturer) -> Result<Manufacturer, AppError> {
    required(&input.name)?;
    optional(&input.country)?;
    optional(&input.phone)?;
    optional(&input.email)?;
    optional(&input.website)?;

    let c = db.lock()?;
    let id = uuid::Uuid::new_v4().to_string();
    c.execute("INSERT INTO manufacturers(id,name,normalized_name,country,phone,email,website) VALUES (?1,?2,?3,?4,?5,?6,?7)",params![id,input.name,normalize(&input.name),input.country,input.phone,input.email,input.website])?;
    get(&c, &id)
}
pub fn list(db: &AppDb, include_inactive: bool) -> Result<Vec<Manufacturer>, AppError> {
    let c = db.lock()?;
    let mut s = c.prepare(&format!(
        "SELECT {COLUMNS} FROM manufacturers WHERE (?1 OR is_active=1) ORDER BY normalized_name,id"
    ))?;
    let rows = s.query_map([include_inactive], map)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
pub fn set_active(db: &AppDb, value: &str, active: bool) -> Result<Manufacturer, AppError> {
    id(value)?;
    let c = db.lock()?;
    c.execute("UPDATE manufacturers SET is_active=?1, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),julianday(updated_at)+0.001/86400)) WHERE id=?2",params![active,value])?;
    get(&c, value)
}
