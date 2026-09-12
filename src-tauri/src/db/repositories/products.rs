use crate::{
    db::{catalog_validation::*, connection::AppDb, models::catalog::*},
    error::{AppError, ErrorCode},
};
use rusqlite::{params, Connection, OptionalExtension, Row};
const COLUMNS: &str = "id, commercial_name_en, commercial_name_ar, normalized_name_en, normalized_name_ar, scientific_name, normalized_scientific_name, manufacturer_id, category_id, route_id, description, notes, is_active, created_at, updated_at";
fn map(r: &Row<'_>) -> rusqlite::Result<Product> {
    Ok(Product {
        id: r.get(0)?,
        commercial_name_en: r.get(1)?,
        commercial_name_ar: r.get(2)?,
        normalized_name_en: r.get(3)?,
        normalized_name_ar: r.get(4)?,
        scientific_name: r.get(5)?,
        normalized_scientific_name: r.get(6)?,
        manufacturer_id: r.get(7)?,
        category_id: r.get(8)?,
        route_id: r.get(9)?,
        description: r.get(10)?,
        notes: r.get(11)?,
        is_active: r.get(12)?,
        created_at: r.get(13)?,
        updated_at: r.get(14)?,
    })
}
pub(crate) fn get(c: &Connection, value: &str) -> Result<Product, AppError> {
    c.query_row(
        &format!("SELECT {COLUMNS} FROM products WHERE id=?1"),
        [value],
        map,
    )
    .optional()?
    .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Product not found."))
}
pub fn create(db: &AppDb, input: &CreateProduct) -> Result<Product, AppError> {
    for text in [
        &input.commercial_name_en,
        &input.commercial_name_ar,
        &input.scientific_name,
        &input.description,
        &input.notes,
    ] {
        optional(text)?;
    }
    if input.commercial_name_en.is_none() && input.commercial_name_ar.is_none() {
        return Err(invalid("An English or Arabic commercial name is required."));
    }
    for value in [&input.manufacturer_id, &input.category_id, &input.route_id]
        .into_iter()
        .flatten()
    {
        id(value)?;
    }
    let c = db.lock()?;
    let value = uuid::Uuid::new_v4().to_string();
    c.execute("INSERT INTO products(id,commercial_name_en,commercial_name_ar,normalized_name_en,normalized_name_ar,scientific_name,normalized_scientific_name,manufacturer_id,category_id,route_id,description,notes) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",params![value,input.commercial_name_en,input.commercial_name_ar,input.commercial_name_en.as_deref().map(normalize),input.commercial_name_ar.as_deref().map(normalize),input.scientific_name,input.scientific_name.as_deref().map(normalize),input.manufacturer_id,input.category_id,input.route_id,input.description,input.notes])?;
    get(&c, &value)
}
pub fn get_product(db: &AppDb, value: &str) -> Result<Product, AppError> {
    id(value)?;
    let c = db.lock()?;
    get(&c, value)
}
pub fn set_active(db: &AppDb, value: &str, active: bool) -> Result<Product, AppError> {
    id(value)?;
    let c = db.lock()?;
    c.execute("UPDATE products SET is_active=?1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),julianday(updated_at)+0.001/86400)) WHERE id=?2",params![active,value])?;
    get(&c, value)
}
pub fn list(db: &AppDb, q: &ProductQuery) -> Result<ProductPage, AppError> {
    let limit = q.limit.unwrap_or(50);
    let offset = q.offset.unwrap_or(0);
    if !(1..=200).contains(&limit) || !(0..=MAX_SAFE_MINOR).contains(&offset) {
        return Err(invalid("Use a limit of 1–200 and a non-negative offset."));
    }
    for value in [&q.manufacturer_id, &q.category_id, &q.route_id]
        .into_iter()
        .flatten()
    {
        id(value)?;
    }
    if q.search.as_ref().is_some_and(|v| v.chars().count() > 4000) {
        return Err(invalid("Search text is too long."));
    }
    let search = normalize(q.search.as_deref().unwrap_or(""));
    let pattern = format!(
        "{}%",
        search
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    );
    // Prefix matching uses normalized-name indexes; barcode is an exact identifier match.
    let filter = r"WHERE (?1 OR is_active=1) AND (?2 IS NULL OR manufacturer_id=?2) AND (?3 IS NULL OR category_id=?3) AND (?4 IS NULL OR route_id=?4) AND (?5='' OR normalized_name_en LIKE ?6 ESCAPE '\' OR normalized_name_ar LIKE ?6 ESCAPE '\' OR normalized_scientific_name LIKE ?6 ESCAPE '\' OR manufacturer_id IN (SELECT id FROM manufacturers WHERE normalized_name LIKE ?6 ESCAPE '\') OR id IN (SELECT pp.product_id FROM product_packages pp JOIN barcodes b ON b.product_package_id=pp.id WHERE b.barcode=?7))";
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let args = params![
        q.include_inactive,
        q.manufacturer_id,
        q.category_id,
        q.route_id,
        search,
        pattern,
        q.search.as_deref().unwrap_or("").trim()
    ];
    let total = tx.query_row(
        &format!("SELECT count(*) FROM products {filter}"),
        args,
        |r| r.get(0),
    )?;
    let items = {
        let mut s=tx.prepare(&format!("SELECT {COLUMNS} FROM products {filter} ORDER BY coalesce(normalized_name_en,normalized_name_ar),id LIMIT ?8 OFFSET ?9"))?;
        let rows = s.query_map(
            params![
                q.include_inactive,
                q.manufacturer_id,
                q.category_id,
                q.route_id,
                search,
                pattern,
                q.search.as_deref().unwrap_or("").trim(),
                limit,
                offset
            ],
            map,
        )?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    tx.commit()?;
    Ok(ProductPage {
        items,
        total,
        limit,
        offset,
    })
}
pub fn detail(db: &AppDb, value: &str) -> Result<ProductDetail, AppError> {
    use super::{
        barcodes, categories, manufacturers, price_history, product_ingredients, product_packages,
        routes,
    };
    id(value)?;
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let product = get(&tx, value)?;
    let manufacturer = product
        .manufacturer_id
        .as_deref()
        .map(|id| manufacturers::get(&tx, id))
        .transpose()?;
    let category = product
        .category_id
        .as_deref()
        .map(|id| categories::get(&tx, id))
        .transpose()?;
    let route = product
        .route_id
        .as_deref()
        .map(|id| routes::get(&tx, id))
        .transpose()?;
    let active_ingredients = product_ingredients::list_on(&tx, value)?;
    let packages = product_packages::list_on(&tx, value, true)?
        .into_iter()
        .map(|package| {
            Ok(PackageDetail {
                barcodes: barcodes::list_on(&tx, &package.id)?,
                current_price: price_history::current_on(&tx, &package.id)?,
                package,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    tx.commit()?;
    Ok(ProductDetail {
        product,
        manufacturer,
        category,
        route,
        active_ingredients,
        packages,
    })
}
