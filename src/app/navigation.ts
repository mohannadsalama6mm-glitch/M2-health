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
export const navigation = [
  {
    group: "",
    items: [{ label: "Dashboard", to: "/dashboard", icon: LayoutDashboard, permission: "dashboard.read" as const }],
  },
  {
    group: "Operations",
    items: [
      { label: "POS / Sales", to: "/sales", icon: ShoppingCart, permission: "sales.read" as const },
      { label: "Purchases", to: "/purchases", icon: Truck, permission: "purchasing.read" as const },
    ],
  },
  {
    group: "Catalog & stock",
    items: [
      { label: "Products", to: "/catalog", icon: Package, permission: "catalog.read" as const },
      { label: "Inventory", to: "/inventory", icon: Boxes, permission: "inventory.read" as const },
      { label: "Expiry", to: "/inventory/expiry", icon: CalendarClock, permission: "inventory.read" as const },
    ],
  },
  {
    group: "Relationships",
    items: [
      { label: "Suppliers", to: "/suppliers", icon: Building2, permission: "suppliers.read" as const },
      { label: "Customers", to: "/customers", icon: Users, permission: "customers.read" as const },
    ],
  },
  {
    group: "Management",
    items: [
      { label: "Reports", to: "/reports", icon: ChartNoAxesCombined, permission: "reports.operational.read" as const },
      { label: "Finance", to: "/finance", icon: Wallet, permission: "finance.read" as const },
      { label: "Employees", to: "/employees", icon: UserRoundCog, permission: "users.read" as const },
      { label: "Branches", to: "/branches", icon: Building2, permission: "branches.read" as const },
    ],
  },
  {
    group: "System",
    items: [
      { label: "Notifications", to: "/notifications", icon: Bell },
      { label: "Audit log", to: "/audit-log", icon: ScrollText, permission: "users.read" as const },
      { label: "Backup & restore", to: "/backup", icon: DatabaseBackup, permission: "users.read" as const },
      { label: "Sync status", to: "/sync", icon: Cloud },
      { label: "Settings", to: "/settings", icon: Settings },
    ],
  },
];
