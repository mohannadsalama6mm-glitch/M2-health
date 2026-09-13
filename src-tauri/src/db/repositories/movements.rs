use crate::{
    db::{
        catalog_validation::*,
        connection::AppDb,
        models::inventory::{
            MovementQuery, StockMovement, StockMovementItem, StockMovementPage, MOVEMENT_TYPES,
        },
    },
    error::AppError,
};
use rusqlite::{params, Connection, Row};

pub(crate) fn insert_on(
    c: &Connection,
    branch_id: &str,
    product_package_id: &str,
    batch_id: &str,
    movement_type: &str,
    quantity_delta: i64,
    cost_price_minor: Option<i64>,
    reference_type: Option<&str>,
    reference_id: Option<&str>,
    reason: &str,
    user: &str,
) -> Result<StockMovement, AppError> {
    let value = uuid::Uuid::new_v4().to_string();
    let now: String = c.query_row(
        "SELECT strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),coalesce(julianday((SELECT max(occurred_at) FROM stock_movements))+0.001/86400,0)))",
        [],
        |r| r.get(0),
    )?;
    c.execute(
        "INSERT INTO stock_movements(id,branch_id,product_package_id,batch_id,movement_type,quantity_delta,cost_price_minor,reference_type,reference_id,reason,user,occurred_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![
            value, branch_id, product_package_id, batch_id, movement_type, quantity_delta,
            cost_price_minor, reference_type, reference_id, reason, user, now
        ],
    )?;
    c.query_row(
        "SELECT id, branch_id, product_package_id, batch_id, movement_type, quantity_delta, cost_price_minor, reference_type, reference_id, reason, user, occurred_at, created_at FROM stock_movements WHERE id=?1",
        [value],
        |r| {
            Ok(StockMovement {
                id: r.get(0)?,
                branch_id: r.get(1)?,
                product_package_id: r.get(2)?,
                batch_id: r.get(3)?,
                movement_type: r.get(4)?,
                quantity_delta: r.get(5)?,
                cost_price_minor: r.get(6)?,
                reference_type: r.get(7)?,
                reference_id: r.get(8)?,
                reason: r.get(9)?,
                user: r.get(10)?,
                occurred_at: r.get(11)?,
                created_at: r.get(12)?,
            })
        },
    )
    .map_err(AppError::from)
}
fn map_item(r: &Row<'_>) -> rusqlite::Result<StockMovementItem> {
    let delta: i64 = r.get(4)?;
    Ok(StockMovementItem {
        id: r.get(0)?,
        occurred_at: r.get(1)?,
        movement_type: r.get(2)?,
        product: r.get(3)?,
        incoming: (delta > 0).then_some(delta),
        outgoing: (delta < 0).then_some(-delta),
        pack: r.get(5)?,
        batch: r.get(6)?,
        branch: r.get(7)?,
        user: r.get(8)?,
        reason: r.get(9)?,
    })
}
const ITEM_SELECT: &str =
    "SELECT m.id, m.occurred_at, m.movement_type, COALESCE(NULLIF(pr.commercial_name_en,''), pr.scientific_name, pr.commercial_name_ar, '—') AS product, m.quantity_delta, pp.package_label, b.batch_number, br.name AS branch_name, m.user, m.reason, m.branch_id, m.product_package_id, m.batch_id FROM stock_movements m JOIN product_packages pp ON pp.id=m.product_package_id JOIN products pr ON pr.id=pp.product_id JOIN inventory_batches b ON b.id=m.batch_id JOIN branches br ON br.id=m.branch_id";
pub fn list(db: &AppDb, q: &MovementQuery) -> Result<StockMovementPage, AppError> {
    let limit = q.limit.unwrap_or(50);
    let offset = q.offset.unwrap_or(0);
    if !(1..=200).contains(&limit) || !(0..=MAX_SAFE_MINOR).contains(&offset) {
        return Err(invalid("Use a limit of 1–200 and a non-negative offset."));
    }
    for value in [&q.branch_id, &q.package_id, &q.batch_id]
        .into_iter()
        .flatten()
    {
        id(value)?;
    }
    if let Some(kind) = &q.movement_type {
        if !MOVEMENT_TYPES.contains(&kind.as_str()) {
            return Err(invalid("Unknown movement type."));
        }
    }
    if q.search.as_ref().is_some_and(|v| v.chars().count() > 4000) {
        return Err(invalid("Search text is too long."));
    }
    let direction = match q.sort_direction.as_deref().map(str::trim) {
        None | Some("") | Some("desc") => "DESC",
        Some("asc") => "ASC",
        Some(other) => return Err(invalid(&format!("Unknown sort direction: {other}."))),
    };
    let order = match q.sort.as_deref().map(str::trim) {
        None | Some("") | Some("occurredAt") => "occurred_at",
        Some("movementType") => "movement_type",
        Some("product") => "product",
        Some(other) => return Err(invalid(&format!("Unknown sort key: {other}."))),
    };
    let search = normalize(q.search.as_deref().unwrap_or(""));
    let pattern = format!(
        "{}%",
        search
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    );
    let filter = r"WHERE (?1 IS NULL OR branch_id=?1) AND (?2 IS NULL OR product_package_id=?2) AND (?3 IS NULL OR batch_id=?3) AND (?4 IS NULL OR movement_type=?4) AND (?5='' OR product LIKE ?6 ESCAPE '\' OR package_label LIKE ?6 ESCAPE '\' OR batch_number LIKE ?6 ESCAPE '\' OR reason LIKE ?6 ESCAPE '\' OR id=?7)";
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let total = tx.query_row(
        &format!("SELECT count(*) FROM ({ITEM_SELECT}) {filter}"),
        params![
            q.branch_id,
            q.package_id,
            q.batch_id,
            q.movement_type,
            search,
            pattern,
            q.search.as_deref().unwrap_or("").trim()
        ],
        |r| r.get(0),
    )?;
    let items = {
        let mut s = tx.prepare(&format!(
            "SELECT * FROM ({ITEM_SELECT}) {filter} ORDER BY {order} {direction}, id LIMIT ?8 OFFSET ?9"
        ))?;
        let rows = s.query_map(
            params![
                q.branch_id,
                q.package_id,
                q.batch_id,
                q.movement_type,
                search,
                pattern,
                q.search.as_deref().unwrap_or("").trim(),
                limit,
                offset
            ],
            map_item,
        )?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    tx.commit()?;
    Ok(StockMovementPage {
        items,
        total,
        limit,
        offset,
    })
}
