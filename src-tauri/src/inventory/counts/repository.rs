use super::{models::*, *};
use rusqlite::{Connection, OptionalExtension, Row};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
fn decode<T: DeserializeOwned>(r: &Row) -> rusqlite::Result<T> {
    let json: String = r.get(0)?;
    serde_json::from_str(&json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}
const HEADER:&str="json_object('id',id,'sessionNumber',session_number,'branchId',branch_id,'status',status,'scopeType',scope_type,'notes',notes,'createdBy',created_by,'revision',revision,'snapshotWatermark',snapshot_watermark,'createdAt',created_at,'startedAt',started_at,'reviewedAt',reviewed_at,'completedAt',completed_at,'cancelledAt',cancelled_at)";
pub fn session(c: &Connection, id: &str) -> Result<Session, AppError> {
    crate::db::catalog_validation::id(id)?;
    c.query_row(
        &format!("SELECT {HEADER} FROM stock_count_sessions WHERE id=?1"),
        [id],
        decode,
    )
    .optional()?
    .ok_or_else(|| invalid("Stock count session not found."))
}
pub fn lines(c: &Connection, id: &str, limit: i64, offset: i64) -> Result<Vec<Line>, AppError> {
    let mut stmt=c.prepare("SELECT json_object('id',id,'sessionId',session_id,'productPackageId',product_package_id,'batchId',batch_id,'expectedQuantity',expected_quantity,'countedQuantity',counted_quantity,'notes',notes,'excludedReason',excluded_reason,'countedAt',counted_at,'revision',revision,'productLabel',product_label,'packageLabel',package_label,'lotNumber',lot_number,'expiryDate',expiry_date,'currentLive',current_live,'snapshotVariance',snapshot_variance,'correctionDelta',correction_delta,'interveningMovements',intervening_movements,'movementId',movement_id) FROM stock_count_line_view WHERE session_id=?1 ORDER BY id LIMIT ?2 OFFSET ?3")?;
    let rows = stmt
        .query_map(params![id, limit, offset], decode)?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}
pub fn summary(c: &Connection, id: &str) -> Result<Summary, AppError> {
    Ok(c.query_row("SELECT json_object('total',count(*),'counted',count(counted_quantity),'uncounted',coalesce(sum(counted_quantity IS NULL AND excluded_reason IS NULL),0),'excluded',count(excluded_reason),'matches',coalesce(sum(snapshot_variance=0),0),'positive',coalesce(sum(snapshot_variance>0),0),'negative',coalesce(sum(snapshot_variance<0),0),'corrections',coalesce(sum(correction_delta!=0),0),'intervening',coalesce(sum(intervening_movements>0),0)) FROM stock_count_line_view WHERE session_id=?1",[id],decode)?)
}
pub fn token(c: &Connection, s: &Session) -> Result<String, AppError> {
    let mut h = Sha256::new();
    h.update(s.id.as_bytes());
    h.update(s.revision.to_le_bytes());
    let mut stmt=c.prepare("SELECT l.id,l.revision,coalesce(sum(m.quantity_delta),0),coalesce(max(m.rowid),0),count(m.id) FROM stock_count_lines l LEFT JOIN inventory_movements m ON m.batch_id=l.batch_id WHERE l.session_id=?1 GROUP BY l.id ORDER BY l.id")?;
    let mut rows = stmt.query([&s.id])?;
    while let Some(r) = rows.next()? {
        h.update(r.get::<_, String>(0)?.as_bytes());
        for n in 1..5 {
            h.update(r.get::<_, i64>(n)?.to_le_bytes());
        }
    }
    Ok(format!("{:x}", h.finalize()))
}
pub fn detail(c: &Connection, id: &str, limit: i64, offset: i64) -> Result<Detail, AppError> {
    let session = session(c, id)?;
    Ok(Detail {
        review_token: token(c, &session)?,
        summary: summary(c, id)?,
        lines: lines(c, id, limit, offset)?,
        session,
        limit,
        offset,
    })
}
pub fn insert_line(
    c: &Connection,
    s: &Session,
    batch: &str,
    notes: Option<&str>,
) -> Result<(), AppError> {
    c.execute("INSERT INTO stock_count_lines(id,session_id,branch_id,product_package_id,batch_id,expected_quantity,product_label,package_label,notes) SELECT ?1,?2,b.branch_id,b.product_package_id,b.id,(SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements WHERE batch_id=b.id AND rowid<=?3),coalesce(p.commercial_name_en,p.commercial_name_ar,p.scientific_name,'Unnamed product'),pk.package_label,?5 FROM inventory_batches b JOIN product_packages pk ON pk.id=b.product_package_id JOIN products p ON p.id=pk.product_id WHERE b.id=?4",params![uuid(),s.id,s.snapshot_watermark,batch,notes])?;
    Ok(())
}
pub fn list(c: &Connection, q: &Query) -> Result<Page, AppError> {
    let (limit, offset) = inventory::page(q.limit, q.offset)?;
    active_branch(c, &q.branch_id)?;
    for date in [&q.date_from, &q.date_to].into_iter().flatten() {
        inventory::date(c, date)?;
    }
    if q.date_from.is_some() && q.date_to.is_some() && q.date_from > q.date_to {
        return Err(invalid("From date must not follow to date."));
    }
    let status = q.status.as_ref().map(|s| {
        serde_json::to_value(s)
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned()
    });
    let filter="branch_id=?1 AND (?2 IS NULL OR status=?2) AND (?3 IS NULL OR created_at>=?3) AND (?4 IS NULL OR created_at<datetime(?4,'+1 day'))";
    let total = c.query_row(
        &format!("SELECT count(*) FROM stock_count_sessions WHERE {filter}"),
        params![q.branch_id, status, q.date_from, q.date_to],
        |r| r.get(0),
    )?;
    let mut stmt=c.prepare(&format!("SELECT {HEADER} FROM stock_count_sessions WHERE {filter} ORDER BY created_at DESC,id LIMIT ?5 OFFSET ?6"))?;
    let sessions = stmt
        .query_map(
            params![q.branch_id, status, q.date_from, q.date_to, limit, offset],
            decode,
        )?
        .collect::<Result<Vec<Session>, _>>()?;
    let items = sessions
        .into_iter()
        .map(|session| {
            Ok(SessionRow {
                summary: summary(c, &session.id)?,
                session,
            })
        })
        .collect::<Result<_, AppError>>()?;
    Ok(Page {
        items,
        total,
        limit,
        offset,
    })
}
