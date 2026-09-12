use crate::{
    db::{catalog_validation::*, connection::AppDb, models::catalog::*},
    error::AppError,
};
use rusqlite::{params, Connection, Row};
const COLUMNS: &str = "id, product_package_id, barcode, is_primary, created_at, updated_at";
fn map(r: &Row<'_>) -> rusqlite::Result<Barcode> {
    Ok(Barcode {
        id: r.get(0)?,
        product_package_id: r.get(1)?,
        barcode: r.get(2)?,
        is_primary: r.get(3)?,
        created_at: r.get(4)?,
        updated_at: r.get(5)?,
    })
}
pub fn add(db: &AppDb, input: &AddBarcode) -> Result<Barcode, AppError> {
    id(&input.product_package_id)?;
    let barcode = input.barcode.trim();
    if barcode.is_empty() || barcode.len() > 128 || !barcode.bytes().all(|b| b.is_ascii_graphic()) {
        return Err(invalid(
            "Barcode must contain 1–128 printable ASCII characters without spaces.",
        ));
    }
    let c = db.lock()?;
    let value = uuid::Uuid::new_v4().to_string();
    c.execute(
        "INSERT INTO barcodes(id,product_package_id,barcode,is_primary) VALUES (?1,?2,?3,?4)",
        params![value, input.product_package_id, barcode, input.is_primary],
    )?;
    Ok(c.query_row(
        &format!("SELECT {COLUMNS} FROM barcodes WHERE id=?1"),
        [value],
        map,
    )?)
}
pub(crate) fn list_on(c: &Connection, package_id: &str) -> Result<Vec<Barcode>, AppError> {
    let mut s=c.prepare(&format!("SELECT {COLUMNS} FROM barcodes WHERE product_package_id=?1 ORDER BY is_primary DESC,created_at,id"))?;
    let rows = s.query_map([package_id], map)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
pub fn list(db: &AppDb, package_id: &str) -> Result<Vec<Barcode>, AppError> {
    id(package_id)?;
    let c = db.lock()?;
    super::product_packages::get(&c, package_id)?;
    list_on(&c, package_id)
}
