use crate::{
    db::{
        catalog_validation::*,
        connection::AppDb,
        models::suppliers::*,
    },
    error::{AppError, ErrorCode},
};
use rusqlite::{params, Connection, OptionalExtension, Row, TransactionBehavior};

const COLUMNS: &str = "id, code, name, phone, email, address, notes, is_active, created_at, updated_at";
fn map(r: &Row<'_>) -> rusqlite::Result<Supplier> {
    Ok(Supplier {
        id: r.get(0)?,
        code: r.get(1)?,
        name: r.get(2)?,
        phone: r.get(3)?,
        email: r.get(4)?,
        address: r.get(5)?,
        notes: r.get(6)?,
        is_active: r.get(7)?,
        created_at: r.get(8)?,
        updated_at: r.get(9)?,
    })
}
fn validate_page(limit_input: Option<i64>, offset_input: Option<i64>) -> Result<(i64, i64), AppError> {
    let limit = limit_input.unwrap_or(50);
    let offset = offset_input.unwrap_or(0);
    if !(1..=200).contains(&limit) || !(0..=MAX_SAFE_MINOR).contains(&offset) {
        return Err(invalid("Use a limit of 1–200 and a non-negative offset."));
    }
    Ok((limit, offset))
}
fn search_pattern(value: &str) -> (String, String) {
    let search = normalize(value);
    let pattern = format!(
        "{}%",
        search
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    );
    (search, pattern)
}
fn validate_fields(name: &str, phone: &str, email: &str, address: &str, notes: &str) -> Result<(), AppError> {
    if name.trim().is_empty() || name.chars().count() > 180 {
        return Err(invalid("A supplier name of 1–180 characters is required."));
    }
    if phone.chars().count() > 40
        || email.chars().count() > 120
        || address.chars().count() > 300
        || notes.chars().count() > 500
    {
        return Err(invalid("A supplier contact field is too long."));
    }
    Ok(())
}
fn code_base(name: &str) -> String {
    let base: String = name
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .take(3)
        .collect::<String>()
        .to_ascii_uppercase();
    if base.is_empty() {
        "SUP".into()
    } else {
        base
    }
}
pub(crate) fn code_exists(c: &Connection, code: &str, exclude_id: &str) -> Result<bool, AppError> {
    Ok(c.query_row(
        "SELECT count(*) FROM suppliers WHERE code=?1 AND id<>?2",
        params![code, exclude_id],
        |r| r.get::<_, i64>(0),
    )? > 0)
}
fn unique_code_on(c: &Connection, base: &str) -> Result<String, AppError> {
    for _ in 0..12 {
        let suffix: String = uuid::Uuid::new_v4().simple().to_string().chars().take(4).collect();
        let code = format!("{base}-{suffix}");
        if !code_exists(c, &code, "")? {
            return Ok(code);
        }
    }
    Err(invalid("Could not allocate a unique supplier code. Retry."))
}
pub(crate) fn insert(
    c: &Connection,
    code: &str,
    name: &str,
    phone: &str,
    email: &str,
    address: &str,
    notes: &str,
) -> Result<Supplier, AppError> {
    let value = uuid::Uuid::new_v4().to_string();
    c.execute(
        "INSERT INTO suppliers(id,code,name,phone,email,address,notes) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![value, code, name, phone, email, address, notes],
    )?;
    get(c, &value)
}
fn validate_create(input: &CreateSupplierInput) -> Result<(), AppError> {
    validate_fields(&input.name, &input.phone, &input.email, &input.address, &input.notes)
}
pub fn create(db: &AppDb, input: &CreateSupplierInput) -> Result<Supplier, AppError> {
    validate_create(input)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let code = unique_code_on(&tx, &code_base(&input.name))?;
    let supplier = insert(
        &tx,
        &code,
        input.name.trim(),
        input.phone.trim(),
        input.email.trim(),
        input.address.trim(),
        input.notes.trim(),
    )?;
    tx.commit()?;
    Ok(supplier)
}
pub(crate) fn get(c: &Connection, value: &str) -> Result<Supplier, AppError> {
    c.query_row(
        &format!("SELECT {COLUMNS} FROM suppliers WHERE id=?1"),
        [value],
        map,
    )
    .optional()?
    .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Supplier not found."))
}
pub fn get_supplier(db: &AppDb, value: &str) -> Result<Supplier, AppError> {
    id(value)?;
    let c = db.lock()?;
    get(&c, value)
}
fn validate_update(input: &UpdateSupplierInput) -> Result<(), AppError> {
    id(&input.id)?;
    validate_fields(&input.name, &input.phone, &input.email, &input.address, &input.notes)
}
pub(crate) fn update_on(
    c: &Connection,
    value: &str,
    name: &str,
    phone: &str,
    email: &str,
    address: &str,
    notes: &str,
) -> Result<Supplier, AppError> {
    c.execute(
        "UPDATE suppliers SET name=?1,phone=?2,email=?3,address=?4,notes=?5,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),julianday(updated_at)+0.001/86400)) WHERE id=?6",
        params![name, phone, email, address, notes, value],
    )?;
    get(c, value)
}
pub fn update(db: &AppDb, input: &UpdateSupplierInput) -> Result<Supplier, AppError> {
    validate_update(input)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    get(&tx, &input.id)?;
    let supplier = update_on(
        &tx,
        &input.id,
        input.name.trim(),
        input.phone.trim(),
        input.email.trim(),
        input.address.trim(),
        input.notes.trim(),
    )?;
    tx.commit()?;
    Ok(supplier)
}
pub(crate) fn set_active_on(c: &Connection, value: &str, active: bool) -> Result<Supplier, AppError> {
    c.execute(
        "UPDATE suppliers SET is_active=?1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),julianday(updated_at)+0.001/86400)) WHERE id=?2",
        params![active, value],
    )?;
    get(c, value)
}
pub fn set_active(db: &AppDb, value: &str, active: bool) -> Result<Supplier, AppError> {
    id(value)?;
    let c = db.lock()?;
    set_active_on(&c, value, active)
}
pub fn list(db: &AppDb, q: &SupplierQuery) -> Result<SupplierPage, AppError> {
    if q.search.as_ref().is_some_and(|v| v.chars().count() > 4000) {
        return Err(invalid("Search text is too long."));
    }
    let (limit, offset) = validate_page(q.limit, q.offset)?;
    let (search, pattern) = search_pattern(q.search.as_deref().unwrap_or(""));
    let filter = r"WHERE (?1 IS NULL OR is_active=?1) AND (?2='' OR code LIKE ?3 ESCAPE '\' OR name LIKE ?3 ESCAPE '\' OR phone LIKE ?3 ESCAPE '\' OR email LIKE ?3 ESCAPE '\')";
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let total = tx.query_row(
        &format!("SELECT count(*) FROM suppliers {filter}"),
        params![q.is_active, search, pattern],
        |r| r.get(0),
    )?;
    let items = {
        let mut s = tx.prepare(&format!(
            "SELECT {COLUMNS} FROM suppliers {filter} ORDER BY name COLLATE NOCASE, id LIMIT ?4 OFFSET ?5"
        ))?;
        let rows = s.query_map(params![q.is_active, search, pattern, limit, offset], map)?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    tx.commit()?;
    Ok(SupplierPage {
        items,
        total,
        limit,
        offset,
    })
}