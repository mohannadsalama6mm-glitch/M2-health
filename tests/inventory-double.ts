import { type Page } from "@playwright/test";

/**
 * Browser-only IPC double for the real inventory commands. It mirrors the exact
 * camelCase wire contracts of the Rust commands (`models/inventory.rs`) with an
 * in-memory ledger store that survives SPA navigation. It layers on top of the
 * catalog double so branch/category/product helpers resolve there. Never used by
 * production code.
 */
export function installInventoryDouble(page: Page) {
  return page.addInitScript(() => {
    const w = window as unknown as {
      __catalogStore?: {
        branches: {
          id: string;
          code: string;
          name: string;
          phone: string;
          address: string;
          isActive: boolean;
          createdAt: string;
          updatedAt: string;
        }[];
      };
      __TAURI_INTERNALS__?: {
        invoke: (
          command: string,
          args?: Record<string, unknown>,
        ) => Promise<unknown>;
      };
      __inventoryStore?: InventoryStore;
    };
    interface Batch {
      id: string;
      branchId: string;
      productPackageId: string;
      batchNumber: string;
      expiryDate: string;
      costPriceMinor: number | null;
      isActive: boolean;
      createdAt: string;
      updatedAt: string;
    }
    interface Movement {
      id: string;
      branchId: string;
      productPackageId: string;
      batchId: string;
      movementType: string;
      quantityDelta: number;
      costPriceMinor: number | null;
      referenceType: string | null;
      referenceId: string | null;
      reason: string;
      user: string;
      occurredAt: string;
      createdAt: string;
    }
    interface PackageRow {
      id: string;
      productId: string;
      nameEn: string | null;
      nameAr: string | null;
      scientificName: string | null;
      manufacturerName: string | null;
      packageLabel: string;
      packSize: string | null;
      sellingPriceMinor: number | null;
      costPriceMinor: number | null;
      categoryId: string | null;
      isActive: boolean;
    }
    interface CountItem {
      batchId: string;
      system: number;
      counted: number;
    }
    interface CountRow {
      id: string;
      branchId: string;
      scope: string;
      categoryId: string | null;
      status: string;
      startedAt: string;
      completedAt: string | null;
      items: CountItem[];
      createdAt: string;
      updatedAt: string;
    }
    interface InventoryStore {
      batches: Batch[];
      movements: Movement[];
      packages: PackageRow[];
      counts: CountRow[];
      reorders: Record<string, number>;
    }
    const normalize = (value: string) =>
      value.trim().toLowerCase().split(/\s+/).join(" ");
    const now = () => new Date().toISOString();
    const uuid = (prefix: string) =>
      `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    const MAIN_BRANCH = "424b25b8-3c75-4d7a-87cd-988397081194";
    const floor = (daysAhead: number) => {
      const d = new Date();
      d.setUTCDate(d.getUTCDate() + daysAhead);
      return d.toISOString().slice(0, 10);
    };
    if (w.__catalogStore) {
      const existing = w.__catalogStore.branches;
      if (!existing.some((b) => b.id === "b-maadi")) {
        existing.push({
          id: "b-maadi",
          code: "MAADI",
          name: "Maadi Branch",
          phone: "",
          address: "",
          isActive: true,
          createdAt: "2026-09-12T00:00:00.000Z",
          updatedAt: "2026-09-12T00:00:00.000Z",
        });
      }
    }
    const existingInvoke = w.__TAURI_INTERNALS__?.invoke;
    if (!w.__inventoryStore) {
      const packages: PackageRow[] = [
        {
          id: "pkg-1",
          productId: "p-1",
          nameEn: "Panadol 500 mg",
          nameAr: null,
          scientificName: "Paracetamol",
          manufacturerName: "Haleon",
          packageLabel: "Box · 10 tablets",
          packSize: "10 tablets",
          sellingPriceMinor: 3850,
          costPriceMinor: 2100,
          categoryId: "c-1",
          isActive: true,
        },
        {
          id: "pkg-2",
          productId: "p-2",
          nameEn: "Amoxicillin 500 mg",
          nameAr: null,
          scientificName: "Amoxicillin",
          manufacturerName: "Nile Pharma",
          packageLabel: "Capsule · 20",
          packSize: "20 capsules",
          sellingPriceMinor: 4800,
          costPriceMinor: 2600,
          categoryId: "c-2",
          isActive: true,
        },
      ];
      const batches: Batch[] = [
        {
          id: "b-1",
          branchId: MAIN_BRANCH,
          productPackageId: "pkg-1",
          batchNumber: "LOT-A",
          expiryDate: "",
          costPriceMinor: 2100,
          isActive: true,
          createdAt: "2026-08-01T00:00:00.000Z",
          updatedAt: "2026-08-01T00:00:00.000Z",
        },
        {
          id: "b-2",
          branchId: MAIN_BRANCH,
          productPackageId: "pkg-1",
          batchNumber: "LOT-B",
          expiryDate: floor(47),
          costPriceMinor: 2100,
          isActive: true,
          createdAt: "2026-08-01T00:00:00.000Z",
          updatedAt: "2026-08-01T00:00:00.000Z",
        },
        {
          id: "b-3",
          branchId: MAIN_BRANCH,
          productPackageId: "pkg-1",
          batchNumber: "LOT-C",
          expiryDate: floor(22),
          costPriceMinor: 2100,
          isActive: true,
          createdAt: "2026-08-01T00:00:00.000Z",
          updatedAt: "2026-08-01T00:00:00.000Z",
        },
        {
          id: "b-4",
          branchId: MAIN_BRANCH,
          productPackageId: "pkg-1",
          batchNumber: "LOT-D",
          expiryDate: floor(-6),
          costPriceMinor: 2100,
          isActive: true,
          createdAt: "2026-08-01T00:00:00.000Z",
          updatedAt: "2026-08-01T00:00:00.000Z",
        },
        {
          id: "b-5",
          branchId: MAIN_BRANCH,
          productPackageId: "pkg-2",
          batchNumber: "LOT-X",
          expiryDate: floor(90),
          costPriceMinor: 2600,
          isActive: true,
          createdAt: "2026-08-01T00:00:00.000Z",
          updatedAt: "2026-08-01T00:00:00.000Z",
        },
      ];
      const open = (batch: Batch, quantity: number, reason: string) => ({
        id: uuid("m"),
        branchId: batch.branchId,
        productPackageId: batch.productPackageId,
        batchId: batch.id,
        movementType: "opening",
        quantityDelta: quantity,
        costPriceMinor: batch.costPriceMinor,
        referenceType: "opening",
        referenceId: null,
        reason,
        user: "Stock clerk",
        occurredAt: now(),
        createdAt: now(),
      });
      const movements: Movement[] = [
        open(batches[0], 24, "Opening balance"),
        open(batches[1], 12, "Opening balance"),
        open(batches[2], 6, "Opening balance"),
        open(batches[3], 4, "Opening balance"),
        open(batches[4], 5, "Opening balance"),
      ];
      const itemFor = (b: Batch) => ({
        batchId: b.id,
        system: movements
          .filter((m) => m.batchId === b.id)
          .reduce((s, m) => s + m.quantityDelta, 0),
        counted: movements
          .filter((m) => m.batchId === b.id)
          .reduce((s, m) => s + m.quantityDelta, 0),
      });
      const items = batches.map(itemFor);
      w.__inventoryStore = {
        batches,
        movements,
        packages,
        counts: [
          {
            id: "SC-001",
            branchId: MAIN_BRANCH,
            scope: "Opening stock",
            categoryId: null,
            status: "completed",
            startedAt: "2026-08-02T09:00:00.000Z",
            completedAt: "2026-08-02T11:00:00.000Z",
            items,
            createdAt: "2026-08-02T09:00:00.000Z",
            updatedAt: "2026-08-02T11:00:00.000Z",
          },
          {
            id: "SC-002",
            branchId: MAIN_BRANCH,
            scope: "Cold storage",
            categoryId: "c-1",
            status: "draft",
            startedAt: "2026-09-01T08:00:00.000Z",
            completedAt: null,
            items,
            createdAt: "2026-09-01T08:00:00.000Z",
            updatedAt: "2026-09-01T08:00:00.000Z",
          },
          {
            id: "SC-003",
            branchId: MAIN_BRANCH,
            scope: "End-of-month",
            categoryId: null,
            status: "in_progress",
            startedAt: "2026-09-10T10:00:00.000Z",
            completedAt: null,
            items,
            createdAt: "2026-09-10T10:00:00.000Z",
            updatedAt: "2026-09-10T10:00:00.000Z",
          },
        ],
        reorders: {},
      };
    }
    const store = w.__inventoryStore;
    const branchName = (id: string) =>
      w.__catalogStore?.branches.find((b) => b.id === id)?.name ?? id;
    const balanceOf = (batchId: string) =>
      store.movements
        .filter((m) => m.batchId === batchId)
        .reduce((s, m) => s + m.quantityDelta, 0);
    const reorderOf = (packageId: string) => store.reorders[packageId] ?? 10;
    const daysToExpiry = (expiry: string) => {
      if (!expiry) return null;
      const nowUtc = new Date();
      const d = new Date(`${expiry}T00:00:00Z`);
      return Math.floor((d.getTime() - nowUtc.getTime()) / 86400000);
    };
    const batchesOf = (branchId: string, packageId: string) =>
      store.batches.filter(
        (b) =>
          b.branchId === branchId &&
          b.productPackageId === packageId &&
          b.isActive &&
          balanceOf(b.id) > 0,
      );
    const pkgBalance = (branchId: string, packageId: string) =>
      batchesOf(branchId, packageId).reduce((s, b) => s + balanceOf(b.id), 0);
    const statusOf = (row: {
      isActive: boolean;
      packageId: string;
      branchId: string;
    }) => {
      if (!row.isActive) return "inactive";
      const quantity = pkgBalance(row.branchId, row.packageId);
      if (quantity === 0) return "out_of_stock";
      if (quantity < reorderOf(row.packageId)) return "low";
      const min = batchesOf(row.branchId, row.packageId).reduce(
        (acc: number | null, b) => {
          const d = daysToExpiry(b.expiryDate);
          if (d == null) return acc;
          return acc == null ? d : Math.min(acc, d);
        },
        null,
      );
      if (min != null && min <= 30) return "expiring";
      return "in_stock";
    };
    const overviewRows = (branchId: string) =>
      store.packages
        .filter((pkg) =>
          store.batches.some(
            (b) =>
              b.productPackageId === pkg.id && b.isActive && balanceOf(b.id) > 0,
          ),
        )
        .map((pkg) => {
          const batches = batchesOf(branchId, pkg.id);
          return {
            packageId: pkg.id,
            productId: pkg.productId,
            nameEn: pkg.nameEn,
            nameAr: pkg.nameAr,
            scientificName: pkg.scientificName,
            manufacturerName: pkg.manufacturerName,
            packageLabel: pkg.packageLabel,
            packSize: pkg.packSize,
            quantity: pkgBalance(branchId, pkg.id),
            reorderLevel: reorderOf(pkg.id),
            batchCount: batches.length,
            batches: batches.map((b) => ({
              batchId: b.id,
              batchNumber: b.batchNumber,
              expiryDate: b.expiryDate,
              quantity: balanceOf(b.id),
            })),
            minExpiryDays: batches.reduce((acc: number | null, b) => {
              const d = daysToExpiry(b.expiryDate);
              if (d == null) return acc;
              return acc == null ? d : Math.min(acc, d);
            }, null),
            sellingPriceMinor: pkg.sellingPriceMinor,
            costPriceMinor: pkg.costPriceMinor,
            status: statusOf({
              isActive: pkg.isActive,
              packageId: pkg.id,
              branchId,
            }),
          };
        });
    const movementItems = (branchId: string | null) =>
      store.movements
        .filter((m) => !branchId || m.branchId === branchId)
        .map((m) => {
          const pkg = store.packages.find((p) => p.id === m.productPackageId);
          const batch = store.batches.find((b) => b.id === m.batchId);
          return {
            id: m.id,
            occurredAt: m.occurredAt,
            movementType: m.movementType,
            product: pkg?.nameEn ?? pkg?.scientificName ?? m.productPackageId,
            pack: pkg?.packageLabel ?? null,
            batch: batch?.batchNumber ?? null,
            branch: branchName(m.branchId),
            incoming: m.quantityDelta > 0 ? m.quantityDelta : null,
            outgoing: m.quantityDelta < 0 ? -m.quantityDelta : null,
            user: m.user,
            reason: m.reason,
          };
        });
    const newInvoke = async (
      command: string,
      args: Record<string, unknown> = {},
    ): Promise<unknown> => {
      switch (command) {
        case "get_stock_overview": {
          const query = (args.query ?? {}) as Record<string, unknown>;
          const branchId = String(query.branchId);
          const search = normalize(String(query.search ?? ""));
          const status = String(query.status ?? "");
          const sort = String(query.sort ?? "name");
          const direction =
            String(query.sortDirection ?? "asc").toLowerCase() === "desc"
              ? -1
              : 1;
          let rows = overviewRows(branchId);
          if (status) rows = rows.filter((r) => r.status === status);
          if (search)
            rows = rows.filter((r) =>
              `${r.nameEn ?? ""} ${r.nameAr ?? ""} ${r.scientificName ?? ""} ${r.packageLabel ?? ""}`
                .toLowerCase()
                .includes(search),
            );
          const strings = new Map<string, (r: (typeof rows)[number]) => string>([
            ["name", (r) => (r.nameEn ?? r.nameAr ?? "").toLowerCase()],
            ["manufacturer", (r) => (r.manufacturerName ?? "").toLowerCase()],
          ]);
          const compare = (a: (typeof rows)[number], b: (typeof rows)[number]) => {
            if (sort === "quantity")
              return (a.quantity - b.quantity) * direction;
            if (sort === "reorder") return (a.reorderLevel - b.reorderLevel) * direction;
            if (sort === "expiry")
              return ((a.minExpiryDays ?? 999) - (b.minExpiryDays ?? 999)) * direction;
            if (sort === "price")
              return ((a.sellingPriceMinor ?? 0) - (b.sellingPriceMinor ?? 0)) * direction;
            if (sort === "status") return a.status.localeCompare(b.status) * direction;
            const getter = strings.get(sort);
            if (!getter)
              return ((a.nameEn ?? a.nameAr ?? "").localeCompare(
                b.nameEn ?? b.nameAr ?? "",
              ) || (a.packageId < b.packageId ? -1 : 1)) * direction;
            return (getter(a).localeCompare(getter(b)) || (a.packageId < b.packageId ? -1 : 1)) * direction;
          };
          rows = rows.sort(compare);
          const limit = Number(query.limit ?? rows.length);
          const offset = Number(query.offset ?? 0);
          const total = rows.length;
          return {
            items: rows.slice(offset, offset + limit),
            total,
            limit,
            offset,
          };
        }
        case "get_stock_summary": {
          const branchId = String(args.branchId);
          const rows = overviewRows(branchId);
          const summary = {
            inStock: rows.filter((r) => r.status === "in_stock").length,
            low: rows.filter((r) => r.status === "low").length,
            outOfStock: rows.filter((r) => r.status === "out_of_stock").length,
            valueMinor: rows.reduce(
              (s, r) => s + r.batches.reduce((x, b) => x + b.quantity * (r.costPriceMinor ?? 0), 0),
              0,
            ),
          };
          return summary;
        }
        case "list_stock_movements": {
          const query = (args.query ?? {}) as Record<string, unknown>;
          const branchId = query.branchId ? String(query.branchId) : null;
          const movementType = String(query.movementType ?? "");
          const search = normalize(String(query.search ?? ""));
          const direction =
            String(query.sortDirection ?? "asc").toLowerCase() === "desc"
              ? -1
              : 1;
          let items = movementItems(branchId);
          if (movementType)
            items = items.filter((m) => m.movementType === movementType);
          if (search)
            items = items.filter((m) =>
              `${m.product} ${m.batch ?? ""} ${m.reason}`.toLowerCase().includes(search),
            );
          items = items.sort((a, b) => {
            const cmp =
              a.occurredAt < b.occurredAt ? -1 : a.occurredAt > b.occurredAt ? 1 : 0;
            return cmp * direction;
          });
          const limit = Number(query.limit ?? items.length);
          const offset = Number(query.offset ?? 0);
          const total = items.length;
          return { items: items.slice(offset, offset + limit), total, limit, offset };
        }
        case "post_opening_stock": {
          const input = (args.input ?? {}) as {
            branchId: string;
            productPackageId: string;
            batchNumber: string;
            expiryDate?: string;
            costPriceMinor?: number | null;
            quantity: number;
            reason?: string;
            user?: string | null;
          };
          const batch: Batch = {
            id: uuid("b"),
            branchId: input.branchId,
            productPackageId: input.productPackageId,
            batchNumber: input.batchNumber,
            expiryDate: input.expiryDate ?? "",
            costPriceMinor: input.costPriceMinor ?? null,
            isActive: true,
            createdAt: now(),
            updatedAt: now(),
          };
          store.batches.push(batch);
          const m: Movement = {
            id: uuid("m"),
            branchId: input.branchId,
            productPackageId: input.productPackageId,
            batchId: batch.id,
            movementType: "opening",
            quantityDelta: input.quantity,
            costPriceMinor: batch.costPriceMinor,
            referenceType: "opening",
            referenceId: null,
            reason: input.reason ?? "",
            user: input.user ?? "",
            occurredAt: now(),
            createdAt: now(),
          };
          store.movements.push(m);
          return m;
        }
        case "adjust_stock": {
          const input = (args.input ?? {}) as {
            branchId: string;
            productPackageId: string;
            batchId?: string | null;
            newQuantity: number;
            reason: string;
            user?: string | null;
          };
          const current = pkgBalance(input.branchId, input.productPackageId);
          let batchId = input.batchId ?? null;
          if (!batchId) {
            const candidates = batchesOf(input.branchId, input.productPackageId)
              .slice()
              .sort(
                (a, b) =>
                  (daysToExpiry(a.expiryDate) ?? 9999) -
                  (daysToExpiry(b.expiryDate) ?? 9999),
              );
            batchId =
              candidates.find((b) => balanceOf(b.id) > 0)?.id ??
              candidates[0]?.id ??
              null;
          }
          if (!batchId) throw { code: "notFound", message: "No active batch for this package." };
          const delta = input.newQuantity - current;
          const m: Movement = {
            id: uuid("m"),
            branchId: input.branchId,
            productPackageId: input.productPackageId,
            batchId,
            movementType: "adjustment",
            quantityDelta: delta,
            costPriceMinor: null,
            referenceType: null,
            referenceId: null,
            reason: input.reason,
            user: input.user ?? "",
            occurredAt: now(),
            createdAt: now(),
          };
          if (delta !== 0) store.movements.push(m);
          return { newQuantity: input.newQuantity, movements: delta !== 0 ? [m] : [] };
        }
        case "write_off_stock": {
          const input = (args.input ?? {}) as {
            batchId: string;
            kind: "damage" | "expired" | "supplier_return";
            quantity: number;
            reason: string;
            user?: string | null;
          };
          const batch = store.batches.find((b) => b.id === input.batchId);
          if (!batch) throw { code: "notFound", message: "Batch not found." };
          const m: Movement = {
            id: uuid("m"),
            branchId: batch.branchId,
            productPackageId: batch.productPackageId,
            batchId: batch.id,
            movementType: input.kind,
            quantityDelta: -input.quantity,
            costPriceMinor: batch.costPriceMinor,
            referenceType: null,
            referenceId: null,
            reason: input.reason,
            user: input.user ?? "",
            occurredAt: now(),
            createdAt: now(),
          };
          store.movements.push(m);
          return m;
        }
        case "transfer_stock": {
          const input = (args.input ?? {}) as {
            fromBranchId: string;
            toBranchId: string;
            productPackageId: string;
            batchId: string;
            quantity: number;
            reason?: string;
            user?: string | null;
          };
          const fromBatch = store.batches.find((b) => b.id === input.batchId);
          if (!fromBatch) throw { code: "notFound", message: "Source batch not found." };
          const dest = store.batches.find(
            (b) =>
              b.branchId === input.toBranchId &&
              b.productPackageId === input.productPackageId &&
              b.batchNumber === fromBatch.batchNumber &&
              b.expiryDate === fromBatch.expiryDate,
          );
          const toBatch = dest ?? {
            ...fromBatch,
            id: uuid("b"),
            branchId: input.toBranchId,
            createdAt: now(),
            updatedAt: now(),
          };
          if (!dest) store.batches.push(toBatch);
          const referenceId = uuid("tr");
          const out: Movement = {
            id: uuid("m"),
            branchId: input.fromBranchId,
            productPackageId: input.productPackageId,
            batchId: input.batchId,
            movementType: "transfer_out",
            quantityDelta: -input.quantity,
            costPriceMinor: fromBatch.costPriceMinor,
            referenceType: "transfer",
            referenceId,
            reason: input.reason ?? "",
            user: input.user ?? "",
            occurredAt: now(),
            createdAt: now(),
          };
          const incoming: Movement = {
            ...out,
            id: uuid("m"),
            branchId: input.toBranchId,
            batchId: toBatch.id,
            movementType: "transfer_in",
            quantityDelta: input.quantity,
          };
          store.movements.push(out, incoming);
          return { out, incoming, fromBatchId: input.batchId, toBatchId: toBatch.id };
        }
        case "list_stock_counts": {
          const query = (args.query ?? {}) as Record<string, unknown>;
          const branchId = query.branchId ? String(query.branchId) : null;
          const status = String(query.status ?? "");
          const search = normalize(String(query.search ?? ""));
          let counts = store.counts.filter((c) => !branchId || c.branchId === branchId);
          if (status) counts = counts.filter((c) => c.status === status);
          if (search) counts = counts.filter((c) => c.id.toLowerCase().includes(search));
          counts = counts.sort((a, b) => (a.createdAt < b.createdAt ? 1 : -1));
          const limit = Number(query.limit ?? counts.length);
          const offset = Number(query.offset ?? 0);
          const total = counts.length;
          return {
            items: counts.slice(offset, offset + limit).map((c) => ({
              id: c.id,
              branchName: branchName(c.branchId),
              scope: c.scope,
              categoryName: null,
              status: c.status,
              startedAt: c.startedAt,
              completedAt: c.completedAt,
              itemCount: c.items.length,
              discrepancyCount: c.items.filter((i) => i.counted !== i.system).length,
            })),
            total,
            limit,
            offset,
          };
        }
        case "get_stock_count": {
          const id = String(args.id);
          const c = store.counts.find((x) => x.id === id);
          if (!c) throw { code: "notFound", message: "Count session not found." };
          const pack = (packageId: string) =>
            store.packages.find((p) => p.id === packageId);
          const batch = c.items
            .map((it) => ({
              ...it,
              batch: store.batches.find((b) => b.id === it.batchId),
            }))
            .map((it) => {
              const pkg = pack(it.batch?.productPackageId ?? "");
              return {
                id: `${c.id}-${it.batchId}`,
                productPackageId: it.batch?.productPackageId ?? "",
                batchId: it.batchId,
                product: pkg?.nameEn ?? pkg?.scientificName ?? "Unnamed",
                pack: pkg?.packageLabel ?? null,
                batchNumber: it.batch?.batchNumber ?? "",
                systemQuantity: it.system,
                countedQuantity: it.counted,
                variance: it.counted - it.system,
              };
            });
          return {
            count: {
              id: c.id,
              branchId: c.branchId,
              scope: c.scope,
              categoryId: c.categoryId,
              status: c.status,
              startedAt: c.startedAt,
              completedAt: c.completedAt,
              createdAt: c.createdAt,
              updatedAt: c.updatedAt,
            },
            branchName: branchName(c.branchId),
            items: batch,
          };
        }
        case "create_stock_count": {
          const input = (args.input ?? {}) as {
            branchId: string;
            scope?: string;
            categoryId?: string | null;
          };
          const batches = batchesOf(input.branchId, "*").length
            ? store.batches.filter(
                (b) => b.branchId === input.branchId && b.isActive && balanceOf(b.id) > 0,
              )
            : [];
          void batches;
          const items = store.batches
            .filter(
              (b) =>
                b.branchId === input.branchId && b.isActive && balanceOf(b.id) > 0,
            )
            .map((b) => {
              const qty = balanceOf(b.id);
              return { batchId: b.id, system: qty, counted: qty };
            });
          const count: CountRow = {
            id: uuid("SC"),
            branchId: input.branchId,
            scope: input.scope ?? "",
            categoryId: input.categoryId ?? null,
            status: "draft",
            startedAt: now(),
            completedAt: null,
            items,
            createdAt: now(),
            updatedAt: now(),
          };
          store.counts.unshift(count);
          return {
            id: count.id,
            branchId: count.branchId,
            scope: count.scope,
            categoryId: count.categoryId,
            status: count.status,
            startedAt: count.startedAt,
            completedAt: count.completedAt,
            createdAt: count.createdAt,
            updatedAt: count.updatedAt,
          };
        }
        case "save_count_item": {
          const input = (args.input ?? {}) as {
            stockCountId: string;
            batchId: string;
            countedQuantity: number;
          };
          const c = store.counts.find((x) => x.id === input.stockCountId);
          if (!c) throw { code: "notFound", message: "Count session not found." };
          const item = c.items.find((i) => i.batchId === input.batchId);
          if (!item) throw { code: "notFound", message: "Count item not found." };
          item.counted = input.countedQuantity;
          c.status = c.status === "completed" ? c.status : "in_progress";
          c.updatedAt = now();
          return {
            id: `${c.id}-${input.batchId}`,
            stockCountId: c.id,
            productPackageId: "pkg",
            batchId: item.batchId,
            systemQuantity: item.system,
            countedQuantity: item.counted,
            variance: item.counted - item.system,
          };
        }
        case "complete_stock_count": {
          const id = String((args.input as { stockCountId: string }).stockCountId);
          const c = store.counts.find((x) => x.id === id);
          if (!c) throw { code: "notFound", message: "Count session not found." };
          if (c.status !== "completed") {
            c.items.forEach((item) => {
              const variance = item.counted - item.system;
              if (variance === 0) return;
              const batch = store.batches.find((b) => b.id === item.batchId);
              store.movements.push({
                id: uuid("m"),
                branchId: c.branchId,
                productPackageId: batch?.productPackageId ?? "",
                batchId: item.batchId,
                movementType: "count_correction",
                quantityDelta: variance,
                costPriceMinor: batch?.costPriceMinor ?? null,
                referenceType: "stock_count",
                referenceId: c.id,
                reason: `Stock count ${c.id}`,
                user: "",
                occurredAt: now(),
                createdAt: now(),
              });
            });
            c.status = "completed";
            c.completedAt = now();
            c.updatedAt = now();
          }
          const fresh = store.counts.find((x) => x.id === id)!;
          const detail = newInvoke("get_stock_count", { id: fresh.id });
          return detail;
        }
        case "list_expiry": {
          const query = (args.query ?? {}) as Record<string, unknown>;
          const branchId = String(query.branchId);
          const windowDays = Number(query.windowDays ?? 30);
          const search = normalize(String(query.search ?? ""));
          let rows: {
            batchId: string;
            packageId: string;
            productId: string;
            nameEn: string | null;
            nameAr: string | null;
            scientificName: string | null;
            packageLabel: string;
            batchNumber: string;
            expiryDate: string;
            days: number;
            quantity: number;
            valueMinor: number;
            costPriceMinor: number | null;
            branchName: string;
          }[] = [];
          store.batches.forEach((b) => {
            const d = daysToExpiry(b.expiryDate);
            if (b.branchId !== branchId || d == null || balanceOf(b.id) <= 0) return;
            if (windowDays === -1 ? d >= 0 : d > windowDays || d < 0) return;
            const pkg = store.packages.find((p) => p.id === b.productPackageId);
            rows.push({
              batchId: b.id,
              packageId: b.productPackageId,
              productId: pkg?.productId ?? "",
              nameEn: pkg?.nameEn ?? null,
              nameAr: pkg?.nameAr ?? null,
              scientificName: pkg?.scientificName ?? null,
              packageLabel: pkg?.packageLabel ?? "",
              batchNumber: b.batchNumber,
              expiryDate: b.expiryDate,
              days: d,
              quantity: balanceOf(b.id),
              valueMinor: balanceOf(b.id) * (b.costPriceMinor ?? 0),
              costPriceMinor: b.costPriceMinor,
              branchName: branchName(branchId),
            });
          });
          if (search)
            rows = rows.filter((r) =>
              `${r.nameEn ?? ""} ${r.batchNumber}`.toLowerCase().includes(search),
            );
          rows = rows.sort(
            (a, b) =>
              a.expiryDate.localeCompare(b.expiryDate) ||
              (a.batchId < b.batchId ? -1 : 1),
          );
          const limit = Number(query.limit ?? rows.length);
          const offset = Number(query.offset ?? 0);
          const total = rows.length;
          return { items: rows.slice(offset, offset + limit), total, limit, offset };
        }
        case "list_low_stock": {
          const query = (args.query ?? {}) as Record<string, unknown>;
          const branchId = String(query.branchId);
          const search = normalize(String(query.search ?? ""));
          let rows = overviewRows(branchId)
            .filter((r) => r.quantity === 0 || r.quantity < r.reorderLevel)
            .map((r) => ({
              packageId: r.packageId,
              productId: r.productId,
              nameEn: r.nameEn,
              nameAr: r.nameAr,
              packageLabel: r.packageLabel,
              quantity: r.quantity,
              reorderLevel: r.reorderLevel,
              status: r.status,
            }));
          if (search) rows = rows.filter((r) => `${r.nameEn ?? ""}`.toLowerCase().includes(search));
          return rows;
        }
        case "set_reorder_level": {
          const input = (args.input ?? {}) as {
            branchId: string;
            productPackageId: string;
            reorderLevel: number;
          };
          store.reorders[input.productPackageId] = input.reorderLevel;
          return {
            branchId: input.branchId,
            productPackageId: input.productPackageId,
            reorderLevel: input.reorderLevel,
            createdAt: now(),
            updatedAt: now(),
          };
        }
        case "fefo_allocation": {
          const branchId = String(args.branchId);
          const packageId = String(args.productPackageId);
          const want = Number(args.quantity ?? Number.MAX_SAFE_INTEGER);
          const batches = batchesOf(branchId, packageId)
            .slice()
            .sort(
              (a, b) =>
                ((daysToExpiry(a.expiryDate) ?? 9999) - (daysToExpiry(b.expiryDate) ?? 9999)) ||
                (a.id < b.id ? -1 : 1),
            );
          let remaining = want;
          return batches.map((b) => {
            const available = balanceOf(b.id);
            const take = remaining > 0 ? Math.min(available, remaining) : 0;
            remaining = Math.max(0, remaining - take);
            return {
              batchId: b.id,
              batchNumber: b.batchNumber,
              expiryDate: b.expiryDate,
              available,
              take,
            };
          });
        }
        default:
          if (existingInvoke) return existingInvoke(command, args);
          throw new Error(`Unexpected native command: ${command}`);
      }
    };
    Object.assign(window, {
      isTauri: true,
      __TAURI_INTERNALS__: {
        invoke: async (
          command: string,
          args: Record<string, unknown> = {},
        ) => newInvoke(command, args),
      },
    });
  });
}