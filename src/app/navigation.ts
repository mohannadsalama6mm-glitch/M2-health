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
    items: [{ label: "Dashboard", to: "/dashboard", icon: LayoutDashboard }],
  },
  {
    group: "Operations",
    items: [
      { label: "POS / Sales", to: "/sales", icon: ShoppingCart },
      { label: "Purchases", to: "/purchases", icon: Truck },
    ],
  },
  {
    group: "Catalog & stock",
    items: [
      { label: "Products", to: "/catalog", icon: Package },
      { label: "Inventory", to: "/inventory", icon: Boxes },
      { label: "Expiry", to: "/inventory/expiry", icon: CalendarClock },
    ],
  },
  {
    group: "Relationships",
    items: [
      { label: "Suppliers", to: "/suppliers", icon: Building2 },
      { label: "Customers", to: "/customers", icon: Users },
    ],
  },
  {
    group: "Management",
    items: [
      { label: "Reports", to: "/reports", icon: ChartNoAxesCombined },
      { label: "Finance", to: "/finance", icon: Wallet },
      { label: "Employees", to: "/employees", icon: UserRoundCog },
      { label: "Branches", to: "/branches", icon: Building2 },
    ],
  },
  {
    group: "System",
    items: [
      { label: "Notifications", to: "/notifications", icon: Bell },
      { label: "Audit log", to: "/audit-log", icon: ScrollText },
      { label: "Backup & restore", to: "/backup", icon: DatabaseBackup },
      { label: "Sync status", to: "/sync", icon: Cloud },
      { label: "Settings", to: "/settings", icon: Settings },
    ],
  },
];
