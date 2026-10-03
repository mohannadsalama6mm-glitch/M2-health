import type { Page } from "@playwright/test";
import { installCatalogDouble } from "./catalog-double";

export async function installCanonicalPartnersDouble(page: Page) {
  await installCatalogDouble(page);
  await page.addInitScript(() => {
    type Partner = {
      id: string;
      code: string;
      displayName: string;
      primaryPhone: string | null;
      secondaryPhone: string | null;
      email: string | null;
      address: string | null;
      notes: string | null;
      isActive: boolean;
      revision: number;
      createdAt: string;
      updatedAt: string;
      contactPerson?: string | null;
      taxRegistrationId?: string | null;
    };
    const w = window as unknown as {
      __TAURI_INTERNALS__: {
        invoke: (
          command: string,
          args?: Record<string, unknown>,
        ) => Promise<unknown>;
      };
      partnerCalls: { command: string; args: Record<string, unknown> }[];
      partnerFailCommand: string;
      partnerDelay: number;
      partnerEmpty: boolean;
    };
    const original = w.__TAURI_INTERNALS__.invoke;
    const now = "2026-10-03T10:00:00.000Z";
    const suppliers: Partner[] = [
      {
        id: "s1",
        code: "SUP-000001",
        displayName: "Nile Medical",
        contactPerson: "Dina Ali",
        primaryPhone: "01000000001",
        secondaryPhone: null,
        email: "sales@nile.test",
        address: "Cairo",
        taxRegistrationId: "TAX-1",
        notes: "Preferred",
        isActive: true,
        revision: 1,
        createdAt: now,
        updatedAt: now,
      },
      {
        id: "s2",
        code: "SUP-000002",
        displayName: "Delta Pharma",
        contactPerson: null,
        primaryPhone: null,
        secondaryPhone: null,
        email: null,
        address: null,
        taxRegistrationId: null,
        notes: null,
        isActive: false,
        revision: 2,
        createdAt: now,
        updatedAt: now,
      },
    ];
    const customers: Partner[] = Array.from({ length: 25 }, (_, index) => ({
      id: `c${index + 1}`,
      code: `CUS-${String(index + 1).padStart(6, "0")}`,
      displayName:
        index === 0
          ? "Ahmed Hassan"
          : `Customer ${String(index + 1).padStart(2, "0")}`,
      primaryPhone: index === 0 ? "01100000001" : null,
      secondaryPhone: null,
      email: index === 0 ? "ahmed@example.test" : null,
      address: null,
      notes: null,
      isActive: index !== 1,
      revision: 1,
      createdAt: now,
      updatedAt: now,
    }));
    w.partnerCalls = [];
    w.partnerFailCommand = "";
    w.partnerDelay = 0;
    w.partnerEmpty = false;
    const list = (rows: Partner[], query: Record<string, unknown>) => {
      const search = String(query.search ?? "").toLowerCase();
      const active = query.isActive;
      let result = w.partnerEmpty
        ? []
        : rows.filter(
            (row) =>
              (active == null || row.isActive === active) &&
              (!search ||
                `${row.code} ${row.displayName} ${row.primaryPhone ?? ""} ${row.email ?? ""} ${row.contactPerson ?? ""} ${row.taxRegistrationId ?? ""}`
                  .toLowerCase()
                  .includes(search)),
          );
      result = result.sort((a, b) =>
        a.displayName.localeCompare(b.displayName),
      );
      const offset = Number(query.offset ?? 0),
        limit = Number(query.limit ?? 20);
      return {
        items: result.slice(offset, offset + limit),
        total: result.length,
        limit,
        offset,
      };
    };
    const find = (rows: Partner[], id: unknown) => {
      const row = rows.find((item) => item.id === String(id));
      if (!row) throw { code: "notFound", message: "Partner not found." };
      return row;
    };
    w.__TAURI_INTERNALS__.invoke = async (command, args = {}) => {
      w.partnerCalls.push({ command, args: structuredClone(args) });
      if (command === "recovery_status")
        return {
          laterWorkflowsEnabled: false,
          inventoryWorkflowsEnabled: true,
          stockCountsEnabled: true,
          inventoryAlertsEnabled: true,
          inventoryUiEnabled: true,
          salesEnabled: true,
          partnersEnabled: true,
        };
      if (command === w.partnerFailCommand)
        throw { code: "validation", message: "Controlled backend rejection" };
      if (command.startsWith("list_canonical_") && w.partnerDelay)
        await new Promise((resolve) => setTimeout(resolve, w.partnerDelay));
      if (command === "list_canonical_suppliers")
        return list(suppliers, args.query as Record<string, unknown>);
      if (command === "list_canonical_customers")
        return list(customers, args.query as Record<string, unknown>);
      if (command === "get_canonical_supplier")
        return structuredClone(find(suppliers, args.id));
      if (command === "get_canonical_customer")
        return structuredClone(find(customers, args.id));
      if (
        command === "create_canonical_supplier" ||
        command === "create_canonical_customer"
      ) {
        const supplier = command.endsWith("supplier"),
          rows = supplier ? suppliers : customers;
        const input = args.input as Record<string, string | null>;
        const row: Partner = {
          id: `${supplier ? "s" : "c"}${rows.length + 1}`,
          code: `${supplier ? "SUP" : "CUS"}-${String(rows.length + 1).padStart(6, "0")}`,
          displayName: String(input.displayName),
          primaryPhone: input.primaryPhone,
          secondaryPhone: input.secondaryPhone,
          email: input.email,
          address: input.address,
          notes: input.notes,
          contactPerson: supplier ? input.contactPerson : undefined,
          taxRegistrationId: supplier ? input.taxRegistrationId : undefined,
          isActive: true,
          revision: 1,
          createdAt: now,
          updatedAt: now,
        };
        rows.push(row);
        return structuredClone(row);
      }
      if (
        command === "update_canonical_supplier" ||
        command === "update_canonical_customer"
      ) {
        const supplier = command.endsWith("supplier"),
          input = args.input as Record<string, unknown>;
        const row = find(supplier ? suppliers : customers, input.id);
        if (row.revision !== input.expectedRevision)
          throw {
            code: "conflict",
            message: "Partner changed. Reload before saving.",
          };
        Object.assign(row, input, {
          revision: row.revision + 1,
          updatedAt: now,
        });
        return structuredClone(row);
      }
      if (
        command === "set_canonical_supplier_active" ||
        command === "set_canonical_customer_active"
      ) {
        const supplier = command.includes("supplier"),
          input = args.input as {
            id: string;
            expectedRevision: number;
            active: boolean;
          };
        const row = find(supplier ? suppliers : customers, input.id);
        if (row.revision !== input.expectedRevision)
          throw {
            code: "conflict",
            message: "Partner changed. Reload before changing status.",
          };
        row.isActive = input.active;
        row.revision++;
        row.updatedAt = now;
        return structuredClone(row);
      }
      return original(command, args);
    };
  });
}
