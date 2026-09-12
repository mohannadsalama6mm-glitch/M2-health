use crate::{
    db::{catalog_validation::*, connection::AppDb, models::catalog::*},
    error::AppError,
};
use rusqlite::{params, Connection, OptionalExtension, Row};
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
pub(crate) fn validate(value: &str) -> Result<String, AppError> {
    let barcode = value.trim();
    if barcode.is_empty() || barcode.len() > 128 || !barcode.bytes().all(|b| b.is_ascii_graphic()) {
        return Err(invalid(
            "Barcode must contain 1–128 printable ASCII characters without spaces.",
        ));
    }
    Ok(barcode.to_string())
}
pub fn add(db: &AppDb, input: &AddBarcode) -> Result<Barcode, AppError> {
    id(&input.product_package_id)?;
    let barcode = validate(&input.barcode)?;
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
/// Replaces the barcode set of one package inside a caller-owned transaction.
/// At most one primary barcode per package is enforced up front (and again by the
/// partial unique index). `values` must already be free of global conflicts.
pub(crate) fn insert_all_on(
    c: &Connection,
    package_id: &str,
    inputs: &[ProductBarcodeInput],
) -> Result<Vec<Barcode>, AppError> {
    if inputs.iter().filter(|b| b.is_primary).count() > 1 {
        return Err(invalid("A package can have at most one primary barcode."));
    }
    let mut out = Vec::with_capacity(inputs.len());
    for input in inputs {
        let barcode = validate(&input.barcode)?;
        let value = uuid::Uuid::new_v4().to_string();
        c.execute(
            "INSERT INTO barcodes(id,product_package_id,barcode,is_primary) VALUES (?1,?2,?3,?4)",
            params![value, package_id, barcode, input.is_primary],
        )?;
        out.push(c.query_row(
            &format!("SELECT {COLUMNS} FROM barcodes WHERE id=?1"),
            [value],
            map,
        )?);
    }
    Ok(out)
}
/// Removes every barcode of every package that belongs to a product.
/// Used by the edit workflow so its package reconciliation can move barcodes freely.
pub(crate) fn wipe_product_on(c: &Connection, product_id: &str) -> Result<(), AppError> {
    c.execute(
        "DELETE FROM barcodes WHERE product_package_id IN (SELECT id FROM product_packages WHERE product_id=?1)",
        [product_id],
    )?;
    Ok(())
}
/// Barcodes that are globally assigned to a *different* product's package.
/// `self_product_id` excludes ownership rows owned by the workflow's own product.
pub(crate) fn conflicts_on(
    c: &Connection,
    self_product_id: &str,
    values: &[String],
) -> Result<Vec<String>, AppError> {
    if values.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = values.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql_params: Vec<rusqlite::types::Value> =
        std::iter::once(rusqlite::types::Value::Text(self_product_id.into()))
            .chain(
                values
                    .iter()
                    .map(|v| rusqlite::types::Value::Text(v.clone())),
            )
            .collect();
    let mut s = c.prepare(&format!(
        "SELECT b.barcode FROM barcodes b JOIN product_packages pp ON pp.id=b.product_package_id WHERE pp.product_id != ?1 AND b.barcode IN ({placeholders})"
    ))?;
    let rows = s.query_map(rusqlite::params_from_iter(sql_params.into_iter()), |r| {
        r.get::<_, String>(0)
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
/// The primary (or first) barcode of a package, as shown on catalog rows.
pub(crate) fn first_on(c: &Connection, package_id: &str) -> Result<Option<Barcode>, AppError> {
    Ok(c.query_row(
        &format!("SELECT {COLUMNS} FROM barcodes WHERE product_package_id=?1 ORDER BY is_primary DESC,created_at,id LIMIT 1"),
        [package_id],
        map,
    )
    .optional()?)
}
pub(crate) fn count_product_on(c: &Connection, product_id: &str) -> Result<i64, AppError> {
    Ok(c.query_row(
        "SELECT count(*) FROM barcodes b JOIN product_packages pp ON pp.id=b.product_package_id WHERE pp.product_id=?1",
        [product_id],
        |r| r.get(0),
    )?)
}
pub fn list(db: &AppDb, package_id: &str) -> Result<Vec<Barcode>, AppError> {
    id(package_id)?;
    let c = db.lock()?;
    super::product_packages::get(&c, package_id)?;
    list_on(&c, package_id)
}
