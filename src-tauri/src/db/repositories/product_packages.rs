use crate::{
    db::{catalog_validation::*, connection::AppDb, models::catalog::*},
    error::{AppError, ErrorCode},
};
use rusqlite::{params, Connection, OptionalExtension, Row};
const COLUMNS: &str = "id, product_id, package_label, pack_size, unit_name, units_per_package, strength_text, is_default, is_active, created_at, updated_at";
fn map(r: &Row<'_>) -> rusqlite::Result<ProductPackage> {
    Ok(ProductPackage {
        id: r.get(0)?,
        product_id: r.get(1)?,
        package_label: r.get(2)?,
        pack_size: r.get(3)?,
        unit_name: r.get(4)?,
        units_per_package: r.get(5)?,
        strength_text: r.get(6)?,
        is_default: r.get(7)?,
        is_active: r.get(8)?,
        created_at: r.get(9)?,
        updated_at: r.get(10)?,
    })
}
pub(crate) fn get(c: &Connection, value: &str) -> Result<ProductPackage, AppError> {
    c.query_row(
        &format!("SELECT {COLUMNS} FROM product_packages WHERE id=?1"),
        [value],
        map,
    )
    .optional()?
    .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Package not found."))
}
fn validate_create(input: &CreateProductPackage) -> Result<(), AppError> {
    id(&input.product_id)?;
    required(&input.package_label)?;
    for text in [&input.pack_size, &input.unit_name, &input.strength_text] {
        optional(text)?;
    }
    if input
        .units_per_package
        .is_some_and(|v| v <= 0 || v > MAX_SAFE_MINOR)
    {
        return Err(invalid("Units per package must be a positive integer."));
    }
    Ok(())
}
/// Inherits the caller's transaction; never opens one itself.
pub(crate) fn create_on(
    c: &Connection,
    input: &CreateProductPackage,
) -> Result<ProductPackage, AppError> {
    validate_create(input)?;
    let value = uuid::Uuid::new_v4().to_string();
    c.execute("INSERT INTO product_packages(id,product_id,package_label,pack_size,unit_name,units_per_package,strength_text,is_default) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",params![value,input.product_id,input.package_label,input.pack_size,input.unit_name,input.units_per_package,input.strength_text,input.is_default])?;
    get(c, &value)
}
pub fn create(db: &AppDb, input: &CreateProductPackage) -> Result<ProductPackage, AppError> {
    validate_create(input)?;
    let c = db.lock()?;
    create_on(&c, input)
}
/// Editable package fields, kept private so the composite workflows reuse one writer.
pub(crate) struct PackageEdit {
    pub package_label: String,
    pub pack_size: Option<String>,
    pub unit_name: Option<String>,
    pub units_per_package: Option<i64>,
    pub strength_text: Option<String>,
    pub is_active: bool,
    pub is_default: bool,
}
/// Inherits the caller's transaction; `is_default` is forced off when the package is inactive.
pub(crate) fn update_on(
    c: &Connection,
    value: &str,
    edit: &PackageEdit,
) -> Result<ProductPackage, AppError> {
    required(&edit.package_label)?;
    for text in [&edit.pack_size, &edit.unit_name, &edit.strength_text] {
        optional(text)?;
    }
    if edit
        .units_per_package
        .is_some_and(|v| v <= 0 || v > MAX_SAFE_MINOR)
    {
        return Err(invalid("Units per package must be a positive integer."));
    }
    let is_default = edit.is_active && edit.is_default;
    c.execute(
        "UPDATE product_packages SET package_label=?1,pack_size=?2,unit_name=?3,units_per_package=?4,strength_text=?5,is_active=?6,is_default=?7,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),julianday(updated_at)+0.001/86400)) WHERE id=?8",
        params![edit.package_label, edit.pack_size, edit.unit_name, edit.units_per_package, edit.strength_text, edit.is_active, is_default, value],
    )?;
    get(c, value)
}
pub(crate) fn set_active_on(
    c: &Connection,
    value: &str,
    active: bool,
) -> Result<ProductPackage, AppError> {
    c.execute("UPDATE product_packages SET is_active=?1,is_default=CASE WHEN ?1 THEN is_default ELSE 0 END,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),julianday(updated_at)+0.001/86400)) WHERE id=?2",params![active,value])?;
    get(c, value)
}
pub fn set_active(db: &AppDb, value: &str, active: bool) -> Result<ProductPackage, AppError> {
    id(value)?;
    let c = db.lock()?;
    set_active_on(&c, value, active)
}
/// The active package a catalog row represents: the default package when one exists,
/// otherwise the oldest active package.
pub(crate) fn default_on(
    c: &Connection,
    product_id: &str,
) -> Result<Option<ProductPackage>, AppError> {
    Ok(c.query_row(
        &format!("SELECT {COLUMNS} FROM product_packages WHERE product_id=?1 AND is_active=1 ORDER BY is_default DESC,created_at,id LIMIT 1"),
        [product_id],
        map,
    )
    .optional()?)
}
pub(crate) fn count_on(c: &Connection, product_id: &str) -> Result<i64, AppError> {
    Ok(c.query_row(
        "SELECT count(*) FROM product_packages WHERE product_id=?1",
        [product_id],
        |r| r.get(0),
    )?)
}
pub(crate) fn list_on(
    c: &Connection,
    product_id: &str,
    include_inactive: bool,
) -> Result<Vec<ProductPackage>, AppError> {
    let mut s=c.prepare(&format!("SELECT {COLUMNS} FROM product_packages WHERE product_id=?1 AND (?2 OR is_active=1) ORDER BY is_default DESC,created_at,id"))?;
    let rows = s.query_map(params![product_id, include_inactive], map)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
pub fn list(
    db: &AppDb,
    product_id: &str,
    include_inactive: bool,
) -> Result<Vec<ProductPackage>, AppError> {
    id(product_id)?;
    let c = db.lock()?;
    super::products::get(&c, product_id)?;
    list_on(&c, product_id, include_inactive)
}
