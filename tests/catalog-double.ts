import { type Page } from "@playwright/test";

/**
 * Browser-only IPC double for the real catalog commands. It mirrors the exact
 * camelCase wire contracts of the Rust commands (`models/catalog.rs`) with an
 * in-memory store that survives SPA navigation. Never used by production code.
 */
export function installCatalogDouble(page: Page) {
  return page.addInitScript(() => {
    interface Branch {
      id: string;
      code: string;
      name: string;
      phone: string;
      address: string;
      isActive: boolean;
      createdAt: string;
      updatedAt: string;
    }
    interface Manufacturer {
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
    interface Category {
      id: string;
      name: string;
      normalizedName: string;
      description: string | null;
      isActive: boolean;
      createdAt: string;
      updatedAt: string;
    }
    interface Route {
      id: string;
      name: string;
      normalizedName: string;
      isActive: boolean;
      createdAt: string;
      updatedAt: string;
    }
    interface ActiveIngredient {
      id: string;
      name: string;
      normalizedName: string;
      description: string | null;
      isActive: boolean;
      createdAt: string;
      updatedAt: string;
    }
    interface Store {
      branches: Branch[];
      manufacturers: Manufacturer[];
      categories: Category[];
      routes: Route[];
      ingredients: ActiveIngredient[];
      products: ProductDetail[];
      priceSeq: number;
    }
    interface Product {
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
    interface ProductDetail {
      product: Product;
      manufacturer: Manufacturer | null;
      category: Category | null;
      route: Route | null;
      activeIngredients: {
        id: string;
        productId: string;
        activeIngredient: ActiveIngredient;
        strengthText: string | null;
        position: number;
        createdAt: string;
        updatedAt: string;
      }[];
      packages: {
        package: {
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
        };
        barcodes: {
          id: string;
          productPackageId: string;
          barcode: string;
          isPrimary: boolean;
          createdAt: string;
          updatedAt: string;
        }[];
        currentPrice:
          | {
              id: string;
              productPackageId: string;
              sellingPriceMinor: number;
              costPriceMinor: number | null;
              effectiveFrom: string;
              effectiveTo: string | null;
              reason: string | null;
              createdAt: string;
            }
          | null;
      }[];
    }
    interface ProductInput {
      commercialNameEn?: string | null;
      commercialNameAr?: string | null;
      scientificName?: string | null;
      manufacturerId?: string | null;
      categoryId?: string | null;
      routeId?: string | null;
      description?: string | null;
      notes?: string | null;
      isActive?: boolean;
      activeIngredients?: { activeIngredientId: string; strengthText?: string | null }[];
      packages?: {
        id?: string | null;
        packageLabel: string;
        packSize?: string | null;
        unitName?: string | null;
        unitsPerPackage?: number | null;
        strengthText?: string | null;
        isDefault?: boolean;
        isActive?: boolean;
        barcodes?: { barcode: string; isPrimary?: boolean }[];
        sellingPriceMinor?: number | null;
        costPriceMinor?: number | null;
      }[];
    }
    const normalize = (value: string) =>
      value.trim().toLowerCase().split(/\s+/).join(" ");
    const now = () => new Date().toISOString();
    const uuid = (prefix: string) => `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    const windowObj = window as unknown as { __catalogStore?: Store };
    if (!windowObj.__catalogStore) {
      const branch: Branch = {
        id: "424b25b8-3c75-4d7a-87cd-988397081194",
        code: "MAIN",
        name: "Main Pharmacy",
        phone: "",
        address: "",
        isActive: true,
        createdAt: "2026-09-12T00:00:00.000Z",
        updatedAt: "2026-09-12T00:00:00.000Z",
      };
      const haleon: Manufacturer = {
        id: "m-1",
        name: "Haleon",
        normalizedName: "haleon",
        country: "UK",
        phone: null,
        email: null,
        website: null,
        isActive: true,
        createdAt: "2026-01-01T00:00:00.000Z",
        updatedAt: "2026-01-01T00:00:00.000Z",
      };
      const category: Category = {
        id: "c-1",
        name: "Pain relief",
        normalizedName: "pain relief",
        description: null,
        isActive: true,
        createdAt: "2026-01-01T00:00:00.000Z",
        updatedAt: "2026-01-01T00:00:00.000Z",
      };
      const route: Route = {
        id: "r-1",
        name: "Oral",
        normalizedName: "oral",
        isActive: true,
        createdAt: "2026-01-01T00:00:00.000Z",
        updatedAt: "2026-01-01T00:00:00.000Z",
      };
      const paracetamol: ActiveIngredient = {
        id: "i-1",
        name: "Paracetamol",
        normalizedName: "paracetamol",
        description: null,
        isActive: true,
        createdAt: "2026-01-01T00:00:00.000Z",
        updatedAt: "2026-01-01T00:00:00.000Z",
      };
      const demoIngredient: ActiveIngredient = {
        id: "i-2",
        name: "Demo second ingredient",
        normalizedName: "demo second ingredient",
        description: null,
        isActive: true,
        createdAt: "2026-01-01T00:00:00.000Z",
        updatedAt: "2026-01-01T00:00:00.000Z",
      };
      const product: Product = {
        id: "p-1",
        commercialNameEn: "Panadol 500 mg",
        commercialNameAr: null,
        normalizedNameEn: "panadol 500 mg",
        normalizedNameAr: null,
        scientificName: "Paracetamol",
        normalizedScientificName: "paracetamol",
        manufacturerId: "m-1",
        categoryId: "c-1",
        routeId: "r-1",
        description: null,
        notes: "Test seed product",
        isActive: true,
        createdAt: "2026-09-01T00:00:00.000Z",
        updatedAt: "2026-09-01T00:00:00.000Z",
      };
      const packageRow = {
        id: "pkg-1",
        productId: "p-1",
        packageLabel: "Box · 10 tablets",
        packSize: "10 tablets",
        unitName: "tablet",
        unitsPerPackage: 10,
        strengthText: "500 mg",
        isDefault: true,
        isActive: true,
        createdAt: "2026-09-01T00:00:00.000Z",
        updatedAt: "2026-09-01T00:00:00.000Z",
      };
      const detail: ProductDetail = {
        product,
        manufacturer: haleon,
        category,
        route,
        activeIngredients: [
          {
            id: "pi-1",
            productId: "p-1",
            activeIngredient: paracetamol,
            strengthText: "500 mg",
            position: 0,
            createdAt: "2026-09-01T00:00:00.000Z",
            updatedAt: "2026-09-01T00:00:00.000Z",
          },
        ],
        packages: [
          {
            package: packageRow,
            barcodes: [
              {
                id: "b-1",
                productPackageId: "pkg-1",
                barcode: "6221001000011",
                isPrimary: true,
                createdAt: "2026-09-01T00:00:00.000Z",
                updatedAt: "2026-09-01T00:00:00.000Z",
              },
            ],
            currentPrice: {
              id: "pr-1",
              productPackageId: "pkg-1",
              sellingPriceMinor: 3850,
              costPriceMinor: 2100,
              effectiveFrom: "2026-09-01T00:00:00.000Z",
              effectiveTo: null,
              reason: "Initial price",
              createdAt: "2026-09-01T00:00:00.000Z",
            },
          },
        ],
      };
      windowObj.__catalogStore = {
        branches: [branch],
        manufacturers: [haleon],
        categories: [category],
        routes: [route],
        ingredients: [paracetamol, demoIngredient],
        products: [detail],
        priceSeq: 1,
      };
    }
    const store = windowObj.__catalogStore;
    const findManufacturer = (id: string | null) =>
      store.manufacturers.find((m) => m.id === id) ?? null;
    const findCategory = (id: string | null) =>
      store.categories.find((c) => c.id === id) ?? null;
    const findRoute = (id: string | null) =>
      store.routes.find((r) => r.id === id) ?? null;
    const findIngredient = (id: string) =>
      store.ingredients.find((i) => i.id === id) ?? null;
    const findDetail = (id: string) =>
      store.products.find((d) => d.product.id === id) ?? null;
    const enrichListItems = () =>
      store.products.map((d) => {
        const p = d.product;
        const entry = d.packages[0];
        return {
          id: p.id,
          nameEn: p.commercialNameEn,
          nameAr: p.commercialNameAr,
          scientificName: p.scientificName,
          manufacturerName: d.manufacturer?.name ?? null,
          categoryName: d.category?.name ?? null,
          routeName: d.route?.name ?? null,
          isActive: p.isActive,
          packageLabel: entry?.package.packageLabel ?? null,
          packSize: entry?.package.packSize ?? null,
          barcode: entry?.barcodes[0]?.barcode ?? null,
          sellingPriceMinor: entry?.currentPrice?.sellingPriceMinor ?? null,
          costPriceMinor: entry?.currentPrice?.costPriceMinor ?? null,
          packageCount: d.packages.length,
          barcodeCount: d.packages.reduce((n, x) => n + x.barcodes.length, 0),
          createdAt: p.createdAt,
          updatedAt: p.updatedAt,
        };
      });
    const toProduct = (input: ProductInput): Product => ({
      id: "p-new",
      commercialNameEn: input.commercialNameEn ?? null,
      commercialNameAr: input.commercialNameAr ?? null,
      normalizedNameEn: normalize(input.commercialNameEn ?? ""),
      normalizedNameAr: normalize(input.commercialNameAr ?? ""),
      scientificName: input.scientificName ?? null,
      normalizedScientificName: normalize(input.scientificName ?? ""),
      manufacturerId: input.manufacturerId ?? null,
      categoryId: input.categoryId ?? null,
      routeId: input.routeId ?? null,
      description: input.description ?? null,
      notes: input.notes ?? null,
      isActive: true,
      createdAt: now(),
      updatedAt: now(),
    });
    const buildDetail = (
      product: Product,
      input: ProductInput,
      existing?: ProductDetail,
    ): ProductDetail => {
      const created = now();
      const activeIngredients = (input.activeIngredients ?? []).map((ing, idx) => {
        const ingredient = findIngredient(ing.activeIngredientId);
        return {
          id: ingredient ? uuid("pi") : uuid("pi"),
          productId: product.id,
          activeIngredient:
            ingredient ??
            ({
              id: ing.activeIngredientId,
              name: ing.activeIngredientId,
              normalizedName: ing.activeIngredientId,
              description: null,
              isActive: true,
              createdAt: now(),
              updatedAt: now(),
            } as ActiveIngredient),
          strengthText: ing.strengthText ?? null,
          position: idx,
          createdAt: created,
          updatedAt: created,
        };
      });
      const packages = (input.packages ?? []).map((row, idx) => {
        const prior =
          row.id && existing
            ? existing.packages.find((x) => x.package.id === row.id)
            : undefined;
        const packageRow = prior
          ? {
              ...prior.package,
              packageLabel: row.packageLabel,
              packSize: row.packSize ?? null,
              unitName: row.unitName ?? null,
              unitsPerPackage: row.unitsPerPackage ?? null,
              strengthText: row.strengthText ?? null,
              isDefault: row.isDefault ?? false,
              isActive: row.isActive ?? true,
              updatedAt: now(),
            }
          : {
              id: uuid("pkg"),
              productId: product.id,
              packageLabel: row.packageLabel,
              packSize: row.packSize ?? null,
              unitName: row.unitName ?? null,
              unitsPerPackage: row.unitsPerPackage ?? null,
              strengthText: row.strengthText ?? null,
              isDefault: row.isDefault ?? idx === 0,
              isActive: row.isActive ?? true,
              createdAt: now(),
              updatedAt: now(),
            };
        const barcodes = (row.barcodes ?? []).map((b, bIdx) => ({
          id: uuid("b"),
          productPackageId: packageRow.id,
          barcode: b.barcode,
          isPrimary: b.isPrimary ?? bIdx === 0,
          createdAt: now(),
          updatedAt: now(),
        }));
        const priceChange =
          row.sellingPriceMinor != null || row.costPriceMinor != null;
        const currentPrice = priceChange
          ? {
              id: uuid("pr"),
              productPackageId: packageRow.id,
              sellingPriceMinor: row.sellingPriceMinor ?? 0,
              costPriceMinor: row.costPriceMinor ?? null,
              effectiveFrom: now(),
              effectiveTo: null,
              reason: null,
              createdAt: now(),
            }
          : prior?.currentPrice ?? null;
        return { package: packageRow, barcodes, currentPrice };
      });
      return {
        product,
        manufacturer: findManufacturer(product.manufacturerId),
        category: findCategory(product.categoryId),
        route: findRoute(product.routeId),
        activeIngredients,
        packages,
      };
    };
    const branchResult = store.branches[0];
    Object.assign(window, {
      isTauri: true,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown> = {}) => {
          switch (command) {
            case "ensure_default_branch":
              return branchResult;
            case "list_branches":
              return store.branches;
            case "list_manufacturers":
              return store.manufacturers;
            case "list_categories":
              return store.categories;
            case "list_routes":
              return store.routes;
            case "list_active_ingredients":
              return store.ingredients;
            case "get_product_detail": {
              const detail = findDetail(String(args.id));
              if (!detail) throw { code: "notFound", message: "Product not found." };
              return detail;
            }
            case "list_products": {
              const query = (args.query ?? {}) as Record<string, unknown>;
              const search = normalize(String(query.search ?? ""));
              const manufacturerId = query.manufacturerId as string | null | undefined;
              const categoryId = query.categoryId as string | null | undefined;
              const routeId = query.routeId as string | null | undefined;
              const includeInactive = Boolean(query.includeInactive);
              const sort = String(query.sort ?? "name");
              const direction =
                String(query.sortDirection ?? "asc").toLowerCase() === "desc"
                  ? -1
                  : 1;
              let rows = enrichListItems();
              rows = rows.filter((row) => {
                const detail = store.products.find((d) => d.product.id === row.id);
                if (!includeInactive && !row.isActive) return false;
                if (manufacturerId && detail?.product.manufacturerId !== manufacturerId)
                  return false;
                if (categoryId && detail?.product.categoryId !== categoryId)
                  return false;
                if (routeId && detail?.product.routeId !== routeId) return false;
                if (search) {
                  const hay = `${row.nameEn ?? ""} ${row.nameAr ?? ""} ${row.scientificName ?? ""} ${row.barcode ?? ""}`.toLowerCase();
                  if (!hay.includes(search)) return false;
                }
                return true;
              });
              const names = (row: (typeof rows)[number]) =>
                (row.nameEn ?? row.nameAr ?? "").toLowerCase();
              const strings = new Map<string, (row: (typeof rows)[number]) => string>([
                ["name", names],
                ["scientific", (row) => (row.scientificName ?? "").toLowerCase()],
                ["manufacturer", (row) => (row.manufacturerName ?? "").toLowerCase()],
                ["category", (row) => (row.categoryName ?? "").toLowerCase()],
                ["route", (row) => (row.routeName ?? "").toLowerCase()],
              ]);
              const price = (row: (typeof rows)[number]) =>
                row.sellingPriceMinor ?? Number.MAX_SAFE_INTEGER;
              const compare = (a: (typeof rows)[number], b: (typeof rows)[number]) => {
                if (sort === "active")
                  return (Number(b.isActive) - Number(a.isActive)) * direction;
                if (sort === "price") return (price(a) - price(b)) * direction;
                if (sort === "createdAt")
                  return (a.createdAt < b.createdAt ? -1 : 1) * direction;
                if (sort === "updatedAt")
                  return (a.updatedAt < b.updatedAt ? -1 : 1) * direction;
                const getter = strings.get(sort) ?? names;
                return (getter(a).localeCompare(getter(b)) || (a.id < b.id ? -1 : 1)) * direction;
              };
              rows = rows.sort(compare);
              const limit = Number(query.limit ?? rows.length);
              const offset = Number(query.offset ?? 0);
              const total = rows.length;
              const activeTotal = rows.filter((row) => row.isActive).length;
              return {
                items: rows.slice(offset, offset + limit),
                total,
                activeTotal,
                limit,
                offset,
              };
            }
            case "create_product_full": {
              const input = args.input as ProductInput;
              const product = { ...toProduct(input), id: uuid("p") };
              const detail = buildDetail(product, input);
              store.products.push(detail);
              return product;
            }
            case "update_product_full": {
              const input = args.input as ProductInput;
              const id = String(input.id);
              const existing = findDetail(id);
              if (!existing) throw { code: "notFound", message: "Product not found." };
              const product: Product = {
                ...existing.product,
                commercialNameEn: input.commercialNameEn ?? null,
                commercialNameAr: input.commercialNameAr ?? null,
                normalizedNameEn: normalize(input.commercialNameEn ?? ""),
                normalizedNameAr: normalize(input.commercialNameAr ?? ""),
                scientificName: input.scientificName ?? null,
                normalizedScientificName: normalize(input.scientificName ?? ""),
                manufacturerId: input.manufacturerId ?? null,
                categoryId: input.categoryId ?? null,
                routeId: input.routeId ?? null,
                description: input.description ?? null,
                notes: input.notes ?? null,
                isActive: input.isActive ?? existing.product.isActive,
                updatedAt: now(),
              };
              const next = buildDetail(product, input, existing);
              store.products = store.products.map((d) =>
                d.product.id === id ? next : d,
              );
              return product;
            }
            case "set_product_active": {
              const id = String(args.id);
              const existing = findDetail(id);
              if (!existing) throw { code: "notFound", message: "Product not found." };
              existing.product.isActive = Boolean(args.active);
              existing.product.updatedAt = now();
              return existing.product;
            }
            case "get_package_price_history": {
              const packageId = String(args.packageId);
              const detail = store.products.find((d) =>
                d.packages.some((x) => x.package.id === packageId),
              );
              const entry = detail?.packages.find((x) => x.package.id === packageId);
              return entry?.currentPrice
                ? [
                    {
                      ...entry.currentPrice,
                      effectiveTo: entry.currentPrice.effectiveTo,
                    },
                  ]
                : [];
            }
            default:
              throw new Error(`Unexpected native command: ${command}`);
          }
        },
      },
    });
  });
}