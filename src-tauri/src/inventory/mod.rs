pub mod balances;
pub mod batches;
pub mod commands;
pub mod expiry;
pub mod levels;
pub mod models;
pub mod movements;
pub mod transfers;
use crate::{
    db::catalog_validation::{id, invalid, MAX_SAFE_MINOR},
    error::AppError,
};
use rusqlite::Connection;
pub(crate) fn scope(
    c: &Connection,
    branch: &str,
    package: &str,
    active: bool,
) -> Result<(), AppError> {
    id(branch)?;
    id(package)?;
    let valid:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM branches b JOIN product_packages pk ON pk.id=?2 JOIN products p ON p.id=pk.product_id WHERE b.id=?1 AND (?3=0 OR (b.is_active=1 AND pk.is_active=1 AND p.is_active=1)))",rusqlite::params![branch,package,active],|r|r.get(0))?;
    if !valid {
        return Err(invalid(
            "A valid branch and package are required; posting requires active records.",
        ));
    }
    Ok(())
}
pub(crate) fn quantity(v: i64) -> Result<(), AppError> {
    if !(1..=MAX_SAFE_MINOR).contains(&v) {
        Err(invalid(
            "Quantity must be a positive whole package count within the safe integer range.",
        ))
    } else {
        Ok(())
    }
}
pub(crate) fn page(limit: Option<i64>, offset: Option<i64>) -> Result<(i64, i64), AppError> {
    let (l, o) = (limit.unwrap_or(50), offset.unwrap_or(0));
    if !(1..=200).contains(&l) || !(0..=MAX_SAFE_MINOR).contains(&o) {
        Err(invalid("Use a limit of 1–200 and a non-negative offset."))
    } else {
        Ok((l, o))
    }
}
pub(crate) fn date(c: &Connection, date: &str) -> Result<(), AppError> {
    let valid: bool = c.query_row(
        "SELECT length(?1)=10 AND date(?1,'+0 days') IS ?1",
        [date],
        |r| r.get(0),
    )?;
    if valid {
        Ok(())
    } else {
        Err(invalid("Use a real calendar date in YYYY-MM-DD format."))
    }
}
