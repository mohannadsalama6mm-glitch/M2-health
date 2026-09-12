use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Manufacturer {
    pub id: String,
    pub name: String,
    pub normalized_name: String,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateManufacturer {
    pub name: String,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub name: String,
    pub normalized_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateCategory {
    pub name: String,
    pub description: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    pub id: String,
    pub name: String,
    pub normalized_name: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateRoute {
    pub name: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveIngredient {
    pub id: String,
    pub name: String,
    pub normalized_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateActiveIngredient {
    pub name: String,
    pub description: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: String,
    pub commercial_name_en: Option<String>,
    pub commercial_name_ar: Option<String>,
    pub normalized_name_en: Option<String>,
    pub normalized_name_ar: Option<String>,
    pub scientific_name: Option<String>,
    pub normalized_scientific_name: Option<String>,
    pub manufacturer_id: Option<String>,
    pub category_id: Option<String>,
    pub route_id: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateProduct {
    pub commercial_name_en: Option<String>,
    pub commercial_name_ar: Option<String>,
    pub scientific_name: Option<String>,
    pub manufacturer_id: Option<String>,
    pub category_id: Option<String>,
    pub route_id: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductPackage {
    pub id: String,
    pub product_id: String,
    pub package_label: String,
    pub pack_size: Option<String>,
    pub unit_name: Option<String>,
    pub units_per_package: Option<i64>,
    pub strength_text: Option<String>,
    #[serde(default)]
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateProductPackage {
    pub product_id: String,
    pub package_label: String,
    pub pack_size: Option<String>,
    pub unit_name: Option<String>,
    pub units_per_package: Option<i64>,
    pub strength_text: Option<String>,
    #[serde(default)]
    pub is_default: bool,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Barcode {
    pub id: String,
    pub product_package_id: String,
    pub barcode: String,
    #[serde(default)]
    pub is_primary: bool,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AddBarcode {
    pub product_package_id: String,
    pub barcode: String,
    #[serde(default)]
    pub is_primary: bool,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackagePrice {
    pub id: String,
    pub product_package_id: String,
    pub selling_price_minor: i64,
    pub cost_price_minor: Option<i64>,
    pub effective_from: String,
    pub effective_to: Option<String>,
    pub reason: Option<String>,
    pub created_at: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetPackagePrice {
    pub product_package_id: String,
    pub selling_price_minor: i64,
    pub cost_price_minor: Option<i64>,
    pub reason: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LinkProductIngredient {
    pub product_id: String,
    pub active_ingredient_id: String,
    pub strength_text: Option<String>,
    #[serde(default)]
    pub position: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductIngredient {
    pub id: String,
    pub product_id: String,
    pub active_ingredient: ActiveIngredient,
    pub strength_text: Option<String>,
    #[serde(default)]
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageDetail {
    pub package: ProductPackage,
    pub barcodes: Vec<Barcode>,
    pub current_price: Option<PackagePrice>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductDetail {
    pub product: Product,
    pub manufacturer: Option<Manufacturer>,
    pub category: Option<Category>,
    pub route: Option<Route>,
    pub active_ingredients: Vec<ProductIngredient>,
    pub packages: Vec<PackageDetail>,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductQuery {
    pub search: Option<String>,
    pub manufacturer_id: Option<String>,
    pub category_id: Option<String>,
    pub route_id: Option<String>,
    #[serde(default)]
    pub include_inactive: bool,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductPage {
    pub items: Vec<Product>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
