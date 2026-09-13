import { type Page } from "@playwright/test";

/**
 * Browser-only IPC double for the Phase 5 commands: sales, customers, suppliers
 * and purchases. It mirrors the exact camelCase wire contracts of the Rust
 * commands (`models/{sales,customers,suppliers,purchases}.rs`) with an in-memory
 * store that survives SPA navigation, and layers on top of the catalog and
 * inventory doubles (delegates unknown commands to their invoke). Never used by
 * production code.
 */
export function installSalesDouble(page: Page) {
  return page.addInitScript(() => {
    const w = window as unknown as {
      __catalogStore?: {
        branches: {
          id: string;
          code: string;
          name: string;
        }[];
      };
      __TAURI_INTERNALS__?: {
        invoke: (
          command: string,
          args?: Record<string, unknown>,
        ) => Promise<unknown>;
      };
      __phase5Store?: Phase5Store;
    };
    interface Sellable {
      packageId: string;
      productId: string;
      productName: string;
      packageLabel: string;
      packSize: string | null;
      sellingPriceMinor: number;
      costPriceMinor: number;
      barcode: string;
      quantity: number;
      status: string;
    }
    interface Partner {
      id: string;
      code: string;
      name: string;
      phone: string;
      email: string;
      address: string;
      notes: string;
      isActive: boolean;
      createdAt: string;
      updatedAt: string;
    }
    interface SaleDoc {
      id: string;
      branchId: string;
      sequence: number;
      receiptNumber: string;
      customerId: string | null;
      customerName: string;
      customerPhone: string;
      status: string;
      subtotalMinor: number;
      discountMinor: number;
      taxMinor: number;
      totalMinor: number;
      paidMinor: number;
      changeMinor: number;
      paymentMethod: string;
      user: string;
      note: string;
      voidReason: string;
      voidedAt: string | null;
      voidedBy: string | null;
      completedAt: string;
      createdAt: string;
      updatedAt: string;
    }
    interface SaleItemDoc {
      id: string;
      saleId: string;
      productPackageId: string;
      productName: string;
      packageLabel: string;
      quantity: number;
      sellingPriceMinor: number;
      costPriceMinor: number | null;
      lineTotalMinor: number;
      lineCostMinor: number | null;
      position: number;
    }
    interface PurchaseDoc {
      id: string;
      branchId: string;
      supplierId: string | null;
      sequence: number;
      purchaseNumber: string;
      invoiceNumber: string;
      status: string;
      subtotalMinor: number;
      discountMinor: number;
      taxMinor: number;
      totalMinor: number;
      paidMinor: number;
      changeMinor: number;
      paymentMethod: string;
      user: string;
      note: string;
      voidReason: string;
      voidedAt: string | null;
      voidedBy: string | null;
      completedAt: string;
      createdAt: string;
      updatedAt: string;
    }
    interface PurchaseItemDoc {
      id: string;
      purchaseId: string;
      productPackageId: string;
      productName: string;
      packageLabel: string;
      quantity: number;
      unitCostMinor: number;
      lineTotalMinor: number;
      position: number;
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
    interface ReturnDoc {
      id: string;
      saleId: string;
      branchId: string;
      reason: string;
      user: string;
      totalRefundMinor: number;
      returnedAt: string;
      items: {
        id: string;
        saleItemId: string;
        productName: string;
        packageLabel: string;
        quantity: number;
        refundMinor: number;
        lineRefundMinor: number;
      }[];
    }
    interface Phase5Store {
      sellables: Sellable[];
      customers: Partner[];
      suppliers: Partner[];
      sales: SaleDoc[];
      saleItems: Record<string, SaleItemDoc[]>;
      purchases: PurchaseDoc[];
      purchaseItems: Record<string, PurchaseItemDoc[]>;
      returns: ReturnDoc[];
      saleSeq: Record<string, number>;
      purchaseSeq: Record<string, number>;
      movements: Movement[];
    }
    const normalize = (value: string) =>
      value.trim().toLowerCase().split(/\s+/).join(" ");
    const now = () => new Date().toISOString();
    const uuid = (prefix: string) =>
      `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    const MAIN_BRANCH = "424b25b8-3c75-4d7a-87cd-988397081194";
    const branchName = () =>
      w.__catalogStore?.branches.find((b) => b.id === MAIN_BRANCH)?.name ??
      "Main Pharmacy";
    if (!w.__phase5Store) {
      const sellables: Sellable[] = [
        {
          packageId: "pkg-1",
          productId: "p-1",
          productName: "Panadol 500 mg",
          packageLabel: "Box · 10 tablets",
          packSize: "10 tablets",
          sellingPriceMinor: 3850,
          costPriceMinor: 2100,
          barcode: "6221001000011",
          quantity: 24,
          status: "in_stock",
        },
        {
          packageId: "pkg-2",
          productId: "p-2",
          productName: "Amoxicillin 500 mg",
          packageLabel: "Capsule · 20",
          packSize: "20 capsules",
          sellingPriceMinor: 4800,
          costPriceMinor: 2600,
          barcode: "6221001000012",
          quantity: 5,
          status: "in_stock",
        },
        {
          packageId: "pkg-3",
          productId: "p-3",
          productName: "Vitamin C 1000 mg",
          packageLabel: "Jar · 60 tablets",
          packSize: "60 tablets",
          sellingPriceMinor: 2500,
          costPriceMinor: 1200,
          barcode: "6221001000013",
          quantity: 12,
          status: "in_stock",
        },
        {
          packageId: "pkg-7",
          productId: "p-7",
          productName: "Cetirizine 10 mg",
          packageLabel: "Box · 30 tablets",
          packSize: "30 tablets",
          sellingPriceMinor: 2000,
          costPriceMinor: 800,
          barcode: "6221001000014",
          quantity: 8,
          status: "in_stock",
        },
      ];
      const partner = (
        id: string,
        codePrefix: "CUS" | "SUP",
        name: string,
        phone: string,
      ): Partner => ({
        id,
        code: `${codePrefix}-${Math.random().toString(16).slice(2, 6).toUpperCase()}`,
        name,
        phone,
        email: "",
        address: "",
        notes: "",
        isActive: true,
        createdAt: "2026-09-01T00:00:00.000Z",
        updatedAt: "2026-09-01T00:00:00.000Z",
      });
      const customers: Partner[] = [
        partner("c1", "CUS", "Sara Mohamed", "01001234567"),
        partner("c2", "CUS", "Ahmed Khaled", "01009000000"),
        partner("c3", "CUS", "Reem Hassan", "01008000000"),
      ];
      const suppliers: Partner[] = [
        partner("s1", "SUP", "United Pharma Distribution Karim Adel", "01001111111"),
        partner("s2", "SUP", "Nile Medical Supplies", "01002222222"),
      ];
      const seededSale: SaleDoc = {
        id: "1042",
        branchId: MAIN_BRANCH,
        sequence: 1,
        receiptNumber: "00001",
        customerId: null,
        customerName: "Walk-in customer",
        customerPhone: "",
        status: "completed",
        subtotalMinor: 3850,
        discountMinor: 0,
        taxMinor: 0,
        totalMinor: 3850,
        paidMinor: 3850,
        changeMinor: 0,
        paymentMethod: "cash",
        user: "Cashier",
        note: "",
        voidReason: "",
        voidedAt: null,
        voidedBy: null,
        completedAt: "2026-09-10T09:00:00.000Z",
        createdAt: "2026-09-10T09:00:00.000Z",
        updatedAt: "2026-09-10T09:00:00.000Z",
      };
      const seededPurchase: PurchaseDoc = {
        id: "PO-1048",
        branchId: MAIN_BRANCH,
        supplierId: "s2",
        sequence: 1,
        purchaseNumber: "00001",
        invoiceNumber: "",
        status: "completed",
        subtotalMinor: 4800,
        discountMinor: 0,
        taxMinor: 0,
        totalMinor: 4800,
        paidMinor: 4800,
        changeMinor: 0,
        paymentMethod: "cash",
        user: "Buyer",
        note: "",
        voidReason: "",
        voidedAt: null,
        voidedBy: null,
        completedAt: "2026-09-11T09:00:00.000Z",
        createdAt: "2026-09-11T09:00:00.000Z",
        updatedAt: "2026-09-11T09:00:00.000Z",
      };
      w.__phase5Store = {
        sellables,
        customers,
        suppliers,
        sales: [seededSale],
        saleItems: {
          "1042": [
            {
              id: "si-1042-1",
              saleId: "1042",
              productPackageId: "pkg-1",
              productName: "Panadol 500 mg",
              packageLabel: "Box · 10 tablets",
              quantity: 1,
              sellingPriceMinor: 3850,
              costPriceMinor: 2100,
              lineTotalMinor: 3850,
              lineCostMinor: 2100,
              position: 0,
            },
          ],
        },
        purchases: [seededPurchase],
        purchaseItems: {
          "PO-1048": [
            {
              id: "pi-PO-1048-1",
              purchaseId: "PO-1048",
              productPackageId: "pkg-2",
              productName: "Amoxicillin 500 mg",
              packageLabel: "Capsule · 20",
              quantity: 1,
              unitCostMinor: 2600,
              lineTotalMinor: 2600,
              position: 0,
            },
          ],
        },
        returns: [],
        saleSeq: { [MAIN_BRANCH]: 1 },
        purchaseSeq: { [MAIN_BRANCH]: 1 },
        movements: [],
      };
    }
    const store = w.__phase5Store;
    const existingInvoke = w.__TAURI_INTERNALS__?.invoke;
    const findBySellable = (packageId: string) =>
      store.sellables.find((s) => s.packageId === packageId);
    const adjustQuantity = (packageId: string, delta: number) => {
      const item = findBySellable(packageId);
      if (!item) return;
      item.quantity = Math.max(0, item.quantity + delta);
      if (item.quantity === 0) item.status = "out_of_stock";
      else item.status = "in_stock";
    };
    const salePoint = (branchId: string) => {
      store.saleSeq[branchId] = (store.saleSeq[branchId] ?? 0) + 1;
      const sequence = store.saleSeq[branchId];
      return { sequence, receiptNumber: String(sequence).padStart(5, "0") };
    };
    const purchasePoint = (branchId: string) => {
      store.purchaseSeq[branchId] = (store.purchaseSeq[branchId] ?? 0) + 1;
      const sequence = store.purchaseSeq[branchId];
      return { sequence, purchaseNumber: String(sequence).padStart(5, "0") };
    };
    const partnerPage = (
      partners: Partner[],
      query: { search?: string | null; isActive?: boolean | null; limit?: number | null; offset?: number | null },
    ) => {
      const search = normalize(String(query.search ?? ""));
      const limit = Number(query.limit ?? partners.length);
      const offset = Number(query.offset ?? 0);
      let rows = partners;
      if (query.isActive != null) rows = rows.filter((p) => p.isActive === query.isActive);
      if (search)
        rows = rows.filter((p) =>
          `${p.name} ${p.phone} ${p.email} ${p.code}`.toLowerCase().includes(search),
        );
      const total = rows.length;
      return { items: rows.slice(offset, offset + limit), total, limit, offset };
    };
    const saleRow = (s: SaleDoc) => ({
      id: s.id,
      branchId: s.branchId,
      branchName: branchName(),
      sequence: s.sequence,
      receiptNumber: s.receiptNumber,
      customerName: s.customerName,
      status: s.status,
      totalMinor: s.totalMinor,
      paidMinor: s.paidMinor,
      paymentMethod: s.paymentMethod,
      itemCount: (store.saleItems[s.id] ?? []).reduce((n, i) => n + i.quantity, 0),
      user: s.user,
      completedAt: s.completedAt,
      voidReason: s.voidReason,
    });
    const purchaseRow = (p: PurchaseDoc) => ({
      id: p.id,
      branchId: p.branchId,
      branchName: branchName(),
      supplierName:
        store.suppliers.find((s) => s.id === p.supplierId)?.name ?? null,
      sequence: p.sequence,
      purchaseNumber: p.purchaseNumber,
      invoiceNumber: p.invoiceNumber,
      status: p.status,
      totalMinor: p.totalMinor,
      paidMinor: p.paidMinor,
      paymentMethod: p.paymentMethod,
      itemCount: (store.purchaseItems[p.id] ?? []).reduce((n, i) => n + i.quantity, 0),
      user: p.user,
      completedAt: p.completedAt,
      voidReason: p.voidReason,
    });
    const saleDetail = (s: SaleDoc) => {
      const items = (store.saleItems[s.id] ?? []).map((i) => {
        const sellable = findBySellable(i.productPackageId);
        return {
          id: i.id,
          productPackageId: i.productPackageId,
          productName: i.productName,
          packageLabel: i.packageLabel,
          quantity: i.quantity,
          sellingPriceMinor: i.sellingPriceMinor,
          costPriceMinor: i.costPriceMinor,
          lineTotalMinor: i.lineTotalMinor,
          lineCostMinor: i.lineCostMinor,
          position: i.position,
          batches: [
            ...(sellable
              ? [
                  {
                    batchId: `b-${i.productPackageId}`,
                    batchNumber: `LOT-${i.productPackageId.toUpperCase()}`,
                    expiryDate: "",
                    quantity: i.quantity,
                    costPriceMinor: i.costPriceMinor,
                  },
                ]
              : []),
          ],
          returnedQuantity: 0,
          returnableQuantity: i.quantity,
        };
      });
      return {
        sale: s,
        branchName: branchName(),
        items,
        returns: store.returns.filter((r) => r.saleId === s.id),
      };
    };
    const purchaseDetail = (p: PurchaseDoc) => ({
      purchase: p,
      branchName: branchName(),
      supplierName: store.suppliers.find((s) => s.id === p.supplierId)?.name ?? null,
      items: (store.purchaseItems[p.id] ?? []).map((i) => ({
        id: i.id,
        productPackageId: i.productPackageId,
        productName: i.productName,
        packageLabel: i.packageLabel,
        quantity: i.quantity,
        unitCostMinor: i.unitCostMinor,
        lineTotalMinor: i.lineTotalMinor,
        position: i.position,
        batches: [
          {
            id: `pb-${i.purchaseId}-${i.productPackageId}`,
            batchId: `b-${i.productPackageId}`,
            quantity: i.quantity,
            unitCostMinor: i.unitCostMinor,
          },
        ],
      })),
    });
    const newInvoke = async (
      command: string,
      args: Record<string, unknown> = {},
    ): Promise<unknown> => {
      switch (command) {
        case "pos_search": {
          const query = (args.query ?? {}) as Record<string, unknown>;
          const search = normalize(String(query.search ?? ""));
          const limit = Number(query.limit ?? 50);
          const offset = Number(query.offset ?? 0);
          let rows = store.sellables;
          if (search)
            rows = rows.filter((s) =>
              `${s.productName} ${s.packageLabel ?? ""} ${s.barcode}`
                .toLowerCase()
                .includes(search),
            );
          const total = rows.length;
          return {
            items: rows.slice(offset, offset + limit).map((s) => ({
              packageId: s.packageId,
              productId: s.productId,
              productName: s.productName,
              packageLabel: s.packageLabel,
              packSize: s.packSize,
              sellingPriceMinor: s.sellingPriceMinor,
              quantity: s.quantity,
              status: s.status,
            })),
            total,
            limit,
            offset,
          };
        }
        case "complete_sale": {
          const input = (args.input ?? {}) as {
            branchId: string;
            customerId?: string | null;
            customerName?: string;
            customerPhone?: string;
            discountMinor?: number;
            taxMinor?: number;
            paidMinor?: number;
            paymentMethod?: string;
            user?: string | null;
            note?: string;
            lines: { productPackageId: string; quantity: number }[];
          };
          const branchId = input.branchId;
          const { sequence, receiptNumber } = salePoint(branchId);
          const lines = input.lines.map((l) => {
            const sellable = findBySellable(l.productPackageId);
            if (!sellable)
              throw { code: "notFound", message: "Product not found." };
            return {
              sellable,
              quantity: l.quantity,
              lineTotalMinor: sellable.sellingPriceMinor * l.quantity,
            };
          });
          const subtotalMinor = lines.reduce((sum, l) => sum + l.lineTotalMinor, 0);
          const discountMinor = input.discountMinor ?? 0;
          const taxMinor = input.taxMinor ?? 0;
          const totalMinor = Math.max(0, subtotalMinor - discountMinor + taxMinor);
          const paidMinor = input.paidMinor ?? totalMinor;
          const changeMinor = Math.max(0, paidMinor - totalMinor);
          const customer =
            (input.customerId &&
              store.customers.find((c) => c.id === input.customerId)) ||
            null;
          const customerName = input.customerName || customer?.name || "Walk-in customer";
          const sale: SaleDoc = {
            id: uuid("sale"),
            branchId,
            sequence,
            receiptNumber,
            customerId: input.customerId ?? null,
            customerName,
            customerPhone: input.customerPhone ?? customer?.phone ?? "",
            status: "completed",
            subtotalMinor,
            discountMinor,
            taxMinor,
            totalMinor,
            paidMinor,
            changeMinor,
            paymentMethod: input.paymentMethod ?? "cash",
            user: input.user ?? "Cashier",
            note: input.note ?? "",
            voidReason: "",
            voidedAt: null,
            voidedBy: null,
            completedAt: now(),
            createdAt: now(),
            updatedAt: now(),
          };
          store.sales.unshift(sale);
          const items: SaleItemDoc[] = lines.map((l, index) => ({
            id: uuid("si"),
            saleId: sale.id,
            productPackageId: l.sellable.packageId,
            productName: l.sellable.productName,
            packageLabel: l.sellable.packageLabel,
            quantity: l.quantity,
            sellingPriceMinor: l.sellable.sellingPriceMinor,
            costPriceMinor: l.sellable.costPriceMinor,
            lineTotalMinor: l.lineTotalMinor,
            lineCostMinor: l.sellable.costPriceMinor * l.quantity,
            position: index,
          }));
          store.saleItems[sale.id] = items;
          const movements: Movement[] = items.map((item) => {
            adjustQuantity(item.productPackageId, -item.quantity);
            return {
              id: uuid("m"),
              branchId,
              productPackageId: item.productPackageId,
              batchId: `b-${item.productPackageId}`,
              movementType: "sale",
              quantityDelta: -item.quantity,
              costPriceMinor: item.costPriceMinor,
              referenceType: "sale",
              referenceId: sale.id,
              reason: `Sale ${receiptNumber}`,
              user: sale.user,
              occurredAt: now(),
              createdAt: now(),
            };
          });
          store.movements.push(...movements);
          return { sale, movements };
        }
        case "list_sales": {
          const query = (args.query ?? {}) as Record<string, unknown>;
          const branchId = query.branchId ? String(query.branchId) : null;
          const status = String(query.status ?? "");
          const search = normalize(String(query.search ?? ""));
          const limit = Number(query.limit ?? 50);
          const offset = Number(query.offset ?? 0);
          let rows = store.sales.filter((s) => !branchId || s.branchId === branchId);
          if (status) rows = rows.filter((s) => s.status === status);
          if (search)
            rows = rows.filter((s) =>
              `${s.receiptNumber} ${s.customerName}`.toLowerCase().includes(search),
            );
          rows = rows.sort((a, b) =>
            a.completedAt < b.completedAt ? 1 : a.completedAt > b.completedAt ? -1 : 0,
          );
          const total = rows.length;
          return {
            items: rows.slice(offset, offset + limit).map(saleRow),
            total,
            limit,
            offset,
          };
        }
        case "get_sale": {
          const id = String(args.id);
          const sale = store.sales.find((s) => s.id === id);
          if (!sale) throw { code: "notFound", message: "Sale not found." };
          return saleDetail(sale);
        }
        case "void_sale": {
          const input = (args.input ?? {}) as {
            saleId: string;
            reason: string;
            user?: string | null;
          };
          const sale = store.sales.find((s) => s.id === input.saleId);
          if (!sale) throw { code: "notFound", message: "Sale not found." };
          if (sale.status === "void")
            throw { code: "validation", message: "Sale is already void." };
          if (store.returns.some((r) => r.saleId === sale.id))
            throw {
              code: "validation",
              message: "Cannot void a sale that has returns.",
            };
          sale.status = "void";
          sale.voidReason = input.reason;
          sale.voidedBy = input.user ?? "";
          sale.voidedAt = now();
          sale.updatedAt = now();
          const items = store.saleItems[sale.id] ?? [];
          items.forEach((item) => adjustQuantity(item.productPackageId, item.quantity));
          return sale;
        }
        case "return_sale": {
          const input = (args.input ?? {}) as {
            saleId: string;
            reason: string;
            user?: string | null;
            items: { saleItemId: string; quantity: number; refundMinor: number }[];
          };
          const sale = store.sales.find((s) => s.id === input.saleId);
          if (!sale) throw { code: "notFound", message: "Sale not found." };
          if (sale.status !== "completed")
            throw { code: "validation", message: "Only completed sales can be returned." };
          const items = (store.saleItems[sale.id] ?? [])
            .filter((i) => input.items.some((r) => r.saleItemId === i.id))
            .map((i) => {
              const request = input.items.find((r) => r.saleItemId === i.id)!;
              adjustQuantity(i.productPackageId, request.quantity);
              return {
                id: uuid("rsi"),
                saleItemId: i.id,
                productName: i.productName,
                packageLabel: i.packageLabel,
                quantity: request.quantity,
                refundMinor: request.refundMinor,
                lineRefundMinor: request.refundMinor * request.quantity,
              };
            });
          const totalRefundMinor = items.reduce((s, i) => s + i.lineRefundMinor, 0);
          const returned: ReturnDoc = {
            id: uuid("ret"),
            saleId: sale.id,
            branchId: sale.branchId,
            reason: input.reason,
            user: input.user ?? "",
            totalRefundMinor,
            returnedAt: now(),
            items,
          };
          store.returns.push(returned);
          return returned;
        }
        case "list_customers": {
          const query = (args.query ?? {}) as {
            search?: string | null;
            isActive?: boolean | null;
            limit?: number | null;
            offset?: number | null;
          };
          return partnerPage(store.customers, query);
        }
        case "get_customer": {
          const partner = store.customers.find((c) => c.id === String(args.id));
          if (!partner) throw { code: "notFound", message: "Customer not found." };
          return partner;
        }
        case "create_customer": {
          const input = (args.input ?? {}) as {
            name: string;
            phone?: string;
            email?: string;
            address?: string;
            notes?: string;
          };
          const letters = normalize(input.name).replace(/[^a-z0-9]/g, "").slice(0, 3).toUpperCase() || "CUS";
          const created: Partner = {
            id: uuid("c"),
            code: `${letters}-${Math.random().toString(16).slice(2, 6).toUpperCase()}`,
            name: input.name,
            phone: input.phone ?? "",
            email: input.email ?? "",
            address: input.address ?? "",
            notes: input.notes ?? "",
            isActive: true,
            createdAt: now(),
            updatedAt: now(),
          };
          store.customers.push(created);
          return created;
        }
        case "update_customer": {
          const input = (args.input ?? {}) as {
            id: string;
            name?: string;
            phone?: string;
            email?: string;
            address?: string;
            notes?: string;
          };
          const partner = store.customers.find((c) => c.id === input.id);
          if (!partner) throw { code: "notFound", message: "Customer not found." };
          partner.name = input.name ?? partner.name;
          partner.phone = input.phone ?? partner.phone;
          partner.email = input.email ?? partner.email;
          partner.address = input.address ?? partner.address;
          partner.notes = input.notes ?? partner.notes;
          partner.updatedAt = now();
          return partner;
        }
        case "set_customer_active": {
          const id = String(args.id);
          const partner = store.customers.find((c) => c.id === id);
          if (!partner) throw { code: "notFound", message: "Customer not found." };
          partner.isActive = Boolean(args.active);
          partner.updatedAt = now();
          return partner;
        }
        case "list_suppliers": {
          const query = (args.query ?? {}) as {
            search?: string | null;
            isActive?: boolean | null;
            limit?: number | null;
            offset?: number | null;
          };
          return partnerPage(store.suppliers, query);
        }
        case "get_supplier": {
          const partner = store.suppliers.find((s) => s.id === String(args.id));
          if (!partner) throw { code: "notFound", message: "Supplier not found." };
          return partner;
        }
        case "create_supplier": {
          const input = (args.input ?? {}) as {
            name: string;
            phone?: string;
            email?: string;
            address?: string;
            notes?: string;
          };
          const letters = normalize(input.name).replace(/[^a-z0-9]/g, "").slice(0, 3).toUpperCase() || "SUP";
          const created: Partner = {
            id: uuid("s"),
            code: `${letters}-${Math.random().toString(16).slice(2, 6).toUpperCase()}`,
            name: input.name,
            phone: input.phone ?? "",
            email: input.email ?? "",
            address: input.address ?? "",
            notes: input.notes ?? "",
            isActive: true,
            createdAt: now(),
            updatedAt: now(),
          };
          store.suppliers.push(created);
          return created;
        }
        case "update_supplier": {
          const input = (args.input ?? {}) as {
            id: string;
            name?: string;
            phone?: string;
            email?: string;
            address?: string;
            notes?: string;
          };
          const partner = store.suppliers.find((s) => s.id === input.id);
          if (!partner) throw { code: "notFound", message: "Supplier not found." };
          partner.name = input.name ?? partner.name;
          partner.phone = input.phone ?? partner.phone;
          partner.email = input.email ?? partner.email;
          partner.address = input.address ?? partner.address;
          partner.notes = input.notes ?? partner.notes;
          partner.updatedAt = now();
          return partner;
        }
        case "set_supplier_active": {
          const id = String(args.id);
          const partner = store.suppliers.find((s) => s.id === id);
          if (!partner) throw { code: "notFound", message: "Supplier not found." };
          partner.isActive = Boolean(args.active);
          partner.updatedAt = now();
          return partner;
        }
        case "purchase_pos_search": {
          const query = (args.query ?? {}) as Record<string, unknown>;
          const search = normalize(String(query.search ?? ""));
          const limit = Number(query.limit ?? 50);
          const offset = Number(query.offset ?? 0);
          let rows = store.sellables;
          if (search)
            rows = rows.filter((s) =>
              `${s.productName} ${s.packageLabel ?? ""}`.toLowerCase().includes(search),
            );
          const total = rows.length;
          return {
            items: rows.slice(offset, offset + limit).map((s) => ({
              packageId: s.packageId,
              productId: s.productId,
              productName: s.productName,
              packageLabel: s.packageLabel,
              packSize: s.packSize,
              costPriceMinor: s.costPriceMinor,
              status: s.status,
            })),
            total,
            limit,
            offset,
          };
        }
        case "complete_purchase": {
          const input = (args.input ?? {}) as {
            branchId: string;
            supplierId?: string | null;
            invoiceNumber?: string;
            discountMinor?: number;
            taxMinor?: number;
            paidMinor?: number;
            paymentMethod?: string;
            user?: string | null;
            note?: string;
            lines: {
              productPackageId: string;
              quantity: number;
              unitCostMinor: number;
            }[];
          };
          const branchId = input.branchId;
          const { sequence, purchaseNumber } = purchasePoint(branchId);
          const lines = input.lines.map((l, index) => {
            const sellable = findBySellable(l.productPackageId);
            if (!sellable)
              throw { code: "notFound", message: "Product not found." };
            return {
              id: uuid("pi"),
              purchaseId: "",
              productPackageId: sellable.packageId,
              productName: sellable.productName,
              packageLabel: sellable.packageLabel,
              quantity: l.quantity,
              unitCostMinor: l.unitCostMinor,
              lineTotalMinor: l.unitCostMinor * l.quantity,
              position: index,
            };
          });
          const subtotalMinor = lines.reduce((sum, l) => sum + l.lineTotalMinor, 0);
          const discountMinor = input.discountMinor ?? 0;
          const taxMinor = input.taxMinor ?? 0;
          const totalMinor = Math.max(0, subtotalMinor - discountMinor + taxMinor);
          const paidMinor = input.paidMinor ?? totalMinor;
          const changeMinor = Math.max(0, paidMinor - totalMinor);
          const purchase: PurchaseDoc = {
            id: uuid("po"),
            branchId,
            supplierId: input.supplierId ?? null,
            sequence,
            purchaseNumber,
            invoiceNumber: input.invoiceNumber ?? "",
            status: "completed",
            subtotalMinor,
            discountMinor,
            taxMinor,
            totalMinor,
            paidMinor,
            changeMinor,
            paymentMethod: input.paymentMethod ?? "cash",
            user: input.user ?? "Buyer",
            note: input.note ?? "",
            voidReason: "",
            voidedAt: null,
            voidedBy: null,
            completedAt: now(),
            createdAt: now(),
            updatedAt: now(),
          };
          store.purchases.unshift(purchase);
          lines.forEach((l) => {
            l.purchaseId = purchase.id;
            adjustQuantity(l.productPackageId, l.quantity);
          });
          store.purchaseItems[purchase.id] = lines;
          const movements: Movement[] = lines.map((l) => ({
            id: uuid("m"),
            branchId,
            productPackageId: l.productPackageId,
            batchId: `b-${l.productPackageId}`,
            movementType: "purchase",
            quantityDelta: l.quantity,
            costPriceMinor: l.unitCostMinor,
            referenceType: "purchase",
            referenceId: purchase.id,
            reason: `Purchase ${purchaseNumber}`,
            user: purchase.user,
            occurredAt: now(),
            createdAt: now(),
          }));
          store.movements.push(...movements);
          return { purchase, movements };
        }
        case "list_purchases": {
          const query = (args.query ?? {}) as Record<string, unknown>;
          const branchId = query.branchId ? String(query.branchId) : null;
          const status = String(query.status ?? "");
          const search = normalize(String(query.search ?? ""));
          const limit = Number(query.limit ?? 50);
          const offset = Number(query.offset ?? 0);
          let rows = store.purchases.filter((p) => !branchId || p.branchId === branchId);
          if (status) rows = rows.filter((p) => p.status === status);
          if (search)
            rows = rows.filter((p) =>
              `${p.purchaseNumber} ${p.invoiceNumber} ${store.suppliers.find((s) => s.id === p.supplierId)?.name ?? ""}`.toLowerCase().includes(search),
            );
          rows = rows.sort((a, b) =>
            a.completedAt < b.completedAt ? 1 : a.completedAt > b.completedAt ? -1 : 0,
          );
          const total = rows.length;
          return {
            items: rows.slice(offset, offset + limit).map(purchaseRow),
            total,
            limit,
            offset,
          };
        }
        case "get_purchase": {
          const id = String(args.id);
          const purchase = store.purchases.find((p) => p.id === id);
          if (!purchase) throw { code: "notFound", message: "Purchase not found." };
          return purchaseDetail(purchase);
        }
        case "void_purchase": {
          const input = (args.input ?? {}) as {
            purchaseId: string;
            reason: string;
            user?: string | null;
          };
          const purchase = store.purchases.find((p) => p.id === input.purchaseId);
          if (!purchase) throw { code: "notFound", message: "Purchase not found." };
          if (purchase.status === "void")
            throw { code: "validation", message: "Purchase is already void." };
          purchase.status = "void";
          purchase.voidReason = input.reason;
          purchase.voidedBy = input.user ?? "";
          purchase.voidedAt = now();
          purchase.updatedAt = now();
          (store.purchaseItems[purchase.id] ?? []).forEach((item) =>
            adjustQuantity(item.productPackageId, -item.quantity),
          );
          return purchase;
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