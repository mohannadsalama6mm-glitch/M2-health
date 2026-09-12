use crate::{
    db::{catalog_validation::*, connection::AppDb, models::catalog::*},
    error::{AppError, ErrorCode},
};
use rusqlite::{params, Connection, OptionalExtension, Row, TransactionBehavior};
use std::collections::HashSet;
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
fn validate_name_fields(
    commercial_name_en: &Option<String>,
    commercial_name_ar: &Option<String>,
    scientific_name: &Option<String>,
    manufacturer_id: &Option<String>,
    category_id: &Option<String>,
    route_id: &Option<String>,
    description: &Option<String>,
    notes: &Option<String>,
) -> Result<(), AppError> {
    for text in [
        commercial_name_en,
        commercial_name_ar,
        scientific_name,
        description,
        notes,
    ] {
        optional(text)?;
    }
    if commercial_name_en.is_none() && commercial_name_ar.is_none() {
        return Err(invalid("An English or Arabic commercial name is required."));
    }
    for value in [manufacturer_id, category_id, route_id]
        .into_iter()
        .flatten()
    {
        id(value)?;
    }
    Ok(())
}
/// Validates the package rows entered by the add/edit forms and returns the
/// normalized barcode values so the caller can run the global conflict check.
fn validate_package_inputs(packages: &[ProductPackageInput]) -> Result<Vec<String>, AppError> {
    let mut package_ids = HashSet::new();
    let mut defaults = 0usize;
    let mut seen_barcodes = HashSet::new();
    let mut validated = Vec::new();
    for package in packages {
        required(&package.package_label)?;
        for text in [
            &package.pack_size,
            &package.unit_name,
            &package.strength_text,
        ] {
            optional(text)?;
        }
        if package
            .units_per_package
            .is_some_and(|v| v <= 0 || v > MAX_SAFE_MINOR)
        {
            return Err(invalid("Units per package must be a positive integer."));
        }
        if let Some(id) = &package.id {
            if !package_ids.insert(id) {
                return Err(invalid("Each package may appear only once."));
            }
        }
        if package.is_default {
            defaults += 1;
            if !package.is_active {
                return Err(invalid("The default package must be active."));
            }
        }
        if package.barcodes.iter().filter(|b| b.is_primary).count() > 1 {
            return Err(invalid("A package can have at most one primary barcode."));
        }
        if package
            .selling_price_minor
            .is_some_and(|v| v < 0 || v > MAX_SAFE_MINOR)
            || package
                .cost_price_minor
                .is_some_and(|v| v < 0 || v > MAX_SAFE_MINOR)
        {
            return Err(invalid(
                "Prices must be non-negative integer minor units within the supported range.",
            ));
        }
        for barcode in &package.barcodes {
            let value = super::barcodes::validate(&barcode.barcode)?;
            if !seen_barcodes.insert(value.clone()) {
                return Err(invalid("Each barcode may be used only once per product."));
            }
            validated.push(value);
        }
    }
    if defaults > 1 {
        return Err(invalid("At most one package can be the default."));
    }
    Ok(validated)
}
/// Builds the enriched catalog grid row for one product. N+1 lookups are fine at
/// this row/limit scale; React never joins this data.
fn enrich(c: &Connection, product: &Product) -> Result<ProductListItem, AppError> {
    use super::{barcodes, categories, manufacturers, price_history, product_packages, routes};
    let manufacturer_name = product
        .manufacturer_id
        .as_deref()
        .map(|m| manufacturers::get(c, m).map(|v| v.name))
        .transpose()?;
    let category_name = product
        .category_id
        .as_deref()
        .map(|v| categories::get(c, v).map(|v| v.name))
        .transpose()?;
    let route_name = product
        .route_id
        .as_deref()
        .map(|v| routes::get(c, v).map(|v| v.name))
        .transpose()?;
    let package = product_packages::default_on(c, &product.id)?;
    let barcode = package
        .as_ref()
        .map(|p| barcodes::first_on(c, &p.id))
        .transpose()?
        .flatten()
        .map(|b| b.barcode);
    let current = package
        .as_ref()
        .map(|p| price_history::current_on(c, &p.id))
        .transpose()?
        .flatten();
    Ok(ProductListItem {
        id: product.id.clone(),
        name_en: product.commercial_name_en.clone(),
        name_ar: product.commercial_name_ar.clone(),
        scientific_name: product.scientific_name.clone(),
        manufacturer_name,
        category_name,
        route_name,
        is_active: product.is_active,
        package_label: package.as_ref().map(|p| p.package_label.clone()),
        pack_size: package.as_ref().and_then(|p| p.pack_size.clone()),
        barcode,
        selling_price_minor: current.as_ref().map(|p| p.selling_price_minor),
        cost_price_minor: current.as_ref().and_then(|p| p.cost_price_minor),
        package_count: product_packages::count_on(c, &product.id)?,
        barcode_count: barcodes::count_product_on(c, &product.id)?,
        created_at: product.created_at.clone(),
        updated_at: product.updated_at.clone(),
    })
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
    let direction = match q.sort_direction.as_deref().map(str::trim) {
        None | Some("") | Some("asc") => "ASC",
        Some("desc") => "DESC",
        Some(other) => return Err(invalid(&format!("Unknown sort direction: {other}."))),
    };
    let order = match q.sort.as_deref().map(str::trim) {
        None | Some("") | Some("name") => "coalesce(p.normalized_name_en,p.normalized_name_ar)",
        Some("scientific") => "p.normalized_scientific_name",
        Some("manufacturer") => {
            "(SELECT normalized_name FROM manufacturers m WHERE m.id=p.manufacturer_id)"
        }
        Some("category") => {
            "(SELECT normalized_name FROM categories c WHERE c.id=p.category_id)"
        }
        Some("route") => "(SELECT normalized_name FROM routes r WHERE r.id=p.route_id)",
        Some("active") => "p.is_active",
        Some("price") => "(SELECT ph.selling_price_minor FROM product_price_history ph JOIN product_packages pp ON pp.id=ph.product_package_id WHERE pp.product_id=p.id AND pp.is_default=1 AND ph.effective_to IS NULL LIMIT 1)",
        Some("createdAt") => "p.created_at",
        Some("updatedAt") => "p.updated_at",
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
    // Prefix matching uses normalized-name indexes; barcode is an exact identifier match.
    let filter = r"WHERE (?1 OR p.is_active=1) AND (?2 IS NULL OR p.manufacturer_id=?2) AND (?3 IS NULL OR p.category_id=?3) AND (?4 IS NULL OR p.route_id=?4) AND (?5='' OR p.normalized_name_en LIKE ?6 ESCAPE '\' OR p.normalized_name_ar LIKE ?6 ESCAPE '\' OR p.normalized_scientific_name LIKE ?6 ESCAPE '\' OR p.manufacturer_id IN (SELECT id FROM manufacturers WHERE normalized_name LIKE ?6 ESCAPE '\') OR p.id IN (SELECT pp.product_id FROM product_packages pp JOIN barcodes b ON b.product_package_id=pp.id WHERE b.barcode=?7))";
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let total = tx.query_row(
        &format!("SELECT count(*) FROM products p {filter}"),
        params![
            q.include_inactive,
            q.manufacturer_id,
            q.category_id,
            q.route_id,
            search,
            pattern,
            q.search.as_deref().unwrap_or("").trim()
        ],
        |r| r.get(0),
    )?;
    let active_total = tx.query_row(
        &format!("SELECT count(*) FROM products p {filter}"),
        params![
            false,
            q.manufacturer_id,
            q.category_id,
            q.route_id,
            search,
            pattern,
            q.search.as_deref().unwrap_or("").trim()
        ],
        |r| r.get(0),
    )?;
    let items = {
        let mut s = tx.prepare(&format!(
            "SELECT {COLUMNS} FROM products p {filter} ORDER BY {order} {direction}, p.id LIMIT ?8 OFFSET ?9"
        ))?;
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
        let rows: Vec<Product> = rows.collect::<Result<Vec<_>, _>>()?;
        rows.iter()
            .map(|product| enrich(&tx, product))
            .collect::<Result<Vec<_>, _>>()?
    };
    tx.commit()?;
    Ok(ProductPage {
        items,
        total,
        active_total,
        limit,
        offset,
    })
}
/// Atomic product creation: identity, ingredient links, packages, barcodes and the
/// initial price history row are committed in a single transaction.
pub fn create_full(db: &AppDb, input: &CreateProductFull) -> Result<Product, AppError> {
    validate_name_fields(
        &input.commercial_name_en,
        &input.commercial_name_ar,
        &input.scientific_name,
        &input.manufacturer_id,
        &input.category_id,
        &input.route_id,
        &input.description,
        &input.notes,
    )?;
    let barcodes = validate_package_inputs(&input.packages)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    for ingredient in &input.active_ingredients {
        super::active_ingredients::get(&tx, &ingredient.active_ingredient_id)?;
    }
    let conflicts = super::barcodes::conflicts_on(&tx, "", &barcodes)?;
    if !conflicts.is_empty() {
        return Err(AppError::new(
            ErrorCode::Conflict,
            &format!("Barcode already in use: {}.", conflicts.join(", ")),
        ));
    }
    let value = uuid::Uuid::new_v4().to_string();
    tx.execute("INSERT INTO products(id,commercial_name_en,commercial_name_ar,normalized_name_en,normalized_name_ar,scientific_name,normalized_scientific_name,manufacturer_id,category_id,route_id,description,notes) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",params![value,input.commercial_name_en,input.commercial_name_ar,input.commercial_name_en.as_deref().map(normalize),input.commercial_name_ar.as_deref().map(normalize),input.scientific_name,input.scientific_name.as_deref().map(normalize),input.manufacturer_id,input.category_id,input.route_id,input.description,input.notes])?;
    super::product_ingredients::replace_on(&tx, &value, &input.active_ingredients)?;
    for package in &input.packages {
        let pkg = super::product_packages::create_on(
            &tx,
            &CreateProductPackage {
                product_id: value.clone(),
                package_label: package.package_label.clone(),
                pack_size: package.pack_size.clone(),
                unit_name: package.unit_name.clone(),
                units_per_package: package.units_per_package,
                strength_text: package.strength_text.clone(),
                is_default: package.is_default,
            },
        )?;
        if !package.barcodes.is_empty() {
            super::barcodes::insert_all_on(&tx, &pkg.id, &package.barcodes)?;
        }
        if package.selling_price_minor.is_some() || package.cost_price_minor.is_some() {
            super::price_history::set_checked_on(
                &tx,
                &SetPackagePrice {
                    product_package_id: pkg.id,
                    selling_price_minor: package.selling_price_minor.unwrap_or(0),
                    cost_price_minor: package.cost_price_minor,
                    reason: Some("Initial price".into()),
                },
            )?;
        }
    }
    let product = get(&tx, &value)?;
    tx.commit()?;
    Ok(product)
}
/// Atomic product update: identity/classification fields, ingredient set replacement,
/// barcode reconciliation and append-only price changes in one transaction. Packages
/// present in the input are created or updated; packages not listed keep their price
/// history but are not deleted.
pub fn update_full(db: &AppDb, input: &UpdateProductFull) -> Result<Product, AppError> {
    id(&input.id)?;
    validate_name_fields(
        &input.commercial_name_en,
        &input.commercial_name_ar,
        &input.scientific_name,
        &input.manufacturer_id,
        &input.category_id,
        &input.route_id,
        &input.description,
        &input.notes,
    )?;
    let barcodes = validate_package_inputs(&input.packages)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    get(&tx, &input.id)?;
    for ingredient in &input.active_ingredients {
        super::active_ingredients::get(&tx, &ingredient.active_ingredient_id)?;
    }
    let conflicts = super::barcodes::conflicts_on(&tx, &input.id, &barcodes)?;
    if !conflicts.is_empty() {
        return Err(AppError::new(
            ErrorCode::Conflict,
            &format!("Barcode already in use: {}.", conflicts.join(", ")),
        ));
    }
    tx.execute("UPDATE products SET commercial_name_en=?1,commercial_name_ar=?2,normalized_name_en=?3,normalized_name_ar=?4,scientific_name=?5,normalized_scientific_name=?6,manufacturer_id=?7,category_id=?8,route_id=?9,description=?10,notes=?11,is_active=?12,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),julianday(updated_at)+0.001/86400)) WHERE id=?13",params![input.commercial_name_en,input.commercial_name_ar,input.commercial_name_en.as_deref().map(normalize),input.commercial_name_ar.as_deref().map(normalize),input.scientific_name,input.scientific_name.as_deref().map(normalize),input.manufacturer_id,input.category_id,input.route_id,input.description,input.notes,input.is_active,input.id])?;
    super::product_ingredients::replace_on(&tx, &input.id, &input.active_ingredients)?;
    super::barcodes::wipe_product_on(&tx, &input.id)?;
    for package in &input.packages {
        let pkg = match &package.id {
            Some(package_id) => {
                let existing = super::product_packages::get(&tx, package_id)?;
                if existing.product_id != input.id {
                    return Err(invalid(
                        "A package can only be edited on the product it belongs to.",
                    ));
                }
                super::product_packages::update_on(
                    &tx,
                    package_id,
                    &super::product_packages::PackageEdit {
                        package_label: package.package_label.clone(),
                        pack_size: package.pack_size.clone(),
                        unit_name: package.unit_name.clone(),
                        units_per_package: package.units_per_package,
                        strength_text: package.strength_text.clone(),
                        is_active: package.is_active,
                        is_default: package.is_default,
                    },
                )?
            }
            None => super::product_packages::create_on(
                &tx,
                &CreateProductPackage {
                    product_id: input.id.clone(),
                    package_label: package.package_label.clone(),
                    pack_size: package.pack_size.clone(),
                    unit_name: package.unit_name.clone(),
                    units_per_package: package.units_per_package,
                    strength_text: package.strength_text.clone(),
                    is_default: package.is_default,
                },
            )?,
        };
        if !package.barcodes.is_empty() {
            super::barcodes::insert_all_on(&tx, &pkg.id, &package.barcodes)?;
        }
        if package.selling_price_minor.is_some() || package.cost_price_minor.is_some() {
            super::price_history::set_checked_on(
                &tx,
                &SetPackagePrice {
                    product_package_id: pkg.id,
                    selling_price_minor: package.selling_price_minor.unwrap_or(0),
                    cost_price_minor: package.cost_price_minor,
                    reason: None,
                },
            )?;
        }
    }
    let product = get(&tx, &input.id)?;
    tx.commit()?;
    Ok(product)
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
