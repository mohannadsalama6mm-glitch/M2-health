use crate::{
    db::{catalog_validation::*, connection::AppDb, models::catalog::*},
    error::AppError,
};
use rusqlite::{params, Connection};
pub fn link(db: &AppDb, input: &LinkProductIngredient) -> Result<Vec<ProductIngredient>, AppError> {
    id(&input.product_id)?;
    id(&input.active_ingredient_id)?;
    optional(&input.strength_text)?;
    if input.position < 0 || input.position > MAX_SAFE_MINOR {
        return Err(invalid(
            "Ingredient position must be a non-negative integer.",
        ));
    }
    let c = db.lock()?;
    c.execute("INSERT INTO product_active_ingredients(id,product_id,active_ingredient_id,strength_text,position) VALUES (?1,?2,?3,?4,?5)",params![uuid::Uuid::new_v4().to_string(),input.product_id,input.active_ingredient_id,input.strength_text,input.position])?;
    list_on(&c, &input.product_id)
}
pub(crate) fn list_on(
    c: &Connection,
    product_id: &str,
) -> Result<Vec<ProductIngredient>, AppError> {
    let mut s=c.prepare("SELECT id,active_ingredient_id,strength_text,position,created_at,updated_at FROM product_active_ingredients WHERE product_id=?1 ORDER BY position,id")?;
    let rows = s.query_map([product_id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, i64>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
        ))
    })?;
    rows.map(|r| {
        let (id, ingredient, strength_text, position, created_at, updated_at) = r?;
        Ok(ProductIngredient {
            id,
            product_id: product_id.into(),
            active_ingredient: super::active_ingredients::get(c, &ingredient)?,
            strength_text,
            position,
            created_at,
            updated_at,
        })
    })
    .collect()
}
/// Replaces the full ingredient set of a product inside a caller-owned transaction.
/// Duplicate ingredient inputs are rejected up front; positions follow input order.
pub(crate) fn replace_on(
    c: &Connection,
    product_id: &str,
    inputs: &[ProductIngredientInput],
) -> Result<(), AppError> {
    let mut seen = std::collections::HashSet::new();
    for input in inputs {
        id(&input.active_ingredient_id)?;
        optional(&input.strength_text)?;
        if !seen.insert(&input.active_ingredient_id) {
            return Err(invalid(
                "Each active ingredient may be linked to a product only once.",
            ));
        }
    }
    c.execute(
        "DELETE FROM product_active_ingredients WHERE product_id=?1",
        [product_id],
    )?;
    for (index, input) in inputs.iter().enumerate() {
        c.execute(
            "INSERT INTO product_active_ingredients(id,product_id,active_ingredient_id,strength_text,position) VALUES (?1,?2,?3,?4,?5)",
            params![uuid::Uuid::new_v4().to_string(), product_id, input.active_ingredient_id, input.strength_text, index as i64],
        )?;
    }
    Ok(())
}
