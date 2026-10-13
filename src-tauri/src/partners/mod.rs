//! Canonical Phase 5A supplier/customer foundation. Purchasing remains disabled.
pub mod commands;
pub mod models;

use crate::{
    cloud::replication,
    db::{catalog_validation::*, connection::AppDb},
    error::{AppError, ErrorCode},
};
use models::*;
use rusqlite::{params, Connection, OptionalExtension, Row, TransactionBehavior};

const SUPPLIER_COLUMNS: &str = "id,code,display_name,contact_person,primary_phone,secondary_phone,email,address,tax_registration_id,notes,is_active,revision,created_at,updated_at";
const CUSTOMER_COLUMNS: &str = "id,code,display_name,primary_phone,secondary_phone,email,address,notes,is_active,revision,created_at,updated_at";

fn optional_text(
    value: &Option<String>,
    max: usize,
    label: &str,
) -> Result<Option<String>, AppError> {
    let value = value.as_deref().unwrap_or("").trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > max || value.contains('\0') {
        return Err(invalid(&format!(
            "{label} must be at most {max} characters."
        )));
    }
    Ok(Some(value.to_string()))
}

fn display_name(value: &str, label: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 180 || value.contains('\0') {
        return Err(invalid(&format!(
            "A {label} name of 1–180 characters is required."
        )));
    }
    Ok(value.to_string())
}

fn email(value: &Option<String>) -> Result<Option<String>, AppError> {
    let value = optional_text(value, 254, "Email")?;
    if let Some(email) = &value {
        let mut parts = email.split('@');
        if email.chars().any(char::is_whitespace)
            || parts.next().is_none_or(str::is_empty)
            || parts
                .next()
                .is_none_or(|p| p.is_empty() || !p.contains('.'))
            || parts.next().is_some()
        {
            return Err(invalid("Enter a valid email address or leave it empty."));
        }
    }
    Ok(value)
}

fn page(query: &PartnerQuery) -> Result<(i64, i64, String), AppError> {
    let limit = query.limit.unwrap_or(25);
    let offset = query.offset.unwrap_or(0);
    if !(1..=100).contains(&limit) || offset < 0 || offset > MAX_SAFE_MINOR {
        return Err(invalid("Use a limit of 1–100 and a non-negative offset."));
    }
    let search = query.search.as_deref().unwrap_or("").trim();
    if search.chars().count() > 200 || search.contains('\0') {
        return Err(invalid("Search text must be at most 200 characters."));
    }
    Ok((limit, offset, search.to_string()))
}

fn next_code(c: &Connection, kind: &str, prefix: &str) -> Result<String, AppError> {
    c.execute(
        "INSERT INTO partner_sequences(kind,next_number) VALUES(?1,1) ON CONFLICT(kind) DO NOTHING",
        [kind],
    )?;
    let number: i64 = c.query_row(
        "SELECT next_number FROM partner_sequences WHERE kind=?1",
        [kind],
        |r| r.get(0),
    )?;
    c.execute(
        "UPDATE partner_sequences SET next_number=next_number+1 WHERE kind=?1",
        [kind],
    )?;
    Ok(format!("{prefix}-{number:06}"))
}

fn map_supplier(r: &Row<'_>) -> rusqlite::Result<Supplier> {
    Ok(Supplier {
        id: r.get(0)?,
        code: r.get(1)?,
        display_name: r.get(2)?,
        contact_person: r.get(3)?,
        primary_phone: r.get(4)?,
        secondary_phone: r.get(5)?,
        email: r.get(6)?,
        address: r.get(7)?,
        tax_registration_id: r.get(8)?,
        notes: r.get(9)?,
        is_active: r.get(10)?,
        revision: r.get(11)?,
        created_at: r.get(12)?,
        updated_at: r.get(13)?,
    })
}
fn map_customer(r: &Row<'_>) -> rusqlite::Result<Customer> {
    Ok(Customer {
        id: r.get(0)?,
        code: r.get(1)?,
        display_name: r.get(2)?,
        primary_phone: r.get(3)?,
        secondary_phone: r.get(4)?,
        email: r.get(5)?,
        address: r.get(6)?,
        notes: r.get(7)?,
        is_active: r.get(8)?,
        revision: r.get(9)?,
        created_at: r.get(10)?,
        updated_at: r.get(11)?,
    })
}
fn supplier_on(c: &Connection, value: &str) -> Result<Supplier, AppError> {
    c.query_row(
        &format!("SELECT {SUPPLIER_COLUMNS} FROM suppliers WHERE id=?1"),
        [value],
        map_supplier,
    )
    .optional()?
    .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Supplier not found."))
}
fn customer_on(c: &Connection, value: &str) -> Result<Customer, AppError> {
    c.query_row(
        &format!("SELECT {CUSTOMER_COLUMNS} FROM customers WHERE id=?1"),
        [value],
        map_customer,
    )
    .optional()?
    .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Customer not found."))
}

type SupplierValues = (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);
fn supplier_values(f: &SupplierFields) -> Result<SupplierValues, AppError> {
    Ok((
        display_name(&f.display_name, "supplier")?,
        optional_text(&f.contact_person, 120, "Contact person")?,
        optional_text(&f.primary_phone, 40, "Primary phone")?,
        optional_text(&f.secondary_phone, 40, "Secondary phone")?,
        email(&f.email)?,
        optional_text(&f.address, 500, "Address")?,
        optional_text(&f.tax_registration_id, 64, "Tax/registration identifier")?,
        optional_text(&f.notes, 2000, "Notes")?,
    ))
}
type CustomerValues = (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);
fn customer_values(f: &CustomerFields) -> Result<CustomerValues, AppError> {
    Ok((
        display_name(&f.display_name, "customer")?,
        optional_text(&f.primary_phone, 40, "Primary phone")?,
        optional_text(&f.secondary_phone, 40, "Secondary phone")?,
        email(&f.email)?,
        optional_text(&f.address, 500, "Address")?,
        optional_text(&f.notes, 2000, "Notes")?,
    ))
}

pub fn create_supplier(db: &AppDb, f: &SupplierFields) -> Result<Supplier, AppError> {
    let v = supplier_values(f)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let value = uuid::Uuid::new_v4().to_string();
    let code = next_code(&tx, "SUPPLIER", "SUP")?;
    tx.execute("INSERT INTO suppliers(id,code,display_name,contact_person,primary_phone,secondary_phone,email,address,tax_registration_id,notes) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",params![value,code,v.0,v.1,v.2,v.3,v.4,v.5,v.6,v.7])?;
    let result = supplier_on(&tx, &value)?;
    replication::partner_record(&tx, "suppliers", &value)?;
    tx.commit()?;
    Ok(result)
}
pub fn create_customer(db: &AppDb, f: &CustomerFields) -> Result<Customer, AppError> {
    let v = customer_values(f)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let value = uuid::Uuid::new_v4().to_string();
    let code = next_code(&tx, "CUSTOMER", "CUS")?;
    tx.execute("INSERT INTO customers(id,code,display_name,primary_phone,secondary_phone,email,address,notes) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![value,code,v.0,v.1,v.2,v.3,v.4,v.5])?;
    let result = customer_on(&tx, &value)?;
    replication::partner_record(&tx, "customers", &value)?;
    tx.commit()?;
    Ok(result)
}
pub fn get_supplier(db: &AppDb, value: &str) -> Result<Supplier, AppError> {
    id(value)?;
    let connection = db.lock()?;
    supplier_on(&connection, value)
}
pub fn get_customer(db: &AppDb, value: &str) -> Result<Customer, AppError> {
    id(value)?;
    let connection = db.lock()?;
    customer_on(&connection, value)
}

pub fn update_supplier(db: &AppDb, input: &UpdateSupplier) -> Result<Supplier, AppError> {
    id(&input.id)?;
    if input.expected_revision < 1 {
        return Err(invalid("A valid expected revision is required."));
    }
    let v = supplier_values(&input.fields)?;
    let c = db.lock()?;
    let changed=c.execute("UPDATE suppliers SET display_name=?1,contact_person=?2,primary_phone=?3,secondary_phone=?4,email=?5,address=?6,tax_registration_id=?7,notes=?8,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?9 AND revision=?10",params![v.0,v.1,v.2,v.3,v.4,v.5,v.6,v.7,input.id,input.expected_revision])?;
    if changed == 0 {
        supplier_on(&c, &input.id)?;
        return Err(AppError::new(
            ErrorCode::Conflict,
            "Supplier changed. Reload before saving.",
        ));
    }
    replication::partner_record(&c, "suppliers", &input.id)?;
    supplier_on(&c, &input.id)
}
pub fn update_customer(db: &AppDb, input: &UpdateCustomer) -> Result<Customer, AppError> {
    id(&input.id)?;
    if input.expected_revision < 1 {
        return Err(invalid("A valid expected revision is required."));
    }
    let v = customer_values(&input.fields)?;
    let c = db.lock()?;
    let changed=c.execute("UPDATE customers SET display_name=?1,primary_phone=?2,secondary_phone=?3,email=?4,address=?5,notes=?6,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?7 AND revision=?8",params![v.0,v.1,v.2,v.3,v.4,v.5,input.id,input.expected_revision])?;
    if changed == 0 {
        customer_on(&c, &input.id)?;
        return Err(AppError::new(
            ErrorCode::Conflict,
            "Customer changed. Reload before saving.",
        ));
    }
    replication::partner_record(&c, "customers", &input.id)?;
    customer_on(&c, &input.id)
}

pub fn set_supplier_active(db: &AppDb, input: &SetPartnerActive) -> Result<Supplier, AppError> {
    id(&input.id)?;
    if input.expected_revision < 1 {
        return Err(invalid("A valid expected revision is required."));
    }
    let c = db.lock()?;
    let changed=c.execute("UPDATE suppliers SET is_active=?1,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?2 AND revision=?3 AND is_active<>?1",params![input.active,input.id,input.expected_revision])?;
    if changed == 0 {
        let current = supplier_on(&c, &input.id)?;
        if current.revision != input.expected_revision {
            return Err(AppError::new(
                ErrorCode::Conflict,
                "Supplier changed. Reload before changing status.",
            ));
        }
        return Err(invalid("Supplier already has that status."));
    }
    supplier_on(&c, &input.id)
}
pub fn set_customer_active(db: &AppDb, input: &SetPartnerActive) -> Result<Customer, AppError> {
    id(&input.id)?;
    if input.expected_revision < 1 {
        return Err(invalid("A valid expected revision is required."));
    }
    let c = db.lock()?;
    let changed=c.execute("UPDATE customers SET is_active=?1,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?2 AND revision=?3 AND is_active<>?1",params![input.active,input.id,input.expected_revision])?;
    if changed == 0 {
        let current = customer_on(&c, &input.id)?;
        if current.revision != input.expected_revision {
            return Err(AppError::new(
                ErrorCode::Conflict,
                "Customer changed. Reload before changing status.",
            ));
        }
        return Err(invalid("Customer already has that status."));
    }
    customer_on(&c, &input.id)
}

pub fn list_suppliers(db: &AppDb, q: &PartnerQuery) -> Result<PartnerPage<Supplier>, AppError> {
    let (limit, offset, search) = page(q)?;
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let filter="WHERE (?1 IS NULL OR is_active=?1) AND (?2='' OR instr(lower(code||' '||display_name||' '||coalesce(contact_person,'')||' '||coalesce(primary_phone,'')||' '||coalesce(secondary_phone,'')||' '||coalesce(email,'')||' '||coalesce(tax_registration_id,'')),lower(?2))>0)";
    let total = tx.query_row(
        &format!("SELECT count(*) FROM suppliers {filter}"),
        params![q.is_active, search],
        |r| r.get(0),
    )?;
    let items = {
        let mut s=tx.prepare(&format!("SELECT {SUPPLIER_COLUMNS} FROM suppliers {filter} ORDER BY display_name COLLATE NOCASE,id LIMIT ?3 OFFSET ?4"))?;
        let rows = s
            .query_map(params![q.is_active, search, limit, offset], map_supplier)?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    tx.commit()?;
    Ok(PartnerPage {
        items,
        total,
        limit,
        offset,
    })
}
pub fn list_customers(db: &AppDb, q: &PartnerQuery) -> Result<PartnerPage<Customer>, AppError> {
    let (limit, offset, search) = page(q)?;
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let filter="WHERE (?1 IS NULL OR is_active=?1) AND (?2='' OR instr(lower(code||' '||display_name||' '||coalesce(primary_phone,'')||' '||coalesce(secondary_phone,'')||' '||coalesce(email,'')),lower(?2))>0)";
    let total = tx.query_row(
        &format!("SELECT count(*) FROM customers {filter}"),
        params![q.is_active, search],
        |r| r.get(0),
    )?;
    let items = {
        let mut s=tx.prepare(&format!("SELECT {CUSTOMER_COLUMNS} FROM customers {filter} ORDER BY display_name COLLATE NOCASE,id LIMIT ?3 OFFSET ?4"))?;
        let rows = s
            .query_map(params![q.is_active, search, limit, offset], map_customer)?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    tx.commit()?;
    Ok(PartnerPage {
        items,
        total,
        limit,
        offset,
    })
}
