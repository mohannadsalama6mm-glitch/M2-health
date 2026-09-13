use crate::{
    db::{connection::AppDb, models::inventory::*, repositories::settings as repo},
    error::AppError,
};
/// Branch-aware reorder configuration. Absent settings default to 10 at query time.
pub fn set_reorder(db: &AppDb, input: &SetReorderInput) -> Result<InventorySetting, AppError> {
    repo::set_reorder(db, input)
}
