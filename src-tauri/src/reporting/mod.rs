//! Read-only canonical reporting over operational records.
pub mod commands;
pub mod models;
use crate::{
    db::{
        catalog_validation::{id, invalid, MAX_SAFE_MINOR},
        connection::AppDb,
    },
    error::{AppError, ErrorCode},
    inventory,
};
use models::ReportFilter;
use rusqlite::{params, Connection};
use serde_json::{json, Value};

fn validate(c: &Connection, f: &ReportFilter) -> Result<(i64, i64, String), AppError> {
    for d in [&f.date_from, &f.date_to].into_iter().flatten() {
        inventory::date(c, d)?;
    }
    if matches!((&f.date_from,&f.date_to),(Some(a),Some(b)) if a>b) {
        return Err(invalid("Invalid report date range."));
    }
    for v in [
        &f.branch_id,
        &f.customer_id,
        &f.supplier_id,
        &f.product_id,
        &f.product_package_id,
    ]
    .into_iter()
    .flatten()
    {
        id(v)?;
    }
    if let Some(v) = &f.branch_id {
        if !c.query_row(
            "SELECT EXISTS(SELECT 1 FROM branches WHERE id=?1)",
            [v],
            |r| r.get::<_, bool>(0),
        )? {
            return Err(AppError::new(ErrorCode::NotFound, "Branch not found."));
        }
    }
    let search = f.search.as_deref().unwrap_or("").trim();
    if search.chars().count() > 200 || search.contains('\0') {
        return Err(invalid("Report search must be at most 200 characters."));
    }
    let (l, o) = inventory::page(f.limit, f.offset)?;
    Ok((l, o, search.into()))
}
fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    let s: String = r.get(0)?;
    serde_json::from_str(&s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}
fn page(items: Vec<Value>, total: i64, limit: i64, offset: i64) -> Value {
    json!({"items":items,"total":total,"limit":limit,"offset":offset})
}
fn safe(v: i128) -> Result<i64, AppError> {
    i64::try_from(v)
        .ok()
        .filter(|x| *x >= 0 && *x <= MAX_SAFE_MINOR)
        .ok_or_else(|| invalid("Report money total exceeds the safe integer range."))
}
const SALE_SCOPE:&str="(?1 IS NULL OR s.branch_id=?1) AND (?2 IS NULL OR substr(s.posted_at,1,10)>=?2) AND (?3 IS NULL OR substr(s.posted_at,1,10)<=?3)";

pub fn overview(db: &AppDb, f: &ReportFilter) -> Result<Value, AppError> {
    let c = db.lock()?;
    validate(&c, f)?;
    let sales:(i64,i64,i64,i64)=c.query_row(&format!("SELECT count(*),coalesce(sum(total_minor),0),coalesce(sum(discount_minor),0),coalesce(sum(tax_minor),0) FROM sales s WHERE s.status='POSTED' AND {SALE_SCOPE}"),params![f.branch_id,f.date_from,f.date_to],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?;
    let refunds:i64=c.query_row("SELECT coalesce(sum(r.amount_minor),0) FROM sale_refunds r JOIN sales s ON s.id=r.sale_id WHERE s.status='POSTED' AND (?1 IS NULL OR s.branch_id=?1) AND (?2 IS NULL OR substr(s.posted_at,1,10)>=?2) AND (?3 IS NULL OR substr(s.posted_at,1,10)<=?3)",params![f.branch_id,f.date_from,f.date_to],|r|r.get(0))?;
    let units:i64=c.query_row("SELECT coalesce(sum(i.quantity-coalesce((SELECT sum(ri.quantity) FROM sale_return_items ri JOIN sale_returns sr ON sr.id=ri.return_id WHERE ri.sale_item_id=i.id AND sr.status='POSTED'),0)),0) FROM sale_items i JOIN sales s ON s.id=i.sale_id WHERE s.status='POSTED' AND (?1 IS NULL OR s.branch_id=?1) AND (?2 IS NULL OR substr(s.posted_at,1,10)>=?2) AND (?3 IS NULL OR substr(s.posted_at,1,10)<=?3)",params![f.branch_id,f.date_from,f.date_to],|r|r.get(0))?;
    let stock:i64=c.query_row("SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements WHERE (?1 IS NULL OR branch_id=?1)",params![f.branch_id],|r|r.get(0))?;
    let low:i64=c.query_row("SELECT count(*) FROM inventory_levels l WHERE (?1 IS NULL OR l.branch_id=?1) AND (SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements WHERE branch_id=l.branch_id AND product_package_id=l.product_package_id)<=l.reorder_level",params![f.branch_id],|r|r.get(0))?;
    let po:i64=c.query_row("SELECT count(*) FROM purchase_orders WHERE (?1 IS NULL OR substr(created_at,1,10)>=?1) AND (?2 IS NULL OR substr(created_at,1,10)<=?2)",params![f.date_from,f.date_to],|r|r.get(0))?;
    let received:i64=c.query_row("SELECT coalesce(sum(total_quantity),0) FROM purchase_receipts WHERE status='POSTED' AND (?1 IS NULL OR branch_id=?1) AND (?2 IS NULL OR substr(posted_at,1,10)>=?2) AND (?3 IS NULL OR substr(posted_at,1,10)<=?3)",params![f.branch_id,f.date_from,f.date_to],|r|r.get(0))?;
    let returned:i64=c.query_row("SELECT coalesce(sum(total_quantity),0) FROM purchase_returns WHERE status='POSTED' AND (?1 IS NULL OR branch_id=?1) AND (?2 IS NULL OR substr(posted_at,1,10)>=?2) AND (?3 IS NULL OR substr(posted_at,1,10)<=?3)",params![f.branch_id,f.date_from,f.date_to],|r|r.get(0))?;
    Ok(
        json!({"postedSalesCount":sales.0,"grossSalesMinor":sales.1,"discountMinor":sales.2,"taxMinor":sales.3,"refundedMinor":refunds,"netSalesMinor":sales.1-refunds,"netUnitsSold":units,"currentStockUnits":stock,"lowStockPackageCount":low,"purchaseOrderCount":po,"receivedQuantity":received,"purchaseReturnQuantity":returned}),
    )
}

pub fn sales_report(db: &AppDb, f: &ReportFilter) -> Result<Value, AppError> {
    let c = db.lock()?;
    let (l, o, s) = validate(&c, f)?;
    if f.status
        .as_ref()
        .is_some_and(|x| !["POSTED", "VOIDED"].contains(&x.as_str()))
    {
        return Err(invalid("Sales status must be POSTED or VOIDED."));
    }
    let filter=format!("s.status IN ('POSTED','VOIDED') AND {SALE_SCOPE} AND (?4 IS NULL OR s.status=?4) AND (?5 IS NULL OR s.customer_id=?5) AND (?6='' OR instr(lower(coalesce(s.sale_number,'')||' '||coalesce(s.customer_name,'')),lower(?6))>0)");
    let total = c.query_row(
        &format!("SELECT count(*) FROM sales s WHERE {filter}"),
        params![
            f.branch_id,
            f.date_from,
            f.date_to,
            f.status,
            f.customer_id,
            s
        ],
        |r| r.get(0),
    )?;
    let summary:(i64,i64,i64,i64,i64,i64,i64)=c.query_row(&format!("SELECT coalesce(sum(CASE WHEN s.status='POSTED' THEN 1 ELSE 0 END),0),coalesce(sum(CASE WHEN s.status='POSTED' THEN s.total_minor ELSE 0 END),0),coalesce(sum(CASE WHEN s.status='POSTED' THEN s.discount_minor ELSE 0 END),0),coalesce(sum(CASE WHEN s.status='POSTED' THEN s.tax_minor ELSE 0 END),0),coalesce(sum(CASE WHEN s.status='POSTED' THEN (SELECT coalesce(sum(amount_minor),0) FROM sale_refunds WHERE sale_id=s.id) ELSE 0 END),0),coalesce(sum(CASE WHEN s.status='POSTED' THEN (SELECT coalesce(sum(quantity),0) FROM sale_items WHERE sale_id=s.id) ELSE 0 END),0),coalesce(sum(CASE WHEN s.status='POSTED' THEN (SELECT coalesce(sum(ri.quantity),0) FROM sale_return_items ri JOIN sale_items si ON si.id=ri.sale_item_id JOIN sale_returns sr ON sr.id=ri.return_id WHERE si.sale_id=s.id AND sr.status='POSTED') ELSE 0 END),0) FROM sales s WHERE {filter}"),params![f.branch_id,f.date_from,f.date_to,f.status,f.customer_id,s],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?)))?;
    let mut q=c.prepare(&format!("SELECT json_object('id',s.id,'saleNumber',s.sale_number,'branchId',s.branch_id,'customerId',s.customer_id,'customerName',s.customer_name,'status',s.status,'postedAt',s.posted_at,'grossMinor',s.total_minor,'discountMinor',s.discount_minor,'taxMinor',s.tax_minor,'refundedMinor',(SELECT coalesce(sum(amount_minor),0) FROM sale_refunds WHERE sale_id=s.id),'netMinor',CASE WHEN s.status='VOIDED' THEN 0 ELSE s.total_minor-(SELECT coalesce(sum(amount_minor),0) FROM sale_refunds WHERE sale_id=s.id) END,'units',(SELECT coalesce(sum(quantity),0) FROM sale_items WHERE sale_id=s.id),'returnedUnits',(SELECT coalesce(sum(ri.quantity),0) FROM sale_return_items ri JOIN sale_items i ON i.id=ri.sale_item_id JOIN sale_returns sr ON sr.id=ri.return_id WHERE i.sale_id=s.id AND sr.status='POSTED')) FROM sales s WHERE {filter} ORDER BY s.posted_at DESC,s.id DESC LIMIT ?7 OFFSET ?8"))?;
    let items = q
        .query_map(
            params![
                f.branch_id,
                f.date_from,
                f.date_to,
                f.status,
                f.customer_id,
                s,
                l,
                o
            ],
            row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    let mut result = page(items, total, l, o);
    result["summary"] = json!({"postedSalesCount":summary.0,"grossSalesMinor":summary.1,"discountMinor":summary.2,"taxMinor":summary.3,"refundedMinor":summary.4,"netSalesMinor":summary.1-summary.4,"unitsSold":summary.5,"returnedQuantity":summary.6,"netUnitsSold":summary.5-summary.6});
    Ok(result)
}

pub fn sales_by_product(db: &AppDb, f: &ReportFilter) -> Result<Value, AppError> {
    let c = db.lock()?;
    let (l, o, s) = validate(&c, f)?;
    let filter=format!("s.status='POSTED' AND {SALE_SCOPE} AND (?4 IS NULL OR i.product_id=?4) AND (?5 IS NULL OR i.product_package_id=?5) AND (?6='' OR instr(lower(i.product_name_snapshot||' '||i.package_label_snapshot),lower(?6))>0)");
    let total=c.query_row(&format!("SELECT count(*) FROM (SELECT i.product_package_id FROM sale_items i JOIN sales s ON s.id=i.sale_id WHERE {filter} GROUP BY i.product_package_id)"),params![f.branch_id,f.date_from,f.date_to,f.product_id,f.product_package_id,s],|r|r.get(0))?;
    let mut q=c.prepare(&format!("SELECT json_object('productId',min(i.product_id),'productPackageId',i.product_package_id,'productName',min(i.product_name_snapshot),'packageLabel',min(i.package_label_snapshot),'quantitySold',sum(i.quantity),'quantityReturned',sum(coalesce((SELECT sum(ri.quantity) FROM sale_return_items ri JOIN sale_returns sr ON sr.id=ri.return_id WHERE ri.sale_item_id=i.id AND sr.status='POSTED'),0)),'netQuantity',sum(i.quantity-coalesce((SELECT sum(ri.quantity) FROM sale_return_items ri JOIN sale_returns sr ON sr.id=ri.return_id WHERE ri.sale_item_id=i.id AND sr.status='POSTED'),0)),'grossMinor',sum(i.line_total_minor),'refundedMinor',sum(coalesce((SELECT sum(ri.refund_minor) FROM sale_return_items ri JOIN sale_returns sr ON sr.id=ri.return_id WHERE ri.sale_item_id=i.id AND sr.status='POSTED'),0)),'netMinor',sum(i.line_total_minor-coalesce((SELECT sum(ri.refund_minor) FROM sale_return_items ri JOIN sale_returns sr ON sr.id=ri.return_id WHERE ri.sale_item_id=i.id AND sr.status='POSTED'),0))) FROM sale_items i JOIN sales s ON s.id=i.sale_id WHERE {filter} GROUP BY i.product_package_id ORDER BY i.product_package_id LIMIT ?7 OFFSET ?8"))?;
    let items = q
        .query_map(
            params![
                f.branch_id,
                f.date_from,
                f.date_to,
                f.product_id,
                f.product_package_id,
                s,
                l,
                o
            ],
            row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, total, l, o))
}

pub fn sales_by_customer(db: &AppDb, f: &ReportFilter) -> Result<Value, AppError> {
    let c = db.lock()?;
    let (l, o, s) = validate(&c, f)?;
    let filter=format!("s.status='POSTED' AND s.customer_id IS NOT NULL AND {SALE_SCOPE} AND (?4 IS NULL OR s.customer_id=?4) AND (?5='' OR instr(lower(c.code||' '||c.display_name),lower(?5))>0)");
    let total=c.query_row(&format!("SELECT count(*) FROM (SELECT s.customer_id FROM sales s JOIN customers c ON c.id=s.customer_id WHERE {filter} GROUP BY s.customer_id)"),params![f.branch_id,f.date_from,f.date_to,f.customer_id,s],|r|r.get(0))?;
    let mut q=c.prepare(&format!("SELECT json_object('customerId',s.customer_id,'customerCode',c.code,'customerName',c.display_name,'saleCount',count(*),'lastSaleAt',max(s.posted_at),'grossMinor',sum(s.total_minor),'refundedMinor',sum((SELECT coalesce(sum(amount_minor),0) FROM sale_refunds WHERE sale_id=s.id)),'netMinor',sum(s.total_minor-(SELECT coalesce(sum(amount_minor),0) FROM sale_refunds WHERE sale_id=s.id))) FROM sales s JOIN customers c ON c.id=s.customer_id WHERE {filter} GROUP BY s.customer_id,c.code,c.display_name ORDER BY s.customer_id LIMIT ?6 OFFSET ?7"))?;
    let items = q
        .query_map(
            params![f.branch_id, f.date_from, f.date_to, f.customer_id, s, l, o],
            row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, total, l, o))
}

pub fn inventory_stock(db: &AppDb, f: &ReportFilter) -> Result<Value, AppError> {
    let c = db.lock()?;
    let (l, o, s) = validate(&c, f)?;
    let filter="(?1 IS NULL OR m.branch_id=?1) AND (?2 IS NULL OR p.id=?2) AND (?3 IS NULL OR pk.id=?3) AND (?4='' OR instr(lower(coalesce(p.commercial_name_en,'')||' '||coalesce(p.commercial_name_ar,'')||' '||pk.package_label),lower(?4))>0)";
    let total=c.query_row(&format!("SELECT count(*) FROM (SELECT m.branch_id,m.product_package_id FROM inventory_movements m JOIN product_packages pk ON pk.id=m.product_package_id JOIN products p ON p.id=pk.product_id WHERE {filter} GROUP BY m.branch_id,m.product_package_id)"),params![f.branch_id,f.product_id,f.product_package_id,s],|r|r.get(0))?;
    let mut q=c.prepare(&format!("SELECT m.branch_id,b.name,p.id,pk.id,coalesce(p.commercial_name_en,p.commercial_name_ar,'Unnamed product'),pk.package_label,sum(m.quantity_delta),coalesce(l.reorder_level,0) FROM inventory_movements m JOIN branches b ON b.id=m.branch_id JOIN product_packages pk ON pk.id=m.product_package_id JOIN products p ON p.id=pk.product_id LEFT JOIN inventory_levels l ON l.branch_id=m.branch_id AND l.product_package_id=m.product_package_id WHERE {filter} GROUP BY m.branch_id,m.product_package_id ORDER BY coalesce(p.commercial_name_en,p.commercial_name_ar,'') COLLATE NOCASE,pk.package_label COLLATE NOCASE,m.branch_id LIMIT ?5 OFFSET ?6"))?;
    let raw = q
        .query_map(
            params![f.branch_id, f.product_id, f.product_package_id, s, l, o],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, i64>(6)?,
                    r.get::<_, i64>(7)?,
                ))
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    let mut items = Vec::with_capacity(raw.len());
    for (bid, bname, pid, pkid, pname, label, qty, reorder) in raw {
        let mut stmt=c.prepare("SELECT sum(m.quantity_delta),ib.cost_price_minor FROM inventory_batches ib JOIN inventory_movements m ON m.batch_id=ib.id WHERE ib.branch_id=?1 AND ib.product_package_id=?2 GROUP BY ib.id")?;
        let batches = stmt
            .query_map(params![bid, pkid], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, Option<i64>>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let mut value = 0i128;
        let mut unvalued = 0i64;
        for (q, cost) in batches {
            if q > 0 {
                if let Some(cost) = cost {
                    value += i128::from(q) * i128::from(cost)
                } else {
                    unvalued += q;
                }
            }
        }
        let stock_state = if qty <= 0 {
            "OUT_OF_STOCK"
        } else if qty <= reorder {
            "LOW_STOCK"
        } else {
            "IN_STOCK"
        };
        items.push(json!({"branchId":bid,"branchName":bname,"productId":pid,"productPackageId":pkid,"productName":pname,"packageLabel":label,"quantity":qty,"reorderLevel":reorder,"stockState":stock_state,"acquisitionCostValuationMinor":safe(value)?,"unvaluedQuantity":unvalued}));
    }
    Ok(page(items, total, l, o))
}

pub fn inventory_movements(db: &AppDb, f: &ReportFilter) -> Result<Value, AppError> {
    let c = db.lock()?;
    let (l, o, s) = validate(&c, f)?;
    let filter="(?1 IS NULL OR m.branch_id=?1) AND (?2 IS NULL OR substr(m.created_at,1,10)>=?2) AND (?3 IS NULL OR substr(m.created_at,1,10)<=?3) AND (?4 IS NULL OR m.movement_type=?4) AND (?5 IS NULL OR p.id=?5) AND (?6 IS NULL OR pk.id=?6) AND (?7='' OR instr(lower(coalesce(p.commercial_name_en,p.commercial_name_ar,'')||' '||pk.package_label||' '||coalesce(ib.lot_number,'')||' '||coalesce(m.reference_id,'')),lower(?7))>0)";
    let total=c.query_row(&format!("SELECT count(*) FROM inventory_movements m JOIN product_packages pk ON pk.id=m.product_package_id JOIN products p ON p.id=pk.product_id JOIN inventory_batches ib ON ib.id=m.batch_id WHERE {filter}"),params![f.branch_id,f.date_from,f.date_to,f.movement_type,f.product_id,f.product_package_id,s],|r|r.get(0))?;
    let mut q=c.prepare(&format!("SELECT json_object('id',m.id,'branchId',m.branch_id,'branchName',b.name,'productId',p.id,'productPackageId',pk.id,'productName',coalesce(p.commercial_name_en,p.commercial_name_ar,'Unnamed product'),'packageLabel',pk.package_label,'batchId',m.batch_id,'lotNumber',ib.lot_number,'movementType',m.movement_type,'quantityDelta',m.quantity_delta,'referenceType',m.reference_type,'referenceId',m.reference_id,'reason',m.reason,'createdAt',m.created_at) FROM inventory_movements m JOIN branches b ON b.id=m.branch_id JOIN product_packages pk ON pk.id=m.product_package_id JOIN products p ON p.id=pk.product_id JOIN inventory_batches ib ON ib.id=m.batch_id WHERE {filter} ORDER BY m.created_at DESC,m.id DESC LIMIT ?8 OFFSET ?9"))?;
    let items = q
        .query_map(
            params![
                f.branch_id,
                f.date_from,
                f.date_to,
                f.movement_type,
                f.product_id,
                f.product_package_id,
                s,
                l,
                o
            ],
            row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, total, l, o))
}

pub fn expiry_report(db: &AppDb, f: &ReportFilter) -> Result<Value, AppError> {
    let c = db.lock()?;
    let (l, o, s) = validate(&c, f)?;
    let days = f.expiry_days.unwrap_or(90);
    if !(0..=36500).contains(&days) {
        return Err(invalid("Expiry days must be between 0 and 36500."));
    }
    let filter="(?1 IS NULL OR ib.branch_id=?1) AND (?2 IS NULL OR p.id=?2) AND (?3 IS NULL OR pk.id=?3) AND (?4='' OR instr(lower(coalesce(p.commercial_name_en,p.commercial_name_ar,'')||' '||pk.package_label||' '||coalesce(ib.lot_number,'')),lower(?4))>0)";
    let base=format!("WITH balances AS (SELECT ib.id,ib.branch_id,ib.product_package_id,ib.lot_number,ib.expiry_date,sum(m.quantity_delta) quantity FROM inventory_batches ib JOIN inventory_movements m ON m.batch_id=ib.id GROUP BY ib.id HAVING sum(m.quantity_delta)>0) SELECT {{select}} FROM balances ib JOIN branches b ON b.id=ib.branch_id JOIN product_packages pk ON pk.id=ib.product_package_id JOIN products p ON p.id=pk.product_id WHERE {filter}");
    let total = c.query_row(
        &base.replace("{select}", "count(*)"),
        params![f.branch_id, f.product_id, f.product_package_id, s],
        |r| r.get(0),
    )?;
    let sql=format!("{} ORDER BY CASE WHEN ib.expiry_date IS NULL THEN 1 ELSE 0 END,ib.expiry_date,ib.id LIMIT ?6 OFFSET ?7",base.replace("{select}","json_object('batchId',ib.id,'branchId',ib.branch_id,'branchName',b.name,'productId',p.id,'productPackageId',pk.id,'productName',coalesce(p.commercial_name_en,p.commercial_name_ar,'Unnamed product'),'packageLabel',pk.package_label,'lotNumber',ib.lot_number,'expiryDate',ib.expiry_date,'quantity',ib.quantity,'expiryState',CASE WHEN ib.expiry_date IS NULL THEN 'NO_EXPIRY' WHEN ib.expiry_date<date('now') THEN 'EXPIRED' WHEN ib.expiry_date<=date('now','+'||?5||' days') THEN 'EXPIRING' ELSE 'VALID' END,'daysToExpiry',CASE WHEN ib.expiry_date IS NULL THEN NULL ELSE cast(julianday(ib.expiry_date)-julianday(date('now')) AS integer) END)"));
    let mut q = c.prepare(&sql)?;
    let items = q
        .query_map(
            params![
                f.branch_id,
                f.product_id,
                f.product_package_id,
                s,
                days,
                l,
                o
            ],
            row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, total, l, o))
}

pub fn low_stock_report(db: &AppDb, f: &ReportFilter) -> Result<Value, AppError> {
    let c = db.lock()?;
    let (l, o, s) = validate(&c, f)?;
    let filter="(?1 IS NULL OR l.branch_id=?1) AND (?2 IS NULL OR p.id=?2) AND (?3 IS NULL OR pk.id=?3) AND (?4='' OR instr(lower(coalesce(p.commercial_name_en,p.commercial_name_ar,'')||' '||pk.package_label),lower(?4))>0) AND (SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements WHERE branch_id=l.branch_id AND product_package_id=l.product_package_id)<=l.reorder_level";
    let total=c.query_row(&format!("SELECT count(*) FROM inventory_levels l JOIN product_packages pk ON pk.id=l.product_package_id JOIN products p ON p.id=pk.product_id WHERE {filter}"),params![f.branch_id,f.product_id,f.product_package_id,s],|r|r.get(0))?;
    let mut q=c.prepare(&format!("SELECT json_object('branchId',l.branch_id,'branchName',b.name,'productId',p.id,'productPackageId',pk.id,'productName',coalesce(p.commercial_name_en,p.commercial_name_ar,'Unnamed product'),'packageLabel',pk.package_label,'quantity',(SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements WHERE branch_id=l.branch_id AND product_package_id=l.product_package_id),'reorderLevel',l.reorder_level,'minimumStock',l.minimum_stock,'maximumStock',l.maximum_stock,'stockState',CASE WHEN (SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements WHERE branch_id=l.branch_id AND product_package_id=l.product_package_id)<=0 THEN 'OUT_OF_STOCK' ELSE 'LOW_STOCK' END) FROM inventory_levels l JOIN branches b ON b.id=l.branch_id JOIN product_packages pk ON pk.id=l.product_package_id JOIN products p ON p.id=pk.product_id WHERE {filter} ORDER BY l.branch_id,pk.id LIMIT ?5 OFFSET ?6"))?;
    let items = q
        .query_map(
            params![f.branch_id, f.product_id, f.product_package_id, s, l, o],
            row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, total, l, o))
}

pub fn purchasing_report(db: &AppDb, f: &ReportFilter) -> Result<Value, AppError> {
    let c = db.lock()?;
    let (l, o, s) = validate(&c, f)?;
    let allowed = [
        "DRAFT",
        "ORDERED",
        "PARTIALLY_RECEIVED",
        "RECEIVED",
        "CANCELLED",
    ];
    if f.status
        .as_ref()
        .is_some_and(|x| !allowed.contains(&x.as_str()))
    {
        return Err(invalid("Invalid purchase status."));
    }
    let filter="(?1 IS NULL OR substr(po.created_at,1,10)>=?1) AND (?2 IS NULL OR substr(po.created_at,1,10)<=?2) AND (?3 IS NULL OR po.status=?3) AND (?4 IS NULL OR po.supplier_id=?4) AND (?5='' OR instr(lower(po.purchase_number||' '||coalesce(po.supplier_name_snapshot,su.display_name)),lower(?5))>0)";
    let total=c.query_row(&format!("SELECT count(*) FROM purchase_orders po JOIN suppliers su ON su.id=po.supplier_id WHERE {filter}"),params![f.date_from,f.date_to,f.status,f.supplier_id,s],|r|r.get(0))?;
    let mut q=c.prepare(&format!("SELECT json_object('id',po.id,'purchaseNumber',po.purchase_number,'supplierId',po.supplier_id,'supplierName',coalesce(po.supplier_name_snapshot,su.display_name),'status',po.status,'createdAt',po.created_at,'orderedAt',po.ordered_at,'orderedQuantity',(SELECT coalesce(sum(ordered_quantity),0) FROM purchase_order_lines WHERE purchase_order_id=po.id),'orderedValueMinor',po.total_minor,'receivedQuantity',(SELECT coalesce(sum(total_quantity),0) FROM purchase_receipts WHERE purchase_order_id=po.id AND status='POSTED'),'returnedQuantity',(SELECT coalesce(sum(total_quantity),0) FROM purchase_returns WHERE purchase_order_id=po.id AND status='POSTED'),'receiptCount',(SELECT count(*) FROM purchase_receipts WHERE purchase_order_id=po.id AND status='POSTED'),'returnCount',(SELECT count(*) FROM purchase_returns WHERE purchase_order_id=po.id AND status='POSTED')) FROM purchase_orders po JOIN suppliers su ON su.id=po.supplier_id WHERE {filter} ORDER BY po.created_at DESC,po.id DESC LIMIT ?6 OFFSET ?7"))?;
    let items = q
        .query_map(
            params![f.date_from, f.date_to, f.status, f.supplier_id, s, l, o],
            row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, total, l, o))
}

pub fn purchase_receipt_report(db: &AppDb, f: &ReportFilter) -> Result<Value, AppError> {
    let c = db.lock()?;
    let (l, o, s) = validate(&c, f)?;
    let filter="r.status='POSTED' AND (?1 IS NULL OR r.branch_id=?1) AND (?2 IS NULL OR substr(r.posted_at,1,10)>=?2) AND (?3 IS NULL OR substr(r.posted_at,1,10)<=?3) AND (?4 IS NULL OR po.supplier_id=?4) AND (?5='' OR instr(lower(r.receipt_number||' '||po.purchase_number||' '||coalesce(po.supplier_name_snapshot,su.display_name)),lower(?5))>0)";
    let total=c.query_row(&format!("SELECT count(*) FROM purchase_receipts r JOIN purchase_orders po ON po.id=r.purchase_order_id JOIN suppliers su ON su.id=po.supplier_id WHERE {filter}"),params![f.branch_id,f.date_from,f.date_to,f.supplier_id,s],|r|r.get(0))?;
    let mut q=c.prepare(&format!("SELECT json_object('id',r.id,'documentNumber',r.receipt_number,'purchaseOrderId',po.id,'purchaseNumber',po.purchase_number,'branchId',r.branch_id,'supplierId',po.supplier_id,'supplierName',coalesce(po.supplier_name_snapshot,su.display_name),'quantity',r.total_quantity,'acquisitionValueMinor',(SELECT coalesce(sum(quantity*acquisition_cost_minor),0) FROM purchase_receipt_lines WHERE receipt_id=r.id),'postedAt',r.posted_at) FROM purchase_receipts r JOIN purchase_orders po ON po.id=r.purchase_order_id JOIN suppliers su ON su.id=po.supplier_id WHERE {filter} ORDER BY r.posted_at DESC,r.id DESC LIMIT ?6 OFFSET ?7"))?;
    let items = q
        .query_map(
            params![f.branch_id, f.date_from, f.date_to, f.supplier_id, s, l, o],
            row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, total, l, o))
}

pub fn purchase_return_report(db: &AppDb, f: &ReportFilter) -> Result<Value, AppError> {
    let c = db.lock()?;
    let (l, o, s) = validate(&c, f)?;
    let filter="r.status='POSTED' AND (?1 IS NULL OR r.branch_id=?1) AND (?2 IS NULL OR substr(r.posted_at,1,10)>=?2) AND (?3 IS NULL OR substr(r.posted_at,1,10)<=?3) AND (?4 IS NULL OR r.supplier_id=?4) AND (?5='' OR instr(lower(r.return_number||' '||po.purchase_number||' '||su.display_name),lower(?5))>0)";
    let total=c.query_row(&format!("SELECT count(*) FROM purchase_returns r JOIN purchase_orders po ON po.id=r.purchase_order_id JOIN suppliers su ON su.id=r.supplier_id WHERE {filter}"),params![f.branch_id,f.date_from,f.date_to,f.supplier_id,s],|r|r.get(0))?;
    let mut q=c.prepare(&format!("SELECT json_object('id',r.id,'documentNumber',r.return_number,'purchaseOrderId',po.id,'purchaseNumber',po.purchase_number,'receiptId',r.receipt_id,'branchId',r.branch_id,'supplierId',r.supplier_id,'supplierName',su.display_name,'quantity',r.total_quantity,'acquisitionValueMinor',(SELECT coalesce(sum(quantity*acquisition_cost_minor),0) FROM purchase_return_lines WHERE return_id=r.id),'reason',r.reason,'postedAt',r.posted_at) FROM purchase_returns r JOIN purchase_orders po ON po.id=r.purchase_order_id JOIN suppliers su ON su.id=r.supplier_id WHERE {filter} ORDER BY r.posted_at DESC,r.id DESC LIMIT ?6 OFFSET ?7"))?;
    let items = q
        .query_map(
            params![f.branch_id, f.date_from, f.date_to, f.supplier_id, s, l, o],
            row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, total, l, o))
}

pub fn supplier_analytics(db: &AppDb, f: &ReportFilter) -> Result<Value, AppError> {
    let c = db.lock()?;
    let (l, o, s) = validate(&c, f)?;
    let filter="(?1 IS NULL OR su.id=?1) AND (?2='' OR instr(lower(su.code||' '||su.display_name),lower(?2))>0)";
    let total = c.query_row(
        &format!("SELECT count(*) FROM suppliers su WHERE {filter}"),
        params![f.supplier_id, s],
        |r| r.get(0),
    )?;
    let mut q=c.prepare(&format!("SELECT json_object('supplierId',su.id,'supplierCode',su.code,'supplierName',su.display_name,'isActive',json(CASE su.is_active WHEN 1 THEN 'true' ELSE 'false' END),'orderCount',(SELECT count(*) FROM purchase_orders WHERE supplier_id=su.id),'orderedValueMinor',(SELECT coalesce(sum(total_minor),0) FROM purchase_orders WHERE supplier_id=su.id AND status!='CANCELLED'),'receivedQuantity',(SELECT coalesce(sum(r.total_quantity),0) FROM purchase_receipts r JOIN purchase_orders po ON po.id=r.purchase_order_id WHERE po.supplier_id=su.id AND r.status='POSTED'),'receivedValueMinor',(SELECT coalesce(sum(rl.quantity*rl.acquisition_cost_minor),0) FROM purchase_receipt_lines rl JOIN purchase_receipts r ON r.id=rl.receipt_id JOIN purchase_orders po ON po.id=r.purchase_order_id WHERE po.supplier_id=su.id AND r.status='POSTED'),'returnedQuantity',(SELECT coalesce(sum(total_quantity),0) FROM purchase_returns WHERE supplier_id=su.id AND status='POSTED'),'returnedValueMinor',(SELECT coalesce(sum(rl.quantity*rl.acquisition_cost_minor),0) FROM purchase_return_lines rl JOIN purchase_returns r ON r.id=rl.return_id WHERE r.supplier_id=su.id AND r.status='POSTED'),'lastPurchaseAt',(SELECT max(created_at) FROM purchase_orders WHERE supplier_id=su.id)) FROM suppliers su WHERE {filter} ORDER BY su.id LIMIT ?3 OFFSET ?4"))?;
    let items = q
        .query_map(params![f.supplier_id, s, l, o], row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, total, l, o))
}
