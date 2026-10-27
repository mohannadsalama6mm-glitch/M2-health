import {
  LayoutDashboard,
  ShoppingCart,
  Package,
  Boxes,
  Truck,
  Users,
  Building2,
  ChartNoAxesCombined,
  Settings,
  Bell,
  Wallet,
  UserRoundCog,
  DatabaseBackup,
  Cloud,
  ScrollText,
  CalendarClock,
} from "lucide-react";
import type { MessageKey } from "../../localization";
type NavigationItem = {
  labelKey: MessageKey;
  to: string;
  icon: typeof LayoutDashboard;
  permission?: "dashboard.read" | "sales.read" | "purchasing.read" | "catalog.read" | "inventory.read" | "suppliers.read" | "customers.read" | "reports.operational.read" | "finance.read" | "users.read" | "branches.read" | "audit.read" | "backup.manage" | "settings.read";
};
export type NavigationPriority = "primary" | "secondary" | "utility";
type NavigationGroup = { groupKey?: MessageKey; priority: NavigationPriority; items: NavigationItem[] };
export const navigation: NavigationGroup[] = [
  {
    priority: "primary",
    items: [{ labelKey: "nav.dashboard", to: "/dashboard", icon: LayoutDashboard, permission: "dashboard.read" }],
  },
  {
    groupKey: "nav.operations",
    priority: "primary",
    items: [
      { labelKey: "nav.sales", to: "/sales", icon: ShoppingCart, permission: "sales.read" },
      { labelKey: "nav.purchases", to: "/purchases", icon: Truck, permission: "purchasing.read" },
    ],
  },
  {
    groupKey: "nav.catalogStock",
    priority: "primary",
    items: [
      { labelKey: "nav.products", to: "/catalog", icon: Package, permission: "catalog.read" },
      { labelKey: "nav.inventory", to: "/inventory", icon: Boxes, permission: "inventory.read" },
      { labelKey: "nav.expiry", to: "/inventory/expiry", icon: CalendarClock, permission: "inventory.read" },
    ],
  },
  {
    groupKey: "nav.relationships",
    priority: "secondary",
    items: [
      { labelKey: "nav.suppliers", to: "/suppliers", icon: Building2, permission: "suppliers.read" },
      { labelKey: "nav.customers", to: "/customers", icon: Users, permission: "customers.read" },
    ],
  },
  {
    groupKey: "nav.insights",
    priority: "secondary",
    items: [
      { labelKey: "nav.reports", to: "/reports", icon: ChartNoAxesCombined, permission: "reports.operational.read" },
      { labelKey: "nav.finance", to: "/finance", icon: Wallet, permission: "finance.read" },
    ],
  },
  {
    groupKey: "nav.administration",
    priority: "utility",
    items: [
      { labelKey: "nav.employees", to: "/employees", icon: UserRoundCog, permission: "users.read" },
      { labelKey: "nav.branches", to: "/branches", icon: Building2, permission: "branches.read" },
    ],
  },
  {
    groupKey: "nav.system",
    priority: "utility",
    items: [
      { labelKey: "nav.notifications", to: "/notifications", icon: Bell, permission: "inventory.read" },
      { labelKey: "nav.audit", to: "/audit-log", icon: ScrollText, permission: "audit.read" },
      { labelKey: "nav.backup", to: "/backup", icon: DatabaseBackup, permission: "backup.manage" },
      { labelKey: "nav.sync", to: "/sync", icon: Cloud },
      { labelKey: "nav.settings", to: "/settings", icon: Settings, permission: "settings.read" },
    ],
  },
];
