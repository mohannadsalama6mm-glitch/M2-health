import type { Page } from "@playwright/test";
import type {
  Cart,
  Sale,
  PostInput,
  ReturnInput,
  ReturnDocument,
  Query,
} from "../src/lib/tauri/canonical-sales";
import { installCatalogDouble } from "./catalog-double";
export async function installSalesDouble(page: Page) {
  await installCatalogDouble(page);
  await page.addInitScript(() => {
    const w = window as unknown as {
      __TAURI_INTERNALS__: {
        invoke: (c: string, a?: Record<string, unknown>) => Promise<unknown>;
      };
      salesCalls: { command: string; args?: Record<string, unknown> }[];
      salesFailCommand: string;
      salesDelay: number;
      salesTotal: number;
    };
    const original = w.__TAURI_INTERNALS__.invoke;
    w.salesCalls = [];
    w.salesFailCommand = "";
    w.salesDelay = 0;
    w.salesTotal = 0;
    const sales: Sale[] = JSON.parse(
      sessionStorage.getItem("phase4-test-sales") || "[]",
    );
    const save = () =>
      sessionStorage.setItem("phase4-test-sales", JSON.stringify(sales));
    const totals = (cart: Cart) => {
      const subtotalMinor = cart.lines.reduce(
        (s, l) => s + (l.unitPriceMinor ?? 100) * l.quantity,
        0,
      );
      const discountMinor =
        cart.discountMinor +
        cart.lines.reduce((s, l) => s + l.discountMinor, 0);
      return {
        subtotalMinor,
        discountMinor,
        taxMinor: cart.taxMinor,
        totalMinor: subtotalMinor - discountMinor + cart.taxMinor,
        lines: [],
      };
    };
    w.__TAURI_INTERNALS__.invoke = async (command, args) => {
      w.salesCalls.push({ command, args: structuredClone(args) });
      if (command === w.salesFailCommand)
        throw { code: "validation", message: "Controlled backend rejection" };
      if (command === "recovery_status")
        return {
          laterWorkflowsEnabled: false,
          salesEnabled: true,
          inventoryUiEnabled: true,
          inventoryWorkflowsEnabled: true,
          stockCountsEnabled: true,
          inventoryAlertsEnabled: true,
        };
      if (command === "get_sale_finance_trace") {
        const saleId=String(args!.saleId);
        return {saleId,financePosted:true,historicalUnposted:false,coverageNote:"This sale has a posted Finance journal.",journals:[{id:"journal-sale",journalNumber:"JRN-20261005-00001",sourceType:"SALE",sourceId:saleId,sourceParentId:null,branchId:"main",reversalOfJournalId:null,description:"Posted sale",debitTotalMinor:100,creditTotalMinor:100,cogsComplete:true,postedAt:"2026-10-05T10:00:00.000Z"}]};
      }
      if (command === "search_pos_products") {
        if (w.salesDelay) await new Promise((r) => setTimeout(r, w.salesDelay));
        const q = args!.query as Query;
        return {
          items:
            q.search === "missing"
              ? []
              : [
                  {
                    productPackageId: "pkg-1",
                    productId: "p-1",
                    productName: "Canonical medicine",
                    arabicName: "دواء",
                    scientificName: "Ingredient",
                    packageLabel: "Box",
                    currentPriceMinor: 100,
                    availableQuantity: 8,
                    stockStatus: "IN_STOCK",
                    barcodeMatch: q.search === "123456",
                  },
                ],
          total: q.search === "missing" ? 0 : 1,
        };
      }
      if (command === "list_canonical_customers") {
        return {
          items: [{
            id: "customer-1", code: "CUS-000001", displayName: "Ahmed Hassan",
            primaryPhone: "01000000000", secondaryPhone: null, email: null,
            address: null, notes: null, isActive: true, revision: 1,
            createdAt: "2026-09-15T00:00:00Z", updatedAt: "2026-09-15T00:00:00Z",
          }],
          total: 1, limit: 8, offset: 0,
        };
      }
      if (command === "preview_sale_totals") {
        const cart = args!.cart as Cart;
        if (cart.lines.some((l) => l.quantity > 8))
          throw {
            code: "validation",
            message: "Insufficient non-expired stock",
          };
        return totals(cart);
      }
      if (command === "save_sale_draft") {
        const i = args!.input as {
          id: string;
          expectedRevision: number | null;
          cart: Cart;
        };
        let s = sales.find((s) => s.id === i.id);
        if (!s) {
          s = {
            id: i.id,
            saleNumber: null,
            branchId: i.cart.branchId,
            customerId: i.cart.customerId,
            customerCodeSnapshot: i.cart.customerId ? "CUS-000001" : null,
            customerPhoneSnapshot: i.cart.customerId ? "01000000000" : null,
            status: "DRAFT",
            revision: 0,
            cart: i.cart,
            createdAt: "2026-09-15T00:00:00Z",
            postedAt: null,
            voidedAt: null,
            customerName: null,
            notes: null,
            paymentStatus: "UNPAID",
            items: [],
            payments: [],
            returns: [],
            voids: [],
            voidAllocations: [],
            refundedMinor: 0,
            ...totals(i.cart),
          };
          sales.push(s);
        }
        s.cart = structuredClone(i.cart);
        s.customerId = i.cart.customerId;
        s.customerName = i.cart.customerName;
        s.cart.lines.forEach(
          (l) => (l.displayName = "Canonical medicine · Box"),
        );
        s.revision++;
        Object.assign(s, totals(i.cart));
        save();
        return structuredClone(s);
      }
      if (command === "hold_sale" || command === "resume_sale") {
        const i = args!.input as { id: string };
        const s = sales.find((s) => s.id === i.id)!;
        s.status = command === "hold_sale" ? "HELD" : "DRAFT";
        s.revision++;
        save();
        return structuredClone(s);
      }
      if (command === "post_sale") {
        if (w.salesDelay) await new Promise((r) => setTimeout(r, w.salesDelay));
        const p = args!.input as PostInput;
        const s = sales.find((s) => s.id === p.id)!;
        if (p.payments.reduce((a, p) => a + p.amountMinor, 0) !== s.totalMinor)
          throw {
            code: "validation",
            message: "Payments must equal the exact sale total",
          };
        if (s.status === "POSTED") return structuredClone(s);
        s.status = "POSTED";
        s.customerCodeSnapshot = s.customerId ? "CUS-000001" : null;
        s.customerPhoneSnapshot = s.customerId ? "01000000000" : null;
        s.saleNumber = "SAL-20260915-0001";
        s.revision++;
        s.postedAt = "2026-09-15T00:00:00Z";
        s.paymentStatus = "PAID";
        s.payments = p.payments;
        s.items = s.cart.lines.map((l, i) => ({
          id: `item-${i}`,
          productPackageId: l.productPackageId,
          productNameSnapshot: "Canonical medicine",
          packageLabelSnapshot: "Box",
          quantity: l.quantity,
          unitPriceMinor: l.unitPriceMinor ?? 100,
          discountMinor: l.discountMinor,
          saleDiscountMinor: s.cart.discountMinor,
          taxMinor: s.cart.taxMinor,
          lineTotalMinor: s.totalMinor,
          unitCostMinor: 30,
          returnedQuantity: 0,
          remainingQuantity: l.quantity,
          allocations: [
            {
              id: "allocation-1",
              batchId: "batch-1",
              quantity: l.quantity,
              unitCostMinor: 30,
              movementId: "sale-movement-1",
              lotNumber: "Original lot",
              expiryDate: "2030-01-01",
            },
          ],
        }));
        save();
        return structuredClone(s);
      }
      if (command === "get_canonical_sale")
        return structuredClone(sales.find((s) => s.id === args!.id));
      if (command === "list_canonical_sales") {
        const q = args!.query as Query;
        const rows = sales.filter(
          (s) =>
            s.branchId === q.branchId && (!q.status || s.status === q.status),
        );
        return {
          items: structuredClone(rows.slice(q.offset, q.offset + q.limit)),
          total: w.salesTotal || rows.length,
        };
      }
      if (command === "preview_sale_return") {
        const i = args!.input as ReturnInput;
        const s = sales.find((s) => s.id === i.saleId)!;
        return {
          refundMinor: Math.floor(
            (s.items[0].lineTotalMinor * i.lines[0].quantity) /
              s.items[0].quantity,
          ),
        };
      }
      if (command === "create_sale_return") {
        const i = args!.input as ReturnInput;
        const s = sales.find((s) => s.id === i.saleId)!;
        let r = s.returns.find((r) => r.id === i.id);
        if (!r) {
          r = {
            ...i,
            status: "DRAFT",
            refundMinor: Math.floor(
              (s.items[0].lineTotalMinor * i.lines[0].quantity) /
                s.items[0].quantity,
            ),
          } as ReturnDocument;
          s.returns.push(r);
        }
        save();
        return structuredClone(r);
      }
      if (command === "post_sale_return") {
        const i = args!.input as { id: string };
        const s = sales.find((s) => s.returns.some((r) => r.id === i.id))!;
        const r = s.returns.find((r) => r.id === i.id)!;
        if (r.status !== "POSTED") {
          r.status = "POSTED";
          s.items[0].returnedQuantity += r.lines[0].quantity;
          s.items[0].remainingQuantity -= r.lines[0].quantity;
          s.refundedMinor += r.refundMinor;
          r.allocations = [
            {
              originalAllocationId: "allocation-1",
              quantity: r.lines[0].quantity,
              movementId: "return-movement-1",
              refundMinor: r.refundMinor,
            },
          ];
        }
        save();
        return structuredClone(s);
      }
      if (command === "void_canonical_sale") {
        const i = args!.input as { id: string; reason: string };
        const s = sales.find((s) => s.id === i.id)!;
        s.status = "VOIDED";
        s.refundedMinor = s.totalMinor;
        s.voids = [
          { id: "void-1", reason: i.reason, refundMinor: s.totalMinor },
        ];
        s.voidAllocations = [
          {
            originalAllocationId: "allocation-1",
            movementId: "void-movement-1",
          },
        ];
        save();
        return structuredClone(s);
      }
      return original(command, args);
    };
  });
}
