//! Read-only operational projections over the canonical ledger. No schema or posting logic.
use super::*;
use crate::{commands::branches::DatabaseState, db::connection::AppDb};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;
#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StockState {
    InStock,
    LowStock,
    OutOfStock,
}
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Query {
    pub branch_id: String,
    pub product_package_id: Option<String>,
    pub batch_id: Option<String>,
    pub search: Option<String>,
    pub stock_state: Option<StockState>,
    pub manufacturer_id: Option<String>,
    pub category_id: Option<String>,
    pub route_id: Option<String>,
    pub movement_type: Option<models::MovementType>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub document_reference: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub items: Vec<Value>,
    pub total: i64,
    pub business_date: String,
}
fn validate(c: &Connection, q: &Query) -> Result<(i64, i64), AppError> {
    crate::db::catalog_validation::id(&q.branch_id)?;
    if !c.query_row(
        "SELECT EXISTS(SELECT 1 FROM branches WHERE id=?1)",
        [&q.branch_id],
        |r| r.get::<_, bool>(0),
    )? {
        return Err(invalid("Branch not found."));
    }
    for id in [
        &q.product_package_id,
        &q.batch_id,
        &q.manufacturer_id,
        &q.category_id,
        &q.route_id,
    ]
    .into_iter()
    .flatten()
    {
        crate::db::catalog_validation::id(id)?;
    }
    for d in [&q.date_from, &q.date_to].into_iter().flatten() {
        date(c, d)?;
    }
    if matches!((&q.date_from,&q.date_to),(Some(a),Some(b)) if a>b) {
        return Err(invalid("Invalid date range."));
    }
    page(q.limit, q.offset)
}
fn json(r: &rusqlite::Row) -> rusqlite::Result<Value> {
    let s: String = r.get(0)?;
    serde_json::from_str(&s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}
pub fn overview(db: &AppDb, q: &Query) -> Result<Page, AppError> {
    let mut guard = db.lock()?;
    let c = guard.transaction()?;
    let (l, o) = validate(&c, q)?;
    // A branch package is relevant through any batch (including inactive/history),
    // explicit level, ledger entry, or document line (including drafts/cancelled).
    let base="WITH relevant AS (SELECT product_package_id id FROM inventory_batches WHERE branch_id=?1 UNION SELECT product_package_id FROM inventory_levels WHERE branch_id=?1 UNION SELECT product_package_id FROM inventory_movements WHERE branch_id=?1 UNION SELECT l.product_package_id FROM inventory_document_lines l JOIN inventory_documents d ON d.id=l.document_id WHERE d.branch_id=?1 OR d.destination_branch_id=?1), data AS (SELECT k.id,p.id product_id,coalesce(p.commercial_name_en,p.commercial_name_ar,'Unnamed product') name,p.commercial_name_ar,p.scientific_name,k.package_label,k.is_active package_active,p.is_active product_active,p.manufacturer_id,p.category_id,p.route_id,m.name manufacturer,l.reorder_level,l.minimum_stock,l.maximum_stock,coalesce((SELECT sum(quantity_delta) FROM inventory_movements WHERE branch_id=?1 AND product_package_id=k.id),0) quantity,(SELECT count(*) FROM inventory_batches WHERE branch_id=?1 AND product_package_id=k.id) batch_count,(SELECT min(b.expiry_date) FROM inventory_batches b WHERE b.branch_id=?1 AND b.product_package_id=k.id AND b.is_active=1 AND (SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements WHERE batch_id=b.id)>0) nearest_expiry FROM relevant r JOIN product_packages k ON k.id=r.id JOIN products p ON p.id=k.product_id LEFT JOIN manufacturers m ON m.id=p.manufacturer_id LEFT JOIN inventory_levels l ON l.branch_id=?1 AND l.product_package_id=k.id), classified AS (SELECT *,CASE WHEN quantity=0 THEN 'OUT_OF_STOCK' WHEN reorder_level IS NOT NULL AND quantity<=reorder_level THEN 'LOW_STOCK' ELSE 'IN_STOCK' END stock_state FROM data)";
    let filter="(?2 IS NULL OR id=?2) AND (?3='' OR instr(lower(name||' '||coalesce(commercial_name_ar,'')||' '||coalesce(scientific_name,'')||' '||coalesce(manufacturer,'')),lower(?3))>0 OR EXISTS(SELECT 1 FROM barcodes b WHERE b.product_package_id=classified.id AND instr(lower(b.barcode),lower(?3))>0)) AND (?4 IS NULL OR stock_state=?4) AND (?5 IS NULL OR manufacturer_id=?5) AND (?6 IS NULL OR category_id=?6) AND (?7 IS NULL OR route_id=?7)";
    let state = q.stock_state.as_ref().map(|s| match s {
        StockState::InStock => "IN_STOCK",
        StockState::LowStock => "LOW_STOCK",
        StockState::OutOfStock => "OUT_OF_STOCK",
    });
    let search = q.search.as_deref().unwrap_or("").trim();
    let total = c.query_row(
        &format!("{base} SELECT count(*) FROM classified WHERE {filter}"),
        params![
            q.branch_id,
            q.product_package_id,
            search,
            state,
            q.manufacturer_id,
            q.category_id,
            q.route_id
        ],
        |r| r.get(0),
    )?;
    let sql=format!("{base} SELECT json_object('productPackageId',id,'productId',product_id,'productName',name,'packageLabel',package_label,'branchId',?1,'quantity',quantity,'batchCount',batch_count,'nearestExpiry',nearest_expiry,'reorderLevel',reorder_level,'minimumStock',minimum_stock,'maximumStock',maximum_stock,'stockState',stock_state,'packageActive',package_active,'productActive',product_active) FROM classified WHERE {filter} ORDER BY name COLLATE NOCASE,id LIMIT ?8 OFFSET ?9");
    let mut s = c.prepare(&sql)?;
    let items = s
        .query_map(
            params![
                q.branch_id,
                q.product_package_id,
                search,
                state,
                q.manufacturer_id,
                q.category_id,
                q.route_id,
                l,
                o
            ],
            json,
        )?
        .collect::<Result<_, _>>()?;
    Ok(Page {
        items,
        total,
        business_date: alerts::business_date(&c, None)?,
    })
}
pub fn batch_list(db: &AppDb, q: &Query) -> Result<Page, AppError> {
    let mut guard = db.lock()?;
    let c = guard.transaction()?;
    let (l, o) = validate(&c, q)?;
    let filter =
        "b.branch_id=?1 AND (?2 IS NULL OR b.product_package_id=?2) AND (?3 IS NULL OR b.id=?3)";
    let total = c.query_row(
        &format!("SELECT count(*) FROM inventory_batches b WHERE {filter}"),
        params![q.branch_id, q.product_package_id, q.batch_id],
        |r| r.get(0),
    )?;
    let sql=format!("SELECT json_object('id',b.id,'branchId',b.branch_id,'productPackageId',b.product_package_id,'productId',p.id,'productName',coalesce(p.commercial_name_en,p.commercial_name_ar),'packageLabel',k.package_label,'lotNumber',b.lot_number,'expiryDate',b.expiry_date,'receivedAt',b.received_at,'costPriceMinor',b.cost_price_minor,'isActive',b.is_active,'quantity',coalesce((SELECT sum(quantity_delta) FROM inventory_movements WHERE batch_id=b.id),0)) FROM inventory_batches b JOIN product_packages k ON k.id=b.product_package_id JOIN products p ON p.id=k.product_id WHERE {filter} ORDER BY b.expiry_date IS NULL,b.expiry_date,b.created_at,b.id LIMIT ?4 OFFSET ?5");
    let mut s = c.prepare(&sql)?;
    let items = s
        .query_map(
            params![q.branch_id, q.product_package_id, q.batch_id, l, o],
            json,
        )?
        .collect::<Result<_, _>>()?;
    Ok(Page {
        items,
        total,
        business_date: alerts::business_date(&c, None)?,
    })
}
pub fn movement_list(db: &AppDb, q: &Query) -> Result<Page, AppError> {
    let mut guard = db.lock()?;
    let c = guard.transaction()?;
    let (l, o) = validate(&c, q)?;
    let base="FROM inventory_movements m JOIN product_packages k ON k.id=m.product_package_id JOIN products p ON p.id=k.product_id LEFT JOIN inventory_document_movements link ON link.movement_id=m.id LEFT JOIN inventory_documents d ON d.id=link.document_id LEFT JOIN stock_count_lines cl ON m.reference_type='stock_count_line' AND cl.id=m.reference_id LEFT JOIN stock_count_sessions cs ON cs.id=cl.session_id";
    let filter="m.branch_id=?1 AND (?2 IS NULL OR m.product_package_id=?2) AND (?3 IS NULL OR m.batch_id=?3) AND (?4 IS NULL OR m.movement_type=?4) AND (?5 IS NULL OR substr(m.created_at,1,10)>=?5) AND (?6 IS NULL OR substr(m.created_at,1,10)<=?6) AND (?7='' OR instr(lower(coalesce(d.document_number,cs.session_number,m.reference_id,'')),lower(?7))>0)";
    let kind = q.movement_type.as_ref().map(|m| m.as_str());
    let reference = q.document_reference.as_deref().unwrap_or("").trim();
    let total = c.query_row(
        &format!("SELECT count(*) {base} WHERE {filter}"),
        params![
            q.branch_id,
            q.product_package_id,
            q.batch_id,
            kind,
            q.date_from,
            q.date_to,
            reference
        ],
        |r| r.get(0),
    )?;
    let sql=format!("SELECT json_object('id',m.id,'branchId',m.branch_id,'productPackageId',m.product_package_id,'productName',coalesce(p.commercial_name_en,p.commercial_name_ar),'packageLabel',k.package_label,'batchId',m.batch_id,'movementType',m.movement_type,'quantityDelta',m.quantity_delta,'createdAt',m.created_at,'reason',m.reason,'createdBy',m.created_by,'referenceType',m.reference_type,'referenceId',m.reference_id,'documentId',d.id,'documentNumber',d.document_number,'countSessionId',cs.id,'countNumber',cs.session_number) {base} WHERE {filter} ORDER BY m.created_at DESC,m.id DESC LIMIT ?8 OFFSET ?9");
    let mut s = c.prepare(&sql)?;
    let items = s
        .query_map(
            params![
                q.branch_id,
                q.product_package_id,
                q.batch_id,
                kind,
                q.date_from,
                q.date_to,
                reference,
                l,
                o
            ],
            json,
        )?
        .collect::<Result<_, _>>()?;
    Ok(Page {
        items,
        total,
        business_date: alerts::business_date(&c, None)?,
    })
}
#[tauri::command]
pub fn inspect_inventory_overview(
    state: State<'_, DatabaseState>,
    query: Query,
) -> Result<Page, AppError> {
    overview(state.db()?, &query)
}
#[tauri::command]
pub fn inspect_inventory_batches(
    state: State<'_, DatabaseState>,
    query: Query,
) -> Result<Page, AppError> {
    batch_list(state.db()?, &query)
}
#[tauri::command]
pub fn inspect_inventory_movements(
    state: State<'_, DatabaseState>,
    query: Query,
) -> Result<Page, AppError> {
    movement_list(state.db()?, &query)
}
