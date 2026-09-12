use crate::{
    db::{catalog_validation::*, connection::AppDb, models::catalog::*},
    error::AppError,
};
use rusqlite::{params, Connection, OptionalExtension, Row, TransactionBehavior};
const COLUMNS: &str = "id, product_package_id, selling_price_minor, cost_price_minor, effective_from, effective_to, reason, created_at";
fn map(r: &Row<'_>) -> rusqlite::Result<PackagePrice> {
    Ok(PackagePrice {
        id: r.get(0)?,
        product_package_id: r.get(1)?,
        selling_price_minor: r.get(2)?,
        cost_price_minor: r.get(3)?,
        effective_from: r.get(4)?,
        effective_to: r.get(5)?,
        reason: r.get(6)?,
        created_at: r.get(7)?,
    })
}
pub(crate) fn current_on(
    c: &Connection,
    package_id: &str,
) -> Result<Option<PackagePrice>, AppError> {
    Ok(c.query_row(&format!("SELECT {COLUMNS} FROM product_price_history WHERE product_package_id=?1 AND effective_to IS NULL"),[package_id],map).optional()?)
}
pub fn current(db: &AppDb, package_id: &str) -> Result<Option<PackagePrice>, AppError> {
    id(package_id)?;
    let c = db.lock()?;
    super::product_packages::get(&c, package_id)?;
    current_on(&c, package_id)
}
pub fn history(db: &AppDb, package_id: &str) -> Result<Vec<PackagePrice>, AppError> {
    id(package_id)?;
    let c = db.lock()?;
    super::product_packages::get(&c, package_id)?;
    let mut s=c.prepare(&format!("SELECT {COLUMNS} FROM product_price_history WHERE product_package_id=?1 ORDER BY effective_from DESC,id"))?;
    let rows = s.query_map([package_id], map)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
fn validate_price(input: &SetPackagePrice) -> Result<(), AppError> {
    id(&input.product_package_id)?;
    optional(&input.reason)?;
    if !(0..=MAX_SAFE_MINOR).contains(&input.selling_price_minor)
        || input
            .cost_price_minor
            .is_some_and(|v| !(0..=MAX_SAFE_MINOR).contains(&v))
    {
        return Err(invalid(
            "Prices must be non-negative integer minor units within the supported range.",
        ));
    }
    Ok(())
}
/// Effective immediately. No backdating/scheduling. Monotonic milliseconds handle fast
/// updates and clock rollback. Runs inside a caller-owned transaction; the product and
/// package must be active, and a no-op save returns the current row unchanged so repeat
/// submissions never duplicate history.
pub(crate) fn set_checked_on(
    c: &Connection,
    input: &SetPackagePrice,
) -> Result<PackagePrice, AppError> {
    let package = super::product_packages::get(c, &input.product_package_id)?;
    let product = super::products::get(c, &package.product_id)?;
    if !package.is_active || !product.is_active {
        return Err(invalid(
            "Reactivate the product and package before setting a new price.",
        ));
    }
    let old = current_on(c, &input.product_package_id)?;
    if let Some(ref old) = old {
        if old.selling_price_minor == input.selling_price_minor
            && old.cost_price_minor == input.cost_price_minor
        {
            return Ok(old.clone());
        }
    }
    let now:String=c.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),coalesce(julianday(?1)+0.001/86400,0)))",[old.as_ref().map(|p|p.effective_from.as_str())],|r|r.get(0))?;
    if let Some(old) = old {
        c.execute(
            "UPDATE product_price_history SET effective_to=?1 WHERE id=?2",
            params![now, old.id],
        )?;
    }
    let value = uuid::Uuid::new_v4().to_string();
    c.execute("INSERT INTO product_price_history(id,product_package_id,selling_price_minor,cost_price_minor,effective_from,reason) VALUES (?1,?2,?3,?4,?5,?6)",params![value,input.product_package_id,input.selling_price_minor,input.cost_price_minor,now,input.reason])?;
    let price = c.query_row(
        &format!("SELECT {COLUMNS} FROM product_price_history WHERE id=?1"),
        [value],
        map,
    )?;
    Ok(price)
}
/// Effective immediately. No backdating/scheduling. Monotonic milliseconds handle fast updates and clock rollback.
pub fn set(db: &AppDb, input: &SetPackagePrice) -> Result<PackagePrice, AppError> {
    validate_price(input)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let price = set_checked_on(&tx, input)?;
    tx.commit()?;
    Ok(price)
}
