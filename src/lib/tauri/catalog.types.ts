// Phase 2B DTO contracts only. Catalog screens remain demo-based. Money is integer EGP minor units.
export interface Manufacturer {
  id: string;
  name: string;
  normalizedName: string;
  country: string | null;
  phone: string | null;
  email: string | null;
  website: string | null;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}
export interface CreateManufacturer {
  name: string;
  country?: string | null;
  phone?: string | null;
  email?: string | null;
  website?: string | null;
}
export interface Category {
  id: string;
  name: string;
  normalizedName: string;
  description: string | null;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}
export interface CreateCategory {
  name: string;
  description?: string | null;
}
export interface Route {
  id: string;
  name: string;
  normalizedName: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}
export interface CreateRoute {
  name: string;
}
export interface ActiveIngredient {
  id: string;
  name: string;
  normalizedName: string;
  description: string | null;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}
export interface CreateActiveIngredient {
  name: string;
  description?: string | null;
}
export interface Product {
  id: string;
  commercialNameEn: string | null;
  commercialNameAr: string | null;
  normalizedNameEn: string | null;
  normalizedNameAr: string | null;
  scientificName: string | null;
  normalizedScientificName: string | null;
  manufacturerId: string | null;
  categoryId: string | null;
  routeId: string | null;
  description: string | null;
  notes: string | null;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}
export interface CreateProduct {
  commercialNameEn?: string | null;
  commercialNameAr?: string | null;
  scientificName?: string | null;
  manufacturerId?: string | null;
  categoryId?: string | null;
  routeId?: string | null;
  description?: string | null;
  notes?: string | null;
}
export interface ProductPackage {
  id: string;
  productId: string;
  packageLabel: string;
  packSize: string | null;
  unitName: string | null;
  unitsPerPackage: number | null;
  strengthText: string | null;
  isDefault: boolean;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}
export interface CreateProductPackage {
  productId: string;
  packageLabel: string;
  packSize?: string | null;
  unitName?: string | null;
  unitsPerPackage?: number | null;
  strengthText?: string | null;
  isDefault?: boolean;
}
export interface Barcode {
  id: string;
  productPackageId: string;
  barcode: string;
  isPrimary: boolean;
  createdAt: string;
  updatedAt: string;
}
export interface AddBarcode {
  productPackageId: string;
  barcode: string;
  isPrimary?: boolean;
}
export interface PackagePrice {
  id: string;
  productPackageId: string;
  sellingPriceMinor: number;
  costPriceMinor: number | null;
  effectiveFrom: string;
  effectiveTo: string | null;
  reason: string | null;
  createdAt: string;
}
export interface SetPackagePrice {
  productPackageId: string;
  sellingPriceMinor: number;
  costPriceMinor?: number | null;
  reason?: string | null;
}
export interface LinkProductIngredient {
  productId: string;
  activeIngredientId: string;
  strengthText?: string | null;
  position?: number;
}
export interface ProductIngredient {
  id: string;
  productId: string;
  activeIngredient: ActiveIngredient;
  strengthText: string | null;
  position: number;
  createdAt: string;
  updatedAt: string;
}
export interface PackageDetail {
  package: ProductPackage;
  barcodes: Barcode[];
  currentPrice: PackagePrice | null;
}
export interface ProductDetail {
  product: Product;
  manufacturer: Manufacturer | null;
  category: Category | null;
  route: Route | null;
  activeIngredients: ProductIngredient[];
  packages: PackageDetail[];
}
export type CatalogSortKey =
  | "name"
  | "scientific"
  | "manufacturer"
  | "category"
  | "route"
  | "active"
  | "price"
  | "createdAt"
  | "updatedAt";
export interface ProductQuery {
  search?: string | null;
  manufacturerId?: string | null;
  categoryId?: string | null;
  routeId?: string | null;
  includeInactive?: boolean;
  sort?: CatalogSortKey | null;
  sortDirection?: "asc" | "desc" | null;
  limit?: number | null;
  offset?: number | null;
}
export interface ProductListItem {
  id: string;
  nameEn: string | null;
  nameAr: string | null;
  scientificName: string | null;
  manufacturerName: string | null;
  categoryName: string | null;
  routeName: string | null;
  isActive: boolean;
  packageLabel: string | null;
  packSize: string | null;
  barcode: string | null;
  sellingPriceMinor: number | null;
  costPriceMinor: number | null;
  packageCount: number;
  barcodeCount: number;
  createdAt: string;
  updatedAt: string;
}
export interface ProductPage {
  items: ProductListItem[];
  total: number;
  activeTotal: number;
  limit: number;
  offset: number;
}
export interface ProductIngredientInput {
  activeIngredientId: string;
  strengthText?: string | null;
}
export interface ProductBarcodeInput {
  barcode: string;
  isPrimary?: boolean;
}
export interface ProductPackageInput {
  id?: string | null;
  packageLabel: string;
  packSize?: string | null;
  unitName?: string | null;
  unitsPerPackage?: number | null;
  strengthText?: string | null;
  isDefault?: boolean;
  isActive?: boolean;
  barcodes?: ProductBarcodeInput[];
  sellingPriceMinor?: number | null;
  costPriceMinor?: number | null;
}
export interface CreateProductFull {
  commercialNameEn?: string | null;
  commercialNameAr?: string | null;
  scientificName?: string | null;
  manufacturerId?: string | null;
  categoryId?: string | null;
  routeId?: string | null;
  description?: string | null;
  notes?: string | null;
  activeIngredients?: ProductIngredientInput[];
  packages?: ProductPackageInput[];
}
export interface UpdateProductFull extends CreateProductFull {
  id: string;
  isActive: boolean;
}
