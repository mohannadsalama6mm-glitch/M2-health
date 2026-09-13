use crate::{
    db::{catalog_validation::*, connection::AppDb, models::inventory::*},
    error::{AppError, ErrorCode},
};
use rusqlite::{params, Connection, OptionalExtension, Row};
fn map_count(r: &Row<'_>) -> rusqlite::Result<StockCount> {
    Ok(StockCount {
        id: r.get(0)?,
        branch_id: r.get(1)?,
        scope: r.get(2)?,
        category_id: r.get(3)?,
        status: r.get(4)?,
        started_at: r.get(5)?,
        completed_at: r.get(6)?,
        created_at: r.get(7)?,
        updated_at: r.get(8)?,
    })
}
pub(crate) fn insert_count_on(
    c: &Connection,
    id: &str,
    branch_id: &str,
    scope: &str,
    category_id: Option<&str>,
) -> Result<StockCount, AppError> {
    c.execute(
        "INSERT INTO stock_counts(id,branch_id,scope,category_id) VALUES (?1,?2,?3,?4)",
        params![id, branch_id, scope, category_id],
    )?;
    get(c, id)
}
pub(crate) fn get(c: &Connection, value: &str) -> Result<StockCount, AppError> {
    c.query_row(
        "SELECT id, branch_id, scope, category_id, status, started_at, completed_at, created_at, updated_at FROM stock_counts WHERE id=?1",
        [value],
        map_count,
    )
    .optional()?
    .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Stock count session not found."))
}
pub(crate) fn mark_in_progress_on(c: &Connection, value: &str) -> Result<(), AppError> {
    c.execute("UPDATE stock_counts SET status='in_progress',updated_at=strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),julianday(updated_at)+0.001/86400)) WHERE id=?1 AND status!='completed'",[value])?;
    Ok(())
}
pub(crate) fn complete_on(c: &Connection, value: &str) -> Result<(), AppError> {
    c.execute("UPDATE stock_counts SET status='completed',completed_at=strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),coalesce(julianday(started_at)+1.0/86400,0))),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),julianday(updated_at)+0.001/86400)) WHERE id=?1",[value])?;
    Ok(())
}
pub(crate) fn insert_item_on(
    c: &Connection,
    stock_count_id: &str,
    product_package_id: &str,
    batch_id: &str,
    system_quantity: i64,
) -> Result<StockCountItem, AppError> {
    let value = uuid::Uuid::new_v4().to_string();
    c.execute(
        "INSERT INTO stock_count_items(id,stock_count_id,product_package_id,batch_id,system_quantity,counted_quantity,variance) VALUES (?1,?2,?3,?4,?5,?5,0)",
        params![value, stock_count_id, product_package_id, batch_id, system_quantity],
    )?;
    get_item(c, &value)
}
pub(crate) fn get_item(c: &Connection, value: &str) -> Result<StockCountItem, AppError> {
    c.query_row(
        "SELECT id, stock_count_id, product_package_id, batch_id, system_quantity, counted_quantity, variance FROM stock_count_items WHERE id=?1",
        [value],
        |r| {
            Ok(StockCountItem {
                id: r.get(0)?,
                stock_count_id: r.get(1)?,
                product_package_id: r.get(2)?,
                batch_id: r.get(3)?,
                system_quantity: r.get(4)?,
                counted_quantity: r.get(5)?,
                variance: r.get(6)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Count line not found."))
}
pub(crate) fn item_by_batch_on(
    c: &Connection,
    stock_count_id: &str,
    batch_id: &str,
) -> Result<StockCountItem, AppError> {
    c.query_row(
        "SELECT id, stock_count_id, product_package_id, batch_id, system_quantity, counted_quantity, variance FROM stock_count_items WHERE stock_count_id=?1 AND batch_id=?2",
        params![stock_count_id, batch_id],
        |r| {
            Ok(StockCountItem {
                id: r.get(0)?,
                stock_count_id: r.get(1)?,
                product_package_id: r.get(2)?,
                batch_id: r.get(3)?,
                system_quantity: r.get(4)?,
                counted_quantity: r.get(5)?,
                variance: r.get(6)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Count line not found."))
}
/// Records a counted quantity; the CHECK constraint maintains variance and the
/// counted value cannot go negative.
pub(crate) fn save_counted_on(
    c: &Connection,
    stock_count_id: &str,
    batch_id: &str,
    counted_quantity: i64,
) -> Result<StockCountItem, AppError> {
    let item = item_by_batch_on(c, stock_count_id, batch_id)?;
    c.execute(
        "UPDATE stock_count_items SET counted_quantity=?3, variance=?4 WHERE stock_count_id=?1 AND batch_id=?2",
        params![
            stock_count_id,
            batch_id,
            counted_quantity,
            counted_quantity - item.system_quantity
        ],
    )?;
    item_by_batch_on(c, stock_count_id, batch_id)
}
pub(crate) fn listed_items_on(
    c: &Connection,
    stock_count_id: &str,
) -> Result<Vec<StockCountItem>, AppError> {
    let mut s = c.prepare(
        "SELECT id, stock_count_id, product_package_id, batch_id, system_quantity, counted_quantity, variance FROM stock_count_items WHERE stock_count_id=?1 ORDER BY variance<>0 DESC, id",
    )?;
    let rows = s.query_map([stock_count_id], |r| {
        Ok(StockCountItem {
            id: r.get(0)?,
            stock_count_id: r.get(1)?,
            product_package_id: r.get(2)?,
            batch_id: r.get(3)?,
            system_quantity: r.get(4)?,
            counted_quantity: r.get(5)?,
            variance: r.get(6)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
pub(crate) fn list_all_items_on(
    c: &Connection,
    stock_count_id: &str,
) -> Result<Vec<StockCountItem>, AppError> {
    listed_items_on(c, stock_count_id)
}
fn map_view(r: &Row<'_>) -> rusqlite::Result<StockCountItemView> {
    Ok(StockCountItemView {
        id: r.get(0)?,
        product_package_id: r.get(1)?,
        batch_id: r.get(2)?,
        product: r.get(3)?,
        pack: r.get(4)?,
        batch_number: r.get(5)?,
        system_quantity: r.get(6)?,
        counted_quantity: r.get(7)?,
        variance: r.get(8)?,
    })
}
pub fn list(db: &AppDb, q: &CountQuery) -> Result<StockCountPage, AppError> {
    let limit = q.limit.unwrap_or(50);
    let offset = q.offset.unwrap_or(0);
    if !(1..=200).contains(&limit) || !(0..=MAX_SAFE_MINOR).contains(&offset) {
        return Err(invalid("Use a limit of 1–200 and a non-negative offset."));
    }
    if let Some(branch) = &q.branch_id {
        id(branch)?;
    }
    if let Some(status) = &q.status {
        if !["draft", "in_progress", "completed"].contains(&status.as_str()) {
            return Err(invalid("Unknown stock count status."));
        }
    }
    let filter = r"WHERE (?1 IS NULL OR sc.branch_id=?1) AND (?2 IS NULL OR sc.status=?2) AND (?3='' OR sc.id LIKE ?4 ESCAPE '\' OR br.name LIKE ?4 ESCAPE '\' OR sc.scope LIKE ?4 ESCAPE '\')";
    let select = r"SELECT sc.id, br.name, sc.scope, c.name, sc.status, sc.started_at, sc.completed_at, (SELECT COUNT(*) FROM stock_count_items i WHERE i.stock_count_id=sc.id), (SELECT COUNT(*) FROM stock_count_items i WHERE i.stock_count_id=sc.id AND i.variance<>0) FROM stock_counts sc JOIN branches br ON br.id=sc.branch_id LEFT JOIN categories c ON c.id=sc.category_id";
    let pattern = format!(
        "{}%",
        normalize(q.search.as_deref().unwrap_or(""))
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    );
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let total = tx.query_row(
        &format!("SELECT count(*) FROM ({select}) {filter}"),
        params![
            q.branch_id,
            q.status,
            q.search.as_deref().unwrap_or(""),
            pattern
        ],
        |r| r.get(0),
    )?;
    let items = {
        let mut s = tx.prepare(&format!(
            "{select} {filter} ORDER BY sc.started_at DESC, sc.id LIMIT ?5 OFFSET ?6"
        ))?;
        let rows = s.query_map(
            params![
                q.branch_id,
                q.status,
                q.search.as_deref().unwrap_or(""),
                pattern,
                limit,
                offset
            ],
            |r| {
                Ok(StockCountRow {
                    id: r.get(0)?,
                    branch_name: r.get(1)?,
                    scope: r.get(2)?,
                    category_name: r.get(3)?,
                    status: r.get(4)?,
                    started_at: r.get(5)?,
                    completed_at: r.get(6)?,
                    item_count: r.get(7)?,
                    discrepancy_count: r.get(8)?,
                })
            },
        )?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    tx.commit()?;
    Ok(StockCountPage {
        items,
        total,
        limit,
        offset,
    })
}
pub fn detail(db: &AppDb, value: &str) -> Result<StockCountDetail, AppError> {
    id(value)?;
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let count = get(&tx, value)?;
    let branch_name = tx.query_row(
        "SELECT name FROM branches WHERE id=?1",
        [&count.branch_id],
        |r| r.get(0),
    )?;
    let items = {
        let mut s = tx.prepare(
            "SELECT i.id, i.product_package_id, i.batch_id, COALESCE(NULLIF(pr.commercial_name_en,''), pr.scientific_name, pr.commercial_name_ar, '—'), pp.package_label, b.batch_number, i.system_quantity, i.counted_quantity, i.variance FROM stock_count_items i JOIN product_packages pp ON pp.id=i.product_package_id JOIN products pr ON pr.id=pp.product_id JOIN inventory_batches b ON b.id=i.batch_id WHERE i.stock_count_id=?1 ORDER BY i.variance<>0 DESC, i.id",
        )?;
        let rows = s.query_map([value], map_view)?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    tx.commit()?;
    Ok(StockCountDetail {
        count,
        branch_name,
        items,
    })
}
