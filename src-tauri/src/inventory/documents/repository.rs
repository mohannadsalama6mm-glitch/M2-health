use super::models::*;
use crate::{
    db::catalog_validation::{id, invalid},
    error::AppError,
    inventory,
};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::de::DeserializeOwned;

pub fn decode<T: DeserializeOwned>(r: &Row<'_>) -> rusqlite::Result<T> {
    let value: String = r.get(0)?;
    serde_json::from_str(&value).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}
const HEADER: &str = "json_object('id',id,'documentNumber',document_number,'kind',kind,'status',status,'revision',revision,'branchId',branch_id,'destinationBranchId',destination_branch_id,'referenceText',reference_text,'supplierReference',supplier_reference,'notes',notes,'createdBy',created_by,'reversalOf',reversal_of,'reversalReason',reversal_reason,'createdAt',created_at,'updatedAt',updated_at,'postedAt',posted_at)";
pub fn header(c: &Connection, value: &str) -> Result<Header, AppError> {
    id(value)?;
    c.query_row(
        &format!("SELECT {HEADER} FROM inventory_documents WHERE id=?1"),
        [value],
        decode,
    )
    .optional()?
    .ok_or_else(|| invalid("Inventory document not found."))
}
pub fn detail(c: &Connection, value: &str) -> Result<Document, AppError> {
    let header = header(c, value)?;
    let lines=c.prepare("SELECT json_object('id',id,'productPackageId',product_package_id,'batchId',batch_id,'action',action,'quantity',quantity,'reason',reason,'lotNumber',lot_number,'expiryDate',expiry_date,'costPriceMinor',cost_price_minor,'supplierReference',supplier_reference,'productLabel',product_label,'packageLabel',package_label) FROM inventory_document_lines WHERE document_id=?1 ORDER BY position")?.query_map([value],decode)?.collect::<Result<Vec<Line>,_>>()?;
    let movements=c.prepare("SELECT json_object('movementId',m.id,'lineId',dm.line_id,'reversesMovementId',dm.reverses_movement_id,'branchId',m.branch_id,'batchId',m.batch_id,'productPackageId',m.product_package_id,'movementType',m.movement_type,'quantityDelta',m.quantity_delta) FROM inventory_document_movements dm JOIN inventory_movements m ON m.id=dm.movement_id WHERE dm.document_id=?1 ORDER BY dm.line_id,m.quantity_delta")?.query_map([value],decode)?.collect::<Result<Vec<Link>,_>>()?;
    let mut running = std::collections::HashMap::<String, i64>::new();
    let mut preview = Vec::new();
    for line in &lines {
        let (current, expiry) = if let Some(batch) = &line.input.batch_id {
            c.query_row("SELECT coalesce((SELECT sum(quantity_delta) FROM inventory_movements WHERE batch_id=b.id),0),expiry_date FROM inventory_batches b WHERE id=?1",[batch],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,Option<String>>(1)?)))?
        } else {
            (0, line.input.expiry_date.clone())
        };
        let key = line
            .input
            .batch_id
            .clone()
            .unwrap_or_else(|| line.id.clone());
        let delta = if line.input.action.outgoing() {
            -line.input.quantity
        } else {
            line.input.quantity
        };
        let projected = running
            .get(&key)
            .copied()
            .unwrap_or(current)
            .checked_add(delta)
            .ok_or_else(|| invalid("Projected stock exceeds the supported range."))?;
        running.insert(key, projected);
        preview.push(Preview {
            line_id: line.id.clone(),
            current_stock: current,
            projected_stock: projected,
            expiry_date: expiry,
        });
    }
    Ok(Document {
        header,
        lines,
        movements,
        preview,
    })
}
pub fn insert_line(
    c: &Connection,
    doc: &str,
    position: usize,
    l: &LineInput,
) -> Result<String, AppError> {
    let (product,package):(String,String)=c.query_row("SELECT coalesce(nullif(p.commercial_name_en,''),p.commercial_name_ar,p.scientific_name,'Unnamed product'),pk.package_label FROM product_packages pk JOIN products p ON p.id=pk.product_id WHERE pk.id=?1",[&l.product_package_id],|r|Ok((r.get(0)?,r.get(1)?)))?;
    let line = uuid::Uuid::new_v4().to_string();
    c.execute("INSERT INTO inventory_document_lines(id,document_id,position,product_package_id,batch_id,action,quantity,reason,lot_number,expiry_date,cost_price_minor,supplier_reference,product_label,package_label) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",params![line,doc,position as i64,l.product_package_id,l.batch_id,l.action.text(),l.quantity,l.reason.trim(),l.lot_number,l.expiry_date,l.cost_price_minor,l.supplier_reference,product,package])?;
    Ok(line)
}
pub fn number(c: &Connection, kind: Kind) -> Result<String, AppError> {
    let day: String = c.query_row("SELECT strftime('%Y%m%d','now')", [], |r| r.get(0))?;
    let n:i64=c.query_row("INSERT INTO inventory_document_sequences(day,prefix,next_number) VALUES(?1,?2,1) ON CONFLICT(day,prefix) DO UPDATE SET next_number=next_number+1 RETURNING next_number",params![day,kind.prefix()],|r|r.get(0))?;
    Ok(format!("{}-{}-{:04}", kind.prefix(), day, n))
}
pub fn list(c: &Connection, q: &DocumentQuery) -> Result<DocumentPage, AppError> {
    id(&q.branch_id)?;
    let (limit, offset) = inventory::page(q.limit, q.offset)?;
    for date in [&q.date_from, &q.date_to].into_iter().flatten() {
        inventory::date(c, date)?;
    }
    if q.date_from
        .as_ref()
        .zip(q.date_to.as_ref())
        .is_some_and(|(a, b)| a > b)
    {
        return Err(invalid("Start date must precede end date."));
    }
    let status = q.status.map(|v| {
        serde_json::to_value(v)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string()
    });
    let filter="WHERE branch_id=?1 AND (?2 IS NULL OR kind=?2) AND (?3 IS NULL OR status=?3) AND (?4 IS NULL OR created_at>=?4) AND (?5 IS NULL OR created_at<date(?5,'+1 day'))";
    let total = c.query_row(
        &format!("SELECT count(*) FROM inventory_documents {filter}"),
        params![
            q.branch_id,
            q.kind.map(Kind::text),
            status,
            q.date_from,
            q.date_to
        ],
        |r| r.get(0),
    )?;
    let items=c.prepare(&format!("SELECT {HEADER} FROM inventory_documents {filter} ORDER BY created_at DESC,id DESC LIMIT ?6 OFFSET ?7"))?.query_map(params![q.branch_id,q.kind.map(Kind::text),status,q.date_from,q.date_to,limit,offset],decode)?.collect::<Result<Vec<Header>,_>>()?;
    Ok(DocumentPage {
        items,
        total,
        limit,
        offset,
    })
}
