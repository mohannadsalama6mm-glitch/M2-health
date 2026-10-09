import type { User } from "./tauri/auth";

export type Permission =
  | "session.self" | "users.read" | "users.manage" | "roles.read" | "roles.manage"
  | "branches.read" | "branches.manage" | "catalog.read" | "catalog.manage" | "catalog.import"
  | "inventory.read" | "inventory.adjust" | "inventory.receive" | "inventory.count" | "inventory.alert.manage"
  | "sales.read" | "sales.create" | "sales.return" | "sales.void"
  | "purchasing.read" | "purchasing.manage" | "purchasing.receive" | "purchasing.return"
  | "suppliers.read" | "suppliers.manage" | "customers.read" | "customers.manage"
  | "reports.operational.read" | "reports.finance.read" | "reports.export"
  | "finance.read" | "finance.expense.manage" | "dashboard.read";

const grants: Record<string, Set<Permission>> = {
  PHARMACIST: new Set(["dashboard.read", "sales.read", "sales.create", "sales.return", "catalog.read", "inventory.read", "inventory.adjust", "inventory.alert.manage", "customers.read", "customers.manage", "reports.operational.read", "branches.read"]),
  CASHIER: new Set(["dashboard.read", "sales.read", "sales.create", "sales.return", "catalog.read", "inventory.read", "customers.read", "customers.manage", "branches.read"]),
  INVENTORY_MANAGER: new Set(["dashboard.read", "catalog.read", "catalog.manage", "catalog.import", "inventory.read", "inventory.adjust", "inventory.receive", "inventory.count", "inventory.alert.manage", "purchasing.read", "purchasing.manage", "purchasing.receive", "purchasing.return", "suppliers.read", "suppliers.manage", "reports.operational.read", "branches.read"]),
  VIEWER: new Set(["dashboard.read", "sales.read", "catalog.read", "inventory.read", "purchasing.read", "suppliers.read", "customers.read", "reports.operational.read", "branches.read"]),
};

export function userHasPermission(user: User | null, permission: Permission): boolean {
  if (!user) return false;
  return user.roles.some((role) => role.code === "ADMIN" || grants[role.code]?.has(permission));
}
