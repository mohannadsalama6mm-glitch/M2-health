//! Canonical sales. All stock-changing operations own one IMMEDIATE transaction.
pub mod commands;
pub mod models;
mod returns;
use crate::{
    db::{
        catalog_validation::{id, invalid, optional, required, MAX_SAFE_MINOR},
        connection::AppDb,
    },
    error::AppError,
    inventory::{
        self, expiry,
        models::{MovementType, StockOperation},
        movements,
    },
};
use models::*;
pub use returns::{create_return, post_return, preview_return, void};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde_json::{json, Value};
use std::collections::HashSet;
fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn money(v: i128) -> Result<i64, AppError> {
    if !(0..=MAX_SAFE_MINOR as i128).contains(&v) {
        Err(invalid("Money exceeds the safe integer range."))
    } else {
        Ok(v as i64)
    }
}
fn text<T: serde::Serialize>(v: &T) -> Result<String, AppError> {
    serde_json::to_string(v).map_err(|_| invalid("Invalid sales payload."))
}
fn branch(c: &Connection, b: &str) -> Result<(), AppError> {
    id(b)?;
    if !c.query_row(
        "SELECT EXISTS(SELECT 1 FROM branches WHERE id=?1 AND is_active=1)",
        [b],
        |r| r.get::<_, bool>(0),
    )? {
        return Err(invalid("Select an active branch."));
    }
    Ok(())
}
fn decode<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, AppError> {
    serde_json::from_str(s).map_err(|_| invalid("Stored sales data is invalid."))
}
fn jsonrow(r: &rusqlite::Row) -> rusqlite::Result<Value> {
    let s: String = r.get(0)?;
    serde_json::from_str(&s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}
fn objects(c: &Connection, sql: &str, key: &str) -> Result<Vec<Value>, AppError> {
    let mut s = c.prepare(sql)?;
    let result = s
        .query_map([key], jsonrow)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(result)
}
fn stored(c: &Connection, sale: &str) -> Result<(String, i64, Cart), AppError> {
    id(sale)?;
    let r: (String, i64, String) = c.query_row(
        "SELECT status,revision,draft_json FROM sales WHERE id=?1",
        [sale],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    Ok((r.0, r.1, decode(&r.2)?))
}
fn editable(status: &str, revision: i64, expected: i64) -> Result<(), AppError> {
    if !["DRAFT", "HELD"].contains(&status) {
        return Err(invalid("Only draft or held sales may be edited or posted."));
    }
    if revision != expected {
        return Err(invalid("Sale changed. Reload before continuing."));
    }
    Ok(())
}
fn eligible(
    c: &Connection,
    b: &str,
    p: &str,
) -> Result<Vec<inventory::models::ExpiryItem>, AppError> {
    let date: String = c.query_row("SELECT date('now')", [], |r| r.get(0))?;
    let mut out = Vec::new();
    let mut offset = 0;
    loop {
        let mut rows = expiry::fefo_on(c, b, p, &date, 200, offset)?;
        let count = rows.len();
        out.append(&mut rows);
        if count < 200 {
            break;
        }
        offset += 200;
    }
    Ok(out)
}
pub fn calculate(c: &Connection, cart: &Cart, allocate: bool) -> Result<Preview, AppError> {
    branch(c, &cart.branch_id)?;
    optional(&cart.customer_name)?;
    optional(&cart.notes)?;
    money(cart.discount_minor as i128)?;
    money(cart.tax_minor as i128)?;
    if cart.lines.len() > 200 {
        return Err(invalid("A cart supports at most 200 packages."));
    }
    let mut seen = HashSet::new();
    let mut lines = Vec::new();
    let (mut subtotal, mut line_discounts) = (0i128, 0i128);
    for l in &cart.lines {
        if !seen.insert(&l.product_package_id) {
            return Err(invalid("Merge duplicate package lines before saving."));
        }
        inventory::scope(c, &cart.branch_id, &l.product_package_id, true)?;
        inventory::quantity(l.quantity)?;
        let (product,name,label,price):(String,String,String,Option<i64>)=c.query_row("SELECT p.id,coalesce(p.commercial_name_en,p.commercial_name_ar),k.package_label,(SELECT selling_price_minor FROM product_price_history WHERE product_package_id=k.id AND effective_to IS NULL) FROM product_packages k JOIN products p ON p.id=k.product_id WHERE k.id=?1",[&l.product_package_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?;
        let price = l
            .unit_price_minor
            .or(price)
            .ok_or_else(|| invalid("Package has no selling price."))?;
        money(price as i128)?;
        money(l.discount_minor as i128)?;
        let gross = money(price as i128 * l.quantity as i128)?;
        if l.discount_minor > gross {
            return Err(invalid("Line discount exceeds line subtotal."));
        }
        subtotal += gross as i128;
        line_discounts += l.discount_minor as i128;
        let mut allocations = Vec::new();
        if allocate {
            let mut remaining = l.quantity;
            for row in eligible(c, &cart.branch_id, &l.product_package_id)? {
                let take = remaining.min(row.quantity);
                allocations.push(Allocation {
                    batch_id: row.batch.id,
                    quantity: take,
                    unit_cost_minor: row.batch.cost_price_minor,
                    expiry_date: row.batch.expiry_date,
                    lot_number: row.batch.lot_number,
                });
                remaining -= take;
                if remaining == 0 {
                    break;
                }
            }
            if remaining > 0 {
                return Err(invalid("Insufficient non-expired stock. Refresh the cart."));
            }
        }
        lines.push(PricedLine {
            product_id: product,
            product_package_id: l.product_package_id.clone(),
            product_name_snapshot: name,
            package_label_snapshot: label,
            quantity: l.quantity,
            unit_price_minor: price,
            discount_minor: l.discount_minor,
            sale_discount_minor: 0,
            tax_minor: 0,
            line_total_minor: gross - l.discount_minor,
            allocations,
        });
    }
    let net = subtotal - line_discounts;
    if cart.discount_minor as i128 > net {
        return Err(invalid("Sale discount exceeds the net subtotal."));
    }
    let after_discount = net - cart.discount_minor as i128;
    if cart.tax_minor > 0 && after_discount == 0 {
        return Err(invalid("Tax requires a positive net amount."));
    }
    // Cumulative integer apportionment preserves every minor unit in stable cart order.
    let (mut cumulative, mut prior_discount, mut after_cumulative, mut prior_tax) =
        (0i128, 0i128, 0i128, 0i128);
    for l in &mut lines {
        cumulative += l.line_total_minor as i128;
        let discount = if net == 0 {
            0
        } else {
            cart.discount_minor as i128 * cumulative / net
        };
        l.sale_discount_minor = (discount - prior_discount) as i64;
        prior_discount = discount;
        l.line_total_minor -= l.sale_discount_minor;
        after_cumulative += l.line_total_minor as i128;
        let tax = if after_discount == 0 {
            0
        } else {
            cart.tax_minor as i128 * after_cumulative / after_discount
        };
        l.tax_minor = (tax - prior_tax) as i64;
        prior_tax = tax;
        l.line_total_minor = money(l.line_total_minor as i128 + l.tax_minor as i128)?;
    }
    Ok(Preview {
        subtotal_minor: money(subtotal)?,
        discount_minor: money(line_discounts + cart.discount_minor as i128)?,
        tax_minor: cart.tax_minor,
        total_minor: money(after_discount + cart.tax_minor as i128)?,
        lines,
    })
}
pub fn preview(db: &AppDb, cart: &Cart) -> Result<Preview, AppError> {
    calculate(&*db.lock()?, cart, true)
}
pub fn save(db: &AppDb, input: &Save) -> Result<Value, AppError> {
    id(&input.id)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let totals = calculate(&tx, &input.cart, false)?;
    let mut cart = input.cart.clone();
    for (l, p) in cart.lines.iter_mut().zip(&totals.lines) {
        l.unit_price_minor = Some(p.unit_price_minor);
        l.display_name = Some(format!(
            "{} · {}",
            p.product_name_snapshot, p.package_label_snapshot
        ));
    }
    let exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM sales WHERE id=?1)",
        [&input.id],
        |r| r.get(0),
    )?;
    if exists {
        let (s, r, old) = stored(&tx, &input.id)?;
        if input.expected_revision.is_none() && old == cart {
            let result = detail_on(&tx, &input.id)?;
            tx.commit()?;
            return Ok(result);
        }
        editable(
            &s,
            r,
            input
                .expected_revision
                .ok_or_else(|| invalid("Expected revision is required."))?,
        )?;
        if old.branch_id != cart.branch_id {
            return Err(invalid("A persisted cart cannot change branch."));
        }
        tx.execute("UPDATE sales SET draft_json=?2,customer_name=?3,notes=?4,revision=revision+1 WHERE id=?1",params![input.id,text(&cart)?,cart.customer_name,cart.notes])?;
    } else {
        if input.expected_revision.is_some() {
            return Err(invalid("Sale no longer exists."));
        }
        tx.execute("INSERT INTO sales(id,branch_id,status,draft_json,customer_name,notes) VALUES(?1,?2,'DRAFT',?3,?4,?5)",params![input.id,cart.branch_id,text(&cart)?,cart.customer_name,cart.notes])?;
    }
    tx.execute("UPDATE sales SET subtotal_minor=?2,discount_minor=?3,tax_minor=?4,total_minor=?5 WHERE id=?1",params![input.id,totals.subtotal_minor,totals.discount_minor,totals.tax_minor,totals.total_minor])?;
    let result = detail_on(&tx, &input.id)?;
    tx.commit()?;
    Ok(result)
}
pub fn transition(db: &AppDb, input: &Transition, hold: bool) -> Result<Value, AppError> {
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (s, r, _) = stored(&tx, &input.id)?;
    editable(&s, r, input.expected_revision)?;
    tx.execute(
        "UPDATE sales SET status=?2,revision=revision+1 WHERE id=?1",
        params![input.id, if hold { "HELD" } else { "DRAFT" }],
    )?;
    let result = detail_on(&tx, &input.id)?;
    tx.commit()?;
    Ok(result)
}
fn retry(
    c: &Connection,
    request: &str,
    kind: &str,
    target: &str,
    payload: &str,
) -> Result<bool, AppError> {
    id(request)?;
    let old: Option<(String, String, String)> = c
        .query_row(
            "SELECT kind,target_id,payload FROM sale_requests WHERE request_id=?1",
            [request],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    if let Some(v) = old {
        if v != (kind.into(), target.into(), payload.into()) {
            return Err(invalid("Request ID was already used with another payload."));
        }
        return Ok(true);
    }
    Ok(false)
}
fn receipt(
    c: &Connection,
    request: &str,
    kind: &str,
    target: &str,
    payload: &str,
) -> Result<(), AppError> {
    c.execute(
        "INSERT INTO sale_requests(request_id,kind,target_id,payload) VALUES(?1,?2,?3,?4)",
        params![request, kind, target, payload],
    )?;
    Ok(())
}
pub fn post(db: &AppDb, input: &Post) -> Result<Value, AppError> {
    let payload = text(input)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if retry(&tx, &input.request_id, "POST", &input.id, &payload)? {
        return detail_on(&tx, &input.id);
    }
    let (s, r, cart) = stored(&tx, &input.id)?;
    editable(&s, r, input.expected_revision)?;
    if cart.lines.is_empty() {
        return Err(invalid("Add a package before posting."));
    }
    let p = calculate(&tx, &cart, true)?;
    if input.payments.is_empty() || input.payments.len() > 10 {
        return Err(invalid("Provide 1–10 payment records."));
    }
    let mut paid = 0i128;
    for pay in &input.payments {
        money(pay.amount_minor as i128)?;
        optional(&pay.reference)?;
        paid += pay.amount_minor as i128;
    }
    if paid != p.total_minor as i128 {
        return Err(invalid("Payments must equal the exact sale total."));
    }
    let day: String = tx.query_row("SELECT strftime('%Y%m%d','now')", [], |r| r.get(0))?;
    tx.execute("INSERT INTO sale_sequences(day,next_number) VALUES(?1,1) ON CONFLICT(day) DO UPDATE SET next_number=next_number+1",[&day])?;
    let n: i64 = tx.query_row(
        "SELECT next_number FROM sale_sequences WHERE day=?1",
        [&day],
        |r| r.get(0),
    )?;
    let number = format!("SAL-{day}-{n:04}");
    for (position, l) in p.lines.iter().enumerate() {
        let item = uuid();
        let costs: HashSet<Option<i64>> = l.allocations.iter().map(|a| a.unit_cost_minor).collect();
        let cost = if costs.len() == 1 {
            l.allocations[0].unit_cost_minor
        } else {
            None
        };
        tx.execute("INSERT INTO sale_items(id,sale_id,position,product_id,product_package_id,product_name_snapshot,package_label_snapshot,quantity,unit_price_minor,discount_minor,sale_discount_minor,tax_minor,line_total_minor,unit_cost_minor) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",params![item,input.id,position as i64,l.product_id,l.product_package_id,l.product_name_snapshot,l.package_label_snapshot,l.quantity,l.unit_price_minor,l.discount_minor,l.sale_discount_minor,l.tax_minor,l.line_total_minor,cost])?;
        for (pos, a) in l.allocations.iter().enumerate() {
            let allocation = uuid();
            let op = StockOperation {
                request_id: uuid(),
                branch_id: cart.branch_id.clone(),
                product_package_id: l.product_package_id.clone(),
                batch_id: a.batch_id.clone(),
                quantity: a.quantity,
                reason: format!("Sale {number}"),
                created_by: None,
            };
            let m = movements::insert_referenced(
                &tx,
                &op,
                MovementType::Sale,
                None,
                Some(("sale_item_allocation", &allocation)),
            )?;
            tx.execute("INSERT INTO sale_item_allocations(id,sale_item_id,position,batch_id,quantity,unit_cost_minor,movement_id) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![allocation,item,pos as i64,a.batch_id,a.quantity,a.unit_cost_minor,m.id])?;
        }
    }
    for pay in &input.payments {
        tx.execute("INSERT INTO sale_payments(id,sale_id,payment_method,amount_minor,reference) VALUES(?1,?2,?3,?4,?5)",params![uuid(),input.id,pay.payment_method.as_str(),pay.amount_minor,pay.reference])?;
    }
    tx.execute("UPDATE sales SET status='POSTED',revision=revision+1,sale_number=?2,payment_status='PAID',post_request_id=?3,posted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),subtotal_minor=?4,discount_minor=?5,tax_minor=?6,total_minor=?7 WHERE id=?1",params![input.id,number,input.request_id,p.subtotal_minor,p.discount_minor,p.tax_minor,p.total_minor])?;
    receipt(&tx, &input.request_id, "POST", &input.id, &payload)?;
    let result = detail_on(&tx, &input.id)?;
    tx.commit()?;
    drop(c);
    inventory::alerts::refresh_all(db);
    Ok(result)
}
const HEADER:&str="SELECT json_object('id',id,'saleNumber',sale_number,'branchId',branch_id,'customerName',customer_name,'status',status,'revision',revision,'cart',json(draft_json),'subtotalMinor',subtotal_minor,'discountMinor',discount_minor,'taxMinor',tax_minor,'totalMinor',total_minor,'paymentStatus',payment_status,'paymentMethods',(SELECT group_concat(DISTINCT payment_method) FROM sale_payments WHERE sale_id=sales.id),'refundedMinor',(SELECT coalesce(sum(amount_minor),0) FROM sale_refunds WHERE sale_id=sales.id),'createdAt',created_at,'postedAt',posted_at,'voidedAt',voided_at,'notes',notes) FROM sales";
pub fn detail(db: &AppDb, sale: &str) -> Result<Value, AppError> {
    detail_on(&*db.lock()?, sale)
}
fn detail_on(c: &Connection, sale: &str) -> Result<Value, AppError> {
    id(sale)?;
    let mut result: Value = c.query_row(&format!("{HEADER} WHERE id=?1"), [sale], jsonrow)?;
    let mut items=objects(c,"SELECT json_object('id',i.id,'productPackageId',product_package_id,'productNameSnapshot',product_name_snapshot,'packageLabelSnapshot',package_label_snapshot,'quantity',quantity,'unitPriceMinor',unit_price_minor,'discountMinor',discount_minor,'saleDiscountMinor',sale_discount_minor,'taxMinor',tax_minor,'lineTotalMinor',line_total_minor,'unitCostMinor',unit_cost_minor,'returnedQuantity',coalesce((SELECT sum(ri.quantity) FROM sale_return_items ri JOIN sale_returns r ON r.id=ri.return_id WHERE ri.sale_item_id=i.id AND r.status='POSTED'),0)) FROM sale_items i WHERE sale_id=?1 ORDER BY position",sale)?;
    for item in &mut items {
        item["remainingQuantity"] = json!(if result["status"] == "VOIDED" {
            0
        } else {
            item["quantity"].as_i64().unwrap_or(0) - item["returnedQuantity"].as_i64().unwrap_or(0)
        });
        item["allocations"]=json!(objects(c,"SELECT json_object('id',a.id,'batchId',a.batch_id,'quantity',a.quantity,'unitCostMinor',a.unit_cost_minor,'movementId',a.movement_id,'lotNumber',b.lot_number,'expiryDate',b.expiry_date) FROM sale_item_allocations a JOIN inventory_batches b ON b.id=a.batch_id WHERE sale_item_id=?1 ORDER BY position",item["id"].as_str().unwrap_or(""))?);
    }
    result["items"] = json!(items);
    result["payments"]=json!(objects(c,"SELECT json_object('id',id,'paymentMethod',payment_method,'amountMinor',amount_minor,'reference',reference) FROM sale_payments WHERE sale_id=?1 ORDER BY created_at,id",sale)?);
    let mut returns=objects(c,"SELECT json_object('id',id,'status',status,'reason',reason,'refundMinor',refund_minor,'lines',json(lines_json),'createdAt',created_at,'postedAt',posted_at) FROM sale_returns WHERE sale_id=?1 ORDER BY created_at,id",sale)?;
    for r in &mut returns {
        r["allocations"]=json!(objects(c,"SELECT json_object('originalAllocationId',a.original_allocation_id,'quantity',a.quantity,'movementId',a.movement_id,'refundMinor',i.refund_minor) FROM sale_return_allocations a JOIN sale_return_items i ON i.id=a.return_item_id WHERE i.return_id=?1 ORDER BY a.id",r["id"].as_str().unwrap_or(""))?);
    }
    result["returns"] = json!(returns);
    result["voids"]=json!(objects(c,"SELECT json_object('id',id,'reason',reason,'refundMinor',refund_minor,'createdAt',created_at) FROM sale_voids WHERE sale_id=?1",sale)?);
    result["voidAllocations"]=json!(objects(c,"SELECT json_object('originalAllocationId',a.original_allocation_id,'movementId',a.movement_id) FROM sale_void_allocations a JOIN sale_voids v ON v.id=a.void_id WHERE v.sale_id=?1",sale)?);
    result["refundedMinor"] = json!(c.query_row(
        "SELECT coalesce(sum(amount_minor),0) FROM sale_refunds WHERE sale_id=?1",
        [sale],
        |r| r.get::<_, i64>(0)
    )?);
    Ok(result)
}
pub fn list(db: &AppDb, q: &Query) -> Result<Value, AppError> {
    let c = db.lock()?;
    branch(&c, &q.branch_id)?;
    let (l, o) = inventory::page(q.limit, q.offset)?;
    if q.status
        .as_ref()
        .is_some_and(|s| !["DRAFT", "HELD", "POSTED", "VOIDED"].contains(&s.as_str()))
    {
        return Err(invalid("Invalid sale status."));
    }
    for d in [&q.date_from, &q.date_to].into_iter().flatten() {
        inventory::date(&c, d)?;
    }
    if matches!((&q.date_from,&q.date_to),(Some(a),Some(b)) if a>b) {
        return Err(invalid("Invalid date range."));
    }
    let filter="branch_id=?1 AND (?2 IS NULL OR status=?2) AND (?3='' OR instr(lower(coalesce(sale_number,'')||' '||coalesce(customer_name,'')),lower(?3))>0) AND (?4 IS NULL OR substr(created_at,1,10)>=?4) AND (?5 IS NULL OR substr(created_at,1,10)<=?5) AND (?6 IS NULL OR EXISTS(SELECT 1 FROM sale_payments p WHERE p.sale_id=sales.id AND p.payment_method=?6))";
    let search = q.search.as_deref().unwrap_or("").trim();
    let method = q.payment_method.as_ref().map(Method::as_str);
    let total: i64 = c.query_row(
        &format!("SELECT count(*) FROM sales WHERE {filter}"),
        params![
            q.branch_id,
            q.status,
            search,
            q.date_from,
            q.date_to,
            method
        ],
        |r| r.get(0),
    )?;
    let mut s = c.prepare(&format!(
        "{HEADER} WHERE {filter} ORDER BY created_at DESC,id DESC LIMIT ?7 OFFSET ?8"
    ))?;
    let items = s
        .query_map(
            params![
                q.branch_id,
                q.status,
                search,
                q.date_from,
                q.date_to,
                method,
                l,
                o
            ],
            jsonrow,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({"items":items,"total":total}))
}
pub fn search(db: &AppDb, q: &Query) -> Result<Value, AppError> {
    let c = db.lock()?;
    branch(&c, &q.branch_id)?;
    let (l, o) = inventory::page(q.limit, q.offset)?;
    let search = q.search.as_deref().unwrap_or("").trim();
    let filter="k.is_active=1 AND p.is_active=1 AND (?1='' OR instr(lower(coalesce(p.commercial_name_en,'')||' '||coalesce(p.commercial_name_ar,'')||' '||coalesce(p.scientific_name,'')),lower(?1))>0 OR EXISTS(SELECT 1 FROM barcodes b WHERE b.product_package_id=k.id AND b.barcode=?1))";
    let total:i64=c.query_row(&format!("SELECT count(*) FROM product_packages k JOIN products p ON p.id=k.product_id WHERE {filter}"),[search],|r|r.get(0))?;
    let mut s=c.prepare(&format!("SELECT json_object('productPackageId',k.id,'productId',p.id,'productName',coalesce(p.commercial_name_en,p.commercial_name_ar),'arabicName',p.commercial_name_ar,'scientificName',p.scientific_name,'packageLabel',k.package_label,'currentPriceMinor',(SELECT selling_price_minor FROM product_price_history WHERE product_package_id=k.id AND effective_to IS NULL),'barcodeMatch',EXISTS(SELECT 1 FROM barcodes b WHERE b.product_package_id=k.id AND b.barcode=?1)) FROM product_packages k JOIN products p ON p.id=k.product_id WHERE {filter} ORDER BY EXISTS(SELECT 1 FROM barcodes b WHERE b.product_package_id=k.id AND b.barcode=?1) DESC,p.commercial_name_en,k.id LIMIT ?2 OFFSET ?3"))?;
    let mut items = s
        .query_map(params![search, l, o], jsonrow)?
        .collect::<Result<Vec<_>, _>>()?;
    for item in &mut items {
        let rows = eligible(
            &c,
            &q.branch_id,
            item["productPackageId"].as_str().unwrap_or(""),
        )?;
        let available = money(rows.iter().map(|r| r.quantity as i128).sum())?;
        item["availableQuantity"] = json!(available);
        item["stockStatus"] = json!(if available == 0 {
            "OUT_OF_STOCK"
        } else {
            "IN_STOCK"
        });
    }
    Ok(json!({"items":items,"total":total}))
}
