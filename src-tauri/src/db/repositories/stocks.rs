use crate::{
    db::{catalog_validation::*, connection::AppDb, models::inventory::*, repositories::batches},
    error::{AppError, ErrorCode},
};
use rusqlite::{params, Connection, Row};
/// Branch-aware package-level stock overview built from the ledger view. Status,
/// aggregations and sorting are computed in SQL so filtering and pagination stay
/// server-side; per-row batch chips are enriched afterwards.
const INNER_BASE: &str = r"SELECT
    pp.id AS package_id,
    pp.product_id,
    pp.package_label,
    pp.pack_size,
    pp.is_active AS package_active,
    pr.commercial_name_en AS name_en,
    pr.commercial_name_ar AS name_ar,
    pr.scientific_name,
    pr.manufacturer_id,
    pr.category_id,
    mf.name AS manufacturer_name,
    mf.normalized_name AS manufacturer_normalized,
    (SELECT COALESCE(SUM(bal.quantity),0) FROM stock_balances bal JOIN inventory_batches b ON b.id=bal.batch_id WHERE b.branch_id=?1 AND b.product_package_id=pp.id AND b.is_active=1) AS quantity,
    (SELECT COUNT(*) FROM stock_balances bal JOIN inventory_batches b ON b.id=bal.batch_id WHERE b.branch_id=?1 AND b.product_package_id=pp.id AND b.is_active=1 AND bal.quantity>0) AS batch_count,
    (SELECT COALESCE((SELECT reorder_level FROM inventory_settings s WHERE s.branch_id=?1 AND s.product_package_id=pp.id),10)) AS reorder_level,
    (SELECT MIN(CASE WHEN b.expiry_date<>'' THEN CAST(julianday(b.expiry_date)-julianday('now') AS INTEGER) END) FROM stock_balances bal JOIN inventory_batches b ON b.id=bal.batch_id WHERE b.branch_id=?1 AND b.product_package_id=pp.id AND b.is_active=1 AND bal.quantity>0) AS min_expiry_days,
    (SELECT ph.selling_price_minor FROM product_price_history ph WHERE ph.product_package_id=pp.id AND ph.effective_to IS NULL LIMIT 1) AS selling_price_minor,
    (SELECT b.cost_price_minor FROM inventory_batches b LEFT JOIN stock_balances bal ON bal.batch_id=b.id WHERE b.branch_id=?1 AND b.product_package_id=pp.id AND b.is_active=1 AND bal.quantity>0 ORDER BY CASE WHEN b.expiry_date='' THEN 1 ELSE 0 END, b.expiry_date LIMIT 1) AS cost_price_minor
FROM product_packages pp
JOIN products pr ON pr.id=pp.product_id
LEFT JOIN manufacturers mf ON mf.id=pr.manufacturer_id
WHERE EXISTS(SELECT 1 FROM inventory_batches ib WHERE ib.product_package_id=pp.id AND ib.is_active=1)";
const STATUS_SQL: &str =
    "CASE WHEN package_active=0 THEN 'inactive' WHEN quantity=0 THEN 'out_of_stock' WHEN quantity < reorder_level THEN 'low' WHEN min_expiry_days IS NOT NULL AND min_expiry_days <= 30 THEN 'expiring' ELSE 'in_stock' END";
struct RawStockRow {
    package_id: String,
    product_id: String,
    package_label: String,
    pack_size: Option<String>,
    package_active: bool,
    name_en: Option<String>,
    name_ar: Option<String>,
    scientific_name: Option<String>,
    _manufacturer_id: Option<String>,
    _category_id: Option<String>,
    manufacturer_name: Option<String>,
    _manufacturer_normalized: Option<String>,
    quantity: i64,
    batch_count: i64,
    reorder_level: i64,
    min_expiry_days: Option<i64>,
    selling_price_minor: Option<i64>,
    cost_price_minor: Option<i64>,
}
fn map_raw(r: &Row<'_>) -> rusqlite::Result<RawStockRow> {
    Ok(RawStockRow {
        package_id: r.get(0)?,
        product_id: r.get(1)?,
        package_label: r.get(2)?,
        pack_size: r.get(3)?,
        package_active: r.get(4)?,
        name_en: r.get(5)?,
        name_ar: r.get(6)?,
        scientific_name: r.get(7)?,
        _manufacturer_id: r.get(8)?,
        _category_id: r.get(9)?,
        manufacturer_name: r.get(10)?,
        _manufacturer_normalized: r.get(11)?,
        quantity: r.get(12)?,
        batch_count: r.get(13)?,
        reorder_level: r.get(14)?,
        min_expiry_days: r.get(15)?,
        selling_price_minor: r.get(16)?,
        cost_price_minor: r.get(17)?,
    })
}
fn status_of(r: &RawStockRow) -> &'static str {
    if !r.package_active {
        "inactive"
    } else if r.quantity == 0 {
        "out_of_stock"
    } else if r.quantity < r.reorder_level {
        "low"
    } else if r.min_expiry_days.is_some_and(|d| d <= 30) {
        "expiring"
    } else {
        "in_stock"
    }
}
fn validate_page(
    limit_input: Option<i64>,
    offset_input: Option<i64>,
) -> Result<(i64, i64), AppError> {
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
/// Enriches each raw row with its FEFO batch chips inside the same read snapshot.
fn enrich_batches(
    tx: &Connection,
    branch_id: &str,
    rows: &[RawStockRow],
) -> Result<Vec<StockRow>, AppError> {
    rows.iter()
        .map(|r| {
            let mini = batches::list_active_with_balance_on(tx, branch_id, &r.package_id)?
                .into_iter()
                .filter(|(_, qty)| *qty > 0)
                .take(6)
                .map(|(b, qty)| BatchMini {
                    batch_id: b.id,
                    batch_number: b.batch_number,
                    expiry_date: b.expiry_date,
                    quantity: qty,
                })
                .collect::<Vec<_>>();
            Ok(StockRow {
                package_id: r.package_id.clone(),
                product_id: r.product_id.clone(),
                name_en: r.name_en.clone(),
                name_ar: r.name_ar.clone(),
                scientific_name: r.scientific_name.clone(),
                manufacturer_name: r.manufacturer_name.clone(),
                package_label: r.package_label.clone(),
                pack_size: r.pack_size.clone(),
                quantity: r.quantity,
                reorder_level: r.reorder_level,
                batch_count: r.batch_count,
                min_expiry_days: r.min_expiry_days,
                selling_price_minor: r.selling_price_minor,
                cost_price_minor: r.cost_price_minor,
                status: status_of(r).into(),
                batches: mini,
            })
        })
        .collect()
}
fn order_clause(sort: &Option<String>, direction_sql: &str) -> Result<String, AppError> {
    let key = match sort.as_deref().map(str::trim) {
        None | Some("") | Some("name") => "COALESCE(name_en,name_ar)",
        Some("manufacturer") => "COALESCE(manufacturer_normalized,'')",
        Some("quantity") => "quantity",
        Some("reorder") => "reorder_level",
        Some("expiry") => "min_expiry_days",
        Some("price") => "selling_price_minor",
        Some("status") => "status",
        Some(other) => return Err(invalid(&format!("Unknown sort key: {other}."))),
    };
    Ok(format!("ORDER BY {key} {direction_sql}, package_id"))
}
pub fn overview(db: &AppDb, q: &StockOverviewQuery) -> Result<StockOverviewPage, AppError> {
    id(&q.branch_id)?;
    for value in [&q.manufacturer_id, &q.category_id].into_iter().flatten() {
        id(value)?;
    }
    if q.search.as_ref().is_some_and(|v| v.chars().count() > 4000) {
        return Err(invalid("Search text is too long."));
    }
    if let Some(status) = &q.status {
        if ![
            "in_stock",
            "low",
            "out_of_stock",
            "expiring",
            "inactive",
            "",
        ]
        .contains(&status.as_str())
        {
            return Err(invalid("Unknown stock status filter."));
        }
    }
    let (limit, offset) = validate_page(q.limit, q.offset)?;
    let direction = match q.sort_direction.as_deref().map(str::trim) {
        None | Some("") | Some("asc") => "ASC",
        Some("desc") => "DESC",
        Some(other) => return Err(invalid(&format!("Unknown sort direction: {other}."))),
    };
    let (search, pattern) = search_pattern(q.search.as_deref().unwrap_or(""));
    let exact = q.search.as_deref().unwrap_or("").trim();
    let base_filter = r"WHERE (?2='' OR name_en LIKE ?3 ESCAPE '\' OR name_ar LIKE ?3 ESCAPE '\' OR scientific_name LIKE ?3 ESCAPE '\' OR manufacturer_normalized LIKE ?3 ESCAPE '\' OR package_label LIKE ?3 ESCAPE '\' OR EXISTS(SELECT 1 FROM barcodes bc WHERE bc.product_package_id=t.package_id AND bc.barcode=?4) OR EXISTS(SELECT 1 FROM inventory_batches ib WHERE ib.branch_id=?1 AND ib.product_package_id=t.package_id AND ib.batch_number=?4)) AND (?5 IS NULL OR t.manufacturer_id=?5) AND (?6 IS NULL OR t.category_id=?6)";
    let inner = format!("SELECT t.*, {STATUS_SQL} AS status FROM ({INNER_BASE}) t {base_filter}");
    let status_filter = "(?7='' OR s.status=?7)";
    let full = format!("SELECT * FROM ({inner}) s WHERE {status_filter}");
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let total = tx.query_row(
        &format!("SELECT count(*) FROM ({full})"),
        params![
            q.branch_id,
            search,
            pattern,
            exact,
            None::<String>,
            None::<String>,
            q.status.as_deref().unwrap_or("")
        ],
        |r| r.get(0),
    )?;
    let items = {
        let order = order_clause(&q.sort, direction)?;
        let mut s = tx.prepare(&format!(
            "SELECT * FROM ({full}) {order} LIMIT ?8 OFFSET ?9"
        ))?;
        let rows = s.query_map(
            params![
                q.branch_id,
                search,
                pattern,
                exact,
                q.manufacturer_id,
                q.category_id,
                q.status.as_deref().unwrap_or(""),
                limit,
                offset
            ],
            map_raw,
        )?;
        let rows: Vec<RawStockRow> = rows.collect::<Result<Vec<_>, _>>()?;
        enrich_batches(&tx, &q.branch_id, &rows)?
    };
    tx.commit()?;
    Ok(StockOverviewPage {
        items,
        total,
        limit,
        offset,
    })
}
pub fn summary(db: &AppDb, branch_id: &str) -> Result<StockSummary, AppError> {
    id(branch_id)?;
    let c = db.lock()?;
    let value_minor = c.query_row(
        "SELECT COALESCE(SUM(bal.quantity*COALESCE(b.cost_price_minor,0)),0) FROM stock_balances bal JOIN inventory_batches b ON b.id=bal.batch_id WHERE b.branch_id=?1 AND b.is_active=1 AND bal.quantity>0",
        [branch_id],
        |r| r.get(0),
    )?;
    let statuses: Vec<(String, i64)> = {
        let mut s = c.prepare(&format!(
            "SELECT {STATUS_SQL} AS status, COUNT(*) FROM ({INNER_BASE}) t GROUP BY status"
        ))?;
        let rows = s.query_map([branch_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    let mut out = StockSummary {
        in_stock: 0,
        low: 0,
        out_of_stock: 0,
        value_minor,
    };
    for (status, count) in statuses {
        match status.as_str() {
            "in_stock" => out.in_stock = count,
            "low" => out.low = count,
            "out_of_stock" => out.out_of_stock = count,
            _ => {}
        }
    }
    Ok(out)
}
fn expiry_filter(window_days: &Option<i64>) -> (String, bool) {
    match window_days {
        Some(-1) => ("b.expiry_date<>'' AND CAST(julianday(b.expiry_date)-julianday('now') AS INTEGER)<0".into(), true),
        Some(n) => (format!("b.expiry_date<>'' AND CAST(julianday(b.expiry_date)-julianday('now') AS INTEGER) BETWEEN 0 AND {n}"), false),
        None => (format!("b.expiry_date<>'' AND CAST(julianday(b.expiry_date)-julianday('now') AS INTEGER) BETWEEN 0 AND 30"), false),
    }
}
pub fn expiry(db: &AppDb, q: &ExpiryQuery) -> Result<ExpiryPage, AppError> {
    id(&q.branch_id)?;
    if q.search.as_ref().is_some_and(|v| v.chars().count() > 4000) {
        return Err(invalid("Search text is too long."));
    }
    let (limit, offset) = validate_page(q.limit, q.offset)?;
    let (search, pattern) = search_pattern(q.search.as_deref().unwrap_or(""));
    let exact = q.search.as_deref().unwrap_or("").trim();
    let (window_sql, _) = expiry_filter(&q.window_days);
    let filter = format!(
        r"WHERE b.branch_id=?1 AND {window_sql} AND COALESCE(bal.quantity,0)>0 AND (?2='' OR COALESCE(NULLIF(pr.commercial_name_en,''),pr.scientific_name,pr.commercial_name_ar,'') LIKE ?3 ESCAPE '\' OR pp.package_label LIKE ?3 ESCAPE '\' OR b.batch_number=?4)"
    );
    let select = r"SELECT b.id, pp.id, pp.product_id, pr.commercial_name_en, pr.commercial_name_ar, pr.scientific_name, pp.package_label, b.batch_number, b.expiry_date, CAST(julianday(b.expiry_date)-julianday('now') AS INTEGER), COALESCE(bal.quantity,0), COALESCE(bal.quantity,0)*COALESCE(b.cost_price_minor,0), b.cost_price_minor, br.name FROM inventory_batches b JOIN branches br ON br.id=b.branch_id JOIN product_packages pp ON pp.id=b.product_package_id JOIN products pr ON pr.id=pp.product_id LEFT JOIN stock_balances bal ON bal.batch_id=b.id";
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let scoped = format!("{select} {filter}");
    let total = tx.query_row(
        &format!("SELECT count(*) FROM ({scoped})"),
        params![q.branch_id, search, pattern, exact],
        |r| r.get(0),
    )?;
    let items = {
        let mut s = tx.prepare(&format!(
            "SELECT * FROM ({scoped}) ORDER BY expiry_date, batch_number LIMIT ?5 OFFSET ?6"
        ))?;
        let rows = s.query_map(
            params![q.branch_id, search, pattern, exact, limit, offset],
            |r| {
                Ok(ExpiryRow {
                    batch_id: r.get(0)?,
                    package_id: r.get(1)?,
                    product_id: r.get(2)?,
                    name_en: r.get(3)?,
                    name_ar: r.get(4)?,
                    scientific_name: r.get(5)?,
                    package_label: r.get(6)?,
                    batch_number: r.get(7)?,
                    expiry_date: r.get(8)?,
                    days: r.get(9)?,
                    quantity: r.get(10)?,
                    value_minor: r.get(11)?,
                    cost_price_minor: r.get(12)?,
                    branch_name: r.get(13)?,
                })
            },
        )?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    tx.commit()?;
    Ok(ExpiryPage {
        items,
        total,
        limit,
        offset,
    })
}
pub fn low_stock(db: &AppDb, q: &LowStockQuery) -> Result<LowStockPage, AppError> {
    id(&q.branch_id)?;
    if q.search.as_ref().is_some_and(|v| v.chars().count() > 4000) {
        return Err(invalid("Search text is too long."));
    }
    let (limit, offset) = validate_page(q.limit, q.offset)?;
    let (search, pattern) = search_pattern(q.search.as_deref().unwrap_or(""));
    let filter = r"WHERE (?2='' OR name_en LIKE ?3 ESCAPE '\' OR name_ar LIKE ?3 ESCAPE '\' OR package_label LIKE ?3 ESCAPE '\') AND (quantity=0 OR quantity < reorder_level) AND package_active=1";
    let select = format!(
        r"SELECT package_id, product_id, name_en, name_ar, package_label, quantity, reorder_level, CASE WHEN quantity=0 THEN 'out_of_stock' ELSE 'low' END AS status FROM ({INNER_BASE}) t {filter}"
    );
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let total = tx.query_row(
        &format!("SELECT count(*) FROM ({select})"),
        params![q.branch_id, search, pattern],
        |r| r.get(0),
    )?;
    let items = {
        let mut s = tx.prepare(&format!(
            "{select} ORDER BY status, reorder_level - quantity DESC LIMIT ?4 OFFSET ?5"
        ))?;
        let rows = s.query_map(params![q.branch_id, search, pattern, limit, offset], |r| {
            Ok(LowStockRow {
                package_id: r.get(0)?,
                product_id: r.get(1)?,
                name_en: r.get(2)?,
                name_ar: r.get(3)?,
                package_label: r.get(4)?,
                quantity: r.get(5)?,
                reorder_level: r.get(6)?,
                status: r.get(7)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    tx.commit()?;
    Ok(LowStockPage {
        items,
        total,
        limit,
        offset,
    })
}
/// FEFO allocation: batches with stock ordered by earliest expiry first (no-expiry
/// lots last). Read-only foundation for the future POS; the optional `quantity`
/// bounds the allocation and errors when coverage is insufficient.
pub fn fefo_allocation(
    db: &AppDb,
    branch_id: &str,
    product_package_id: &str,
    quantity: Option<i64>,
) -> Result<Vec<FefoAllocation>, AppError> {
    id(branch_id)?;
    id(product_package_id)?;
    if quantity.is_some_and(|v| v <= 0) {
        return Err(invalid("Quantity must be positive."));
    }
    let c = db.lock()?;
    let available = batches::package_balance_on(&c, branch_id, product_package_id)?;
    if let Some(q) = quantity {
        if available < q {
            return Err(AppError::new(
                ErrorCode::Conflict,
                "Insufficient stock to cover the requested allocation.",
            ));
        }
    }
    let batches = batches::list_active_with_balance_on(&c, branch_id, product_package_id)?;
    let mut remaining = quantity;
    let mut out = Vec::new();
    for (batch, qty) in batches {
        if qty <= 0 {
            continue;
        }
        let take = match remaining {
            Some(r) if r > 0 => {
                let t = r.min(qty);
                remaining = Some(r - t);
                t
            }
            _ => 0,
        };
        out.push(FefoAllocation {
            batch_id: batch.id,
            batch_number: batch.batch_number,
            expiry_date: batch.expiry_date,
            available: qty,
            take,
        });
    }
    Ok(out)
}
