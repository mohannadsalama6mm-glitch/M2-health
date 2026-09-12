import { Navigate, Route, Routes, useNavigate } from "react-router-dom";
import { Dashboard } from "../features/dashboard/Dashboard";
import { DesignSystem } from "../features/design-system/DesignSystem";
import { Catalog } from "../features/catalog/Catalog";
import { ProductDetails } from "../features/catalog/ProductDetails";
import { ProductForm } from "../features/catalog/ProductForm";
import { Sales, SaleDetails } from "../features/sales/Sales";
import { Inventory, StockMovements } from "../features/inventory/Inventory";
import { StockCounts } from "../features/inventory/StockCounts";
import { Expiry } from "../features/inventory/Expiry";
import {
  Purchases,
  PurchaseDetails,
  PurchaseForm,
  ReceivePurchase,
} from "../features/purchases/Purchases";
import { Partners, PartnerDetails } from "../features/relationships/Partners";
import { Reports } from "../features/reports/Reports";
import { Finance } from "../features/finance/Finance";
import {
  Employees,
  EmployeeDetails,
  Roles,
} from "../features/management/People";
import { Branches, BranchDetails } from "../features/management/Branches";
import { Notifications, AuditLog } from "../features/system/Notifications";
import { Backup, Sync } from "../features/system/BackupSync";
import { Settings } from "../features/system/Settings";
import { EmptyState, LinkButton } from "../design-system";
import { useDemo } from "./DemoContext";
function DashboardRoute() {
  const navigate = useNavigate();
  const { notify } = useDemo();
  const destinations: Record<string, string> = {
    "New sale": "/sales",
    "Add product": "/catalog/new",
    "Receive purchase": "/purchases/PO-1048/receive",
    "Stock count": "/inventory/counts",
    "Print barcode": "/settings?section=Barcode",
    "View reports": "/reports",
    "Sales history": "/sales",
    "Low stock inventory": "/inventory",
    "Product performance": "/reports",
    "Expiry report": "/inventory/expiry",
  };
  return (
    <Dashboard
      onAction={(label) =>
        destinations[label]
          ? navigate(destinations[label])
          : notify("Dashboard date is fixed to the sample reporting period.")
      }
    />
  );
}
export function AppRoutes() {
  const { notify } = useDemo();
  return (
    <Routes>
      <Route path="/" element={<Navigate to="/dashboard" replace />} />
      <Route path="/dashboard" element={<DashboardRoute />} />
      <Route path="/sales" element={<Sales />} />
      <Route path="/sales/:id" element={<SaleDetails />} />
      <Route path="/catalog" element={<Catalog />} />
      <Route path="/catalog/new" element={<ProductForm key="new" />} />
      <Route path="/catalog/:id" element={<ProductDetails />} />
      <Route path="/catalog/:id/edit" element={<ProductForm />} />
      <Route path="/inventory" element={<Inventory />} />
      <Route path="/inventory/movements" element={<StockMovements />} />
      <Route path="/inventory/counts" element={<StockCounts />} />
      <Route path="/inventory/expiry" element={<Expiry />} />
      <Route path="/purchases" element={<Purchases />} />
      <Route path="/purchases/new" element={<PurchaseForm />} />
      <Route path="/purchases/:id" element={<PurchaseDetails />} />
      <Route path="/purchases/:id/receive" element={<ReceivePurchase />} />
      <Route path="/suppliers" element={<Partners kind="suppliers" />} />
      <Route
        path="/suppliers/:id"
        element={<PartnerDetails kind="suppliers" />}
      />
      <Route path="/customers" element={<Partners kind="customers" />} />
      <Route
        path="/customers/:id"
        element={<PartnerDetails kind="customers" />}
      />
      <Route path="/reports" element={<Reports />} />
      <Route path="/finance" element={<Finance />} />
      <Route path="/employees" element={<Employees />} />
      <Route path="/employees/:id" element={<EmployeeDetails />} />
      <Route path="/roles" element={<Roles />} />
      <Route path="/branches" element={<Branches />} />
      <Route path="/branches/:id" element={<BranchDetails />} />
      <Route path="/notifications" element={<Notifications />} />
      <Route path="/audit-log" element={<AuditLog />} />
      <Route path="/backup" element={<Backup />} />
      <Route path="/sync" element={<Sync />} />
      <Route path="/settings" element={<Settings />} />
      {import.meta.env.DEV && (
        <Route
          path="/design-system"
          element={<DesignSystem notify={notify} />}
        />
      )}
      <Route
        path="*"
        element={
          <EmptyState
            title="Workspace not found"
            description="Choose a module from the sidebar or return to the dashboard."
            action={<LinkButton to="/dashboard">Dashboard</LinkButton>}
          />
        }
      />
    </Routes>
  );
}
