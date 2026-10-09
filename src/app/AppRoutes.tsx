import { InventoryWorkspace } from "../features/inventory/InventoryWorkspace";
import { Navigate, Route, Routes, useNavigate } from "react-router-dom";
import { Dashboard } from "../features/dashboard/Dashboard";
import { DesignSystem } from "../features/design-system/DesignSystem";
import { Catalog } from "../features/catalog/Catalog";
import { ImportProducts } from "../features/catalog/ImportProducts";
import { ProductDetails } from "../features/catalog/ProductDetails";
import { ProductForm } from "../features/catalog/ProductForm";
import { Sales, SaleDetails } from "../features/sales/Sales";
import { Inventory, StockMovements } from "../features/inventory/Inventory";
import { InventoryDocuments } from "../features/inventory/Documents";
import { CountSessions } from "../features/inventory/CountSessions";
import { StockCounts } from "../features/inventory/StockCounts";
import { InventoryAlerts } from "../features/inventory/InventoryAlerts";
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
import { RecoveryBoundary } from "./RecoveryBoundary";
import { isDesktopRuntime } from "../lib/tauri/client";
import { useAuth } from "./AuthContext";
import { CanonicalEmployees, CanonicalEmployeeDetails, CanonicalRoles } from "../features/management/CanonicalPeople";
import type { ReactNode } from "react";
import type { Permission } from "../lib/permissions";
function Authorized({ permission, children }: { permission: Permission; children: ReactNode }) {
  const auth = useAuth();
  if (!auth.hasPermission(permission)) return <EmptyState title="Access denied" description="Your role does not permit this workspace." />;
  return <>{children}</>;
}
function NativeUnavailable({ area }: { area: string }) {
  return <EmptyState title={`${area} unavailable`} description="This workspace is a design preview and is not connected to persisted desktop data." />;
}
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
  const auth = useAuth();
  const canonicalIdentity = isDesktopRuntime() && auth.enabled;
  return (
    <RecoveryBoundary><Routes>
      <Route path="/" element={<Navigate to="/dashboard" replace />} />
      <Route path="/dashboard" element={<Authorized permission="dashboard.read"><DashboardRoute /></Authorized>} />
      <Route path="/sales" element={<Authorized permission="sales.read"><Sales /></Authorized>} />
      <Route path="/sales/:id" element={<Authorized permission="sales.read"><SaleDetails /></Authorized>} />
      <Route path="/catalog" element={<Authorized permission="catalog.read"><Catalog /></Authorized>} />
      <Route path="/catalog/import" element={<Authorized permission="catalog.import"><ImportProducts /></Authorized>} />
      <Route path="/catalog/new" element={<Authorized permission="catalog.manage"><ProductForm key="new" /></Authorized>} />
      <Route path="/catalog/:id" element={<Authorized permission="catalog.read"><ProductDetails /></Authorized>} />
      <Route path="/catalog/:id/edit" element={<Authorized permission="catalog.manage"><ProductForm /></Authorized>} />
      <Route path="/inventory" element={<Authorized permission="inventory.read"><Inventory /></Authorized>} /><Route path="/inventory/batches" element={<Authorized permission="inventory.read"><InventoryWorkspace mode="batches" /></Authorized>} /><Route path="/inventory/opening" element={<Authorized permission="inventory.adjust"><InventoryWorkspace mode="opening" /></Authorized>} /><Route path="/inventory/levels" element={<Authorized permission="inventory.adjust"><InventoryWorkspace mode="levels" /></Authorized>} />
      <Route path="/inventory/documents" element={<Authorized permission="inventory.read"><InventoryDocuments /></Authorized>} />
      <Route path="/inventory/documents/:id" element={<Authorized permission="inventory.read"><InventoryDocuments /></Authorized>} />
      <Route path="/inventory/count-sessions" element={<Authorized permission="inventory.read"><CountSessions /></Authorized>} />
      <Route path="/inventory/count-sessions/:id" element={<Authorized permission="inventory.read"><CountSessions /></Authorized>} />
      <Route path="/inventory/movements" element={<Authorized permission="inventory.read"><StockMovements /></Authorized>} />
      <Route path="/inventory/counts" element={<Authorized permission="inventory.count"><StockCounts /></Authorized>} />
      <Route path="/inventory/expiry" element={<Authorized permission="inventory.read"><InventoryAlerts mode="expiry" /></Authorized>} /><Route path="/inventory/alerts" element={<Authorized permission="inventory.read"><InventoryAlerts mode="alerts" /></Authorized>} /><Route path="/inventory/low-stock" element={<Authorized permission="inventory.read"><InventoryAlerts mode="low-stock" /></Authorized>} /><Route path="/inventory/fefo" element={<Authorized permission="inventory.read"><InventoryAlerts mode="fefo" /></Authorized>} />
      <Route path="/purchases" element={<Authorized permission="purchasing.read"><Purchases /></Authorized>} />
      <Route path="/purchases/new" element={<Authorized permission="purchasing.manage"><PurchaseForm /></Authorized>} />
      <Route path="/purchases/:id" element={<Authorized permission="purchasing.read"><PurchaseDetails /></Authorized>} />
      <Route path="/purchases/:id/receive" element={<Authorized permission="purchasing.receive"><ReceivePurchase /></Authorized>} />
      <Route path="/suppliers" element={<Authorized permission="suppliers.read"><Partners kind="suppliers" /></Authorized>} />
      <Route
        path="/suppliers/:id"
        element={<Authorized permission="suppliers.read"><PartnerDetails kind="suppliers" /></Authorized>}
      />
      <Route path="/customers" element={<Authorized permission="customers.read"><Partners kind="customers" /></Authorized>} />
      <Route
        path="/customers/:id"
        element={<Authorized permission="customers.read"><PartnerDetails kind="customers" /></Authorized>}
      />
      <Route path="/reports" element={<Authorized permission="reports.operational.read"><Reports /></Authorized>} />
      <Route path="/finance/*" element={<Authorized permission="finance.read"><Finance /></Authorized>} />
      <Route path="/employees" element={<Authorized permission="users.read">{canonicalIdentity ? <CanonicalEmployees /> : <Employees />}</Authorized>} />
      <Route path="/employees/:id" element={<Authorized permission="users.read">{canonicalIdentity ? <CanonicalEmployeeDetails /> : <EmployeeDetails />}</Authorized>} />
      <Route path="/roles" element={<Authorized permission="roles.read">{canonicalIdentity ? <CanonicalRoles /> : <Roles />}</Authorized>} />
      <Route path="/branches" element={<Authorized permission="branches.read"><Branches /></Authorized>} />
      <Route path="/branches/:id" element={<Authorized permission="branches.read"><BranchDetails /></Authorized>} />
      <Route path="/notifications" element={isDesktopRuntime() ? <NativeUnavailable area="Notifications" /> : <Notifications />} />
      <Route path="/audit-log" element={isDesktopRuntime() ? <NativeUnavailable area="Audit log" /> : <AuditLog />} />
      <Route path="/backup" element={isDesktopRuntime() ? <NativeUnavailable area="Backup and restore" /> : <Backup />} />
      <Route path="/sync" element={isDesktopRuntime() ? <NativeUnavailable area="Synchronization" /> : <Sync />} />
      <Route path="/settings" element={isDesktopRuntime() ? <NativeUnavailable area="Settings" /> : <Settings />} />
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
    </Routes></RecoveryBoundary>
  );
}


