import { nativeRequest } from "./client";
import type {
  ActiveIngredient,
  Category,
  Manufacturer,
  PackagePrice,
  Product,
  ProductDetail,
  ProductPage,
  ProductQuery,
  Route,
} from "./catalog.types";
import type { CreateProductFull, UpdateProductFull } from "./catalog.types";
export const listManufacturers = (includeInactive = false) =>
  nativeRequest<Manufacturer[]>("list_manufacturers", { includeInactive });
export const listCategories = (includeInactive = false) =>
  nativeRequest<Category[]>("list_categories", { includeInactive });
export const listRoutes = (includeInactive = false) =>
  nativeRequest<Route[]>("list_routes", { includeInactive });
export const listActiveIngredients = (includeInactive = false) =>
  nativeRequest<ActiveIngredient[]>("list_active_ingredients", {
    includeInactive,
  });
export const listProducts = (query: ProductQuery) =>
  nativeRequest<ProductPage>("list_products", { query });
export const getProduct = (id: string) =>
  nativeRequest<Product>("get_product", { id });
export const getProductDetail = (id: string) =>
  nativeRequest<ProductDetail>("get_product_detail", { id });
export const createProductFull = (input: CreateProductFull) =>
  nativeRequest<Product>("create_product_full", { input });
export const updateProductFull = (input: UpdateProductFull) =>
  nativeRequest<Product>("update_product_full", { input });
export const setProductActive = (id: string, active: boolean) =>
  nativeRequest<Product>("set_product_active", { id, active });
export const getPackagePriceHistory = (packageId: string) =>
  nativeRequest<PackagePrice[]>("get_package_price_history", { packageId });
/** Integer EGP minor units → major display amount (e.g. 3950 → 39.5). */
export const toMajor = (minor: number): number => minor / 100;
/** Major display amount (decimal string from an EGP input) → integer minor units. */
export const toMinor = (major: string): number | null => {
  if (major.trim() === "") return null;
  const value = Number(major);
  if (!Number.isFinite(value) || value < 0) return null;
  return Math.round(value * 100);
};
