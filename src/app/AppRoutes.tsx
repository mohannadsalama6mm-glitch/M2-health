import { InventoryWorkspace } from "../features/inventory/InventoryWorkspace";
import { Navigate, Route, Routes, useNavigate } from "react-router-dom";
import { Dashboard } from "../features/dashboard/Dashboard";
import { CanonicalDashboard } from "../features/dashboard/CanonicalDashboard";
import { DesignSystem } from "../features/design-system/DesignSystem";
import { Catalog } from "../features/catalog/Catalog";
import { ImportProducts } from "../features/catalog/ImportProducts";
import { ProductDetails } from "../features/catalog/ProductDetails";
import { ProductForm } from "../features/catalog/ProductForm";
import { Sales, SaleDetails } from "../features/sales/Sales";
import { CanonicalSales } from "../features/sales/CanonicalSales";
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
import { CanonicalPurchases } from "../features/purchases/CanonicalPurchases";
import { Partners, PartnerDetails } from "../features/relationships/Partners";
import { CanonicalPartners } from "../features/relationships/CanonicalPartners";
import { Reports } from "../features/reports/Reports";
import { CanonicalReports } from "../features/reports/CanonicalReports";
import { Finance } from "../features/finance/Finance";
import { CanonicalFinance } from "../features/finance/CanonicalFinance";
import {
  Employees,
  EmployeeDetails,
  Roles,
} from "../features/management/People";
import { Branches, BranchDetails } from "../features/management/Branches";
import { CanonicalSync } from "../features/system/CanonicalSync";
import {
  CanonicalBranches,
  CanonicalBranchDetails,
} from "../features/management/CanonicalBranches";
import {
  CanonicalAuditLog,
  CanonicalBackup,
  CanonicalNotifications,
  CanonicalSettings,
} from "../features/system/CanonicalDesktopSystem";
import { EmptyState, LinkButton } from "../design-system";
import { useFeedback } from "./FeedbackContext";
import { RecoveryBoundary } from "./RecoveryBoundary";
import { isDesktopRuntime } from "../lib/tauri/client";
import { useAuth } from "./AuthContext";
import {
  CanonicalEmployees,
  CanonicalEmployeeDetails,
  CanonicalRoles,
} from "../features/management/CanonicalPeople";
import type { ReactNode } from "react";
import type { Permission } from "../lib/permissions";
function Authorized({
  permission,
  children,
}: {
  permission: Permission;
  children: ReactNode;
}) {
  const auth = useAuth();
  if (!auth.hasPermission(permission))
    return (
      <EmptyState
        title="Access denied"
        description="Your role does not permit this workspace."
      />
    );
  return <>{children}</>;
}
function DashboardRoute() {
  if (isDesktopRuntime()) return <CanonicalDashboard />;
  return <BrowserDashboardRoute />;
}
function BrowserDashboardRoute() {
  const navigate = useNavigate();
  const notify = useFeedback();
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
function DesktopRequired({ title }: { title: string }) {
  return (
    <EmptyState
      title={`${title} requires the desktop app`}
      description="This browser preview does not open, change, or simulate the local SQLite database. Install and sign in to M² Health desktop to use this workflow."
    />
  );
}
export function AppRoutes() {
  const notify = useFeedback();
  const desktop = isDesktopRuntime();
  return (
    <RecoveryBoundary>
      <Routes>
        <Route path="/" element={<Navigate to="/dashboard" replace />} />
        <Route
          path="/dashboard"
          element={
            <Authorized permission="dashboard.read">
              <DashboardRoute />
            </Authorized>
          }
        />
        {desktop ? (
          <Route
            path="/sales/*"
            element={
              <Authorized permission="sales.read">
                <CanonicalSales />
              </Authorized>
            }
          />
        ) : (
          <>
            <Route
              path="/sales"
              element={
                <Authorized permission="sales.read">
                  <Sales />
                </Authorized>
              }
            />
            <Route
              path="/sales/:id"
              element={
                <Authorized permission="sales.read">
                  <SaleDetails />
                </Authorized>
              }
            />
          </>
        )}
        <Route
          path="/catalog"
          element={
            <Authorized permission="catalog.read">
              <Catalog />
            </Authorized>
          }
        />
        <Route
          path="/catalog/import"
          element={
            <Authorized permission="catalog.import">
              <ImportProducts />
            </Authorized>
          }
        />
        <Route
          path="/catalog/new"
          element={
            <Authorized permission="catalog.manage">
              <ProductForm key="new" />
            </Authorized>
          }
        />
        <Route
          path="/catalog/:id"
          element={
            <Authorized permission="catalog.read">
              <ProductDetails />
            </Authorized>
          }
        />
        <Route
          path="/catalog/:id/edit"
          element={
            <Authorized permission="catalog.manage">
              <ProductForm />
            </Authorized>
          }
        />
        <Route
          path="/inventory"
          element={
            <Authorized permission="inventory.read">
              <Inventory />
            </Authorized>
          }
        />
        <Route
          path="/inventory/batches"
          element={
            <Authorized permission="inventory.read">
              <InventoryWorkspace mode="batches" />
            </Authorized>
          }
        />
        <Route
          path="/inventory/opening"
          element={
            <Authorized permission="inventory.adjust">
              <InventoryWorkspace mode="opening" />
            </Authorized>
          }
        />
        <Route
          path="/inventory/levels"
          element={
            <Authorized permission="inventory.adjust">
              <InventoryWorkspace mode="levels" />
            </Authorized>
          }
        />
        <Route
          path="/inventory/documents"
          element={
            <Authorized permission="inventory.read">
              <InventoryDocuments />
            </Authorized>
          }
        />
        <Route
          path="/inventory/documents/:id"
          element={
            <Authorized permission="inventory.read">
              <InventoryDocuments />
            </Authorized>
          }
        />
        <Route
          path="/inventory/count-sessions"
          element={
            <Authorized permission="inventory.read">
              <CountSessions />
            </Authorized>
          }
        />
        <Route
          path="/inventory/count-sessions/:id"
          element={
            <Authorized permission="inventory.read">
              <CountSessions />
            </Authorized>
          }
        />
        <Route
          path="/inventory/movements"
          element={
            <Authorized permission="inventory.read">
              <StockMovements />
            </Authorized>
          }
        />
        <Route
          path="/inventory/counts"
          element={
            <Authorized permission="inventory.count">
              <StockCounts />
            </Authorized>
          }
        />
        <Route
          path="/inventory/expiry"
          element={
            <Authorized permission="inventory.read">
              <InventoryAlerts mode="expiry" />
            </Authorized>
          }
        />
        <Route
          path="/inventory/alerts"
          element={
            <Authorized permission="inventory.read">
              <InventoryAlerts mode="alerts" />
            </Authorized>
          }
        />
        <Route
          path="/inventory/low-stock"
          element={
            <Authorized permission="inventory.read">
              <InventoryAlerts mode="low-stock" />
            </Authorized>
          }
        />
        <Route
          path="/inventory/fefo"
          element={
            <Authorized permission="inventory.read">
              <InventoryAlerts mode="fefo" />
            </Authorized>
          }
        />
        {desktop ? (
          <Route
            path="/purchases/*"
            element={
              <Authorized permission="purchasing.read">
                <CanonicalPurchases />
              </Authorized>
            }
          />
        ) : (
          <>
            <Route
              path="/purchases"
              element={
                <Authorized permission="purchasing.read">
                  <Purchases />
                </Authorized>
              }
            />
            <Route
              path="/purchases/new"
              element={
                <Authorized permission="purchasing.manage">
                  <PurchaseForm />
                </Authorized>
              }
            />
            <Route
              path="/purchases/:id"
              element={
                <Authorized permission="purchasing.read">
                  <PurchaseDetails />
                </Authorized>
              }
            />
            <Route
              path="/purchases/:id/receive"
              element={
                <Authorized permission="purchasing.receive">
                  <ReceivePurchase />
                </Authorized>
              }
            />
          </>
        )}
        {desktop ? (
          <>
            <Route
              path="/suppliers/*"
              element={
                <Authorized permission="suppliers.read">
                  <CanonicalPartners />
                </Authorized>
              }
            />
            <Route
              path="/customers/*"
              element={
                <Authorized permission="customers.read">
                  <CanonicalPartners />
                </Authorized>
              }
            />
          </>
        ) : (
          <>
            <Route
              path="/suppliers"
              element={
                <Authorized permission="suppliers.read">
                  <Partners kind="suppliers" />
                </Authorized>
              }
            />
            <Route
              path="/suppliers/:id"
              element={
                <Authorized permission="suppliers.read">
                  <PartnerDetails kind="suppliers" />
                </Authorized>
              }
            />
            <Route
              path="/customers"
              element={
                <Authorized permission="customers.read">
                  <Partners kind="customers" />
                </Authorized>
              }
            />
            <Route
              path="/customers/:id"
              element={
                <Authorized permission="customers.read">
                  <PartnerDetails kind="customers" />
                </Authorized>
              }
            />
          </>
        )}
        <Route
          path="/reports"
          element={
            <Authorized permission="reports.operational.read">
              {isDesktopRuntime() ? <CanonicalReports /> : <Reports />}
            </Authorized>
          }
        />
        <Route
          path="/finance/*"
          element={
            <Authorized permission="finance.read">
              {isDesktopRuntime() ? <CanonicalFinance /> : <Finance />}
            </Authorized>
          }
        />
        <Route
          path="/employees"
          element={
            <Authorized permission="users.read">
              {desktop ? <CanonicalEmployees /> : <Employees />}
            </Authorized>
          }
        />
        <Route
          path="/employees/:id"
          element={
            <Authorized permission="users.read">
              {desktop ? <CanonicalEmployeeDetails /> : <EmployeeDetails />}
            </Authorized>
          }
        />
        <Route
          path="/roles"
          element={
            <Authorized permission="roles.read">
              {desktop ? <CanonicalRoles /> : <Roles />}
            </Authorized>
          }
        />
        <Route
          path="/branches"
          element={
            <Authorized permission="branches.read">
              {isDesktopRuntime() ? <CanonicalBranches /> : <Branches />}
            </Authorized>
          }
        />
        <Route
          path="/branches/:id"
          element={
            <Authorized permission="branches.read">
              {isDesktopRuntime() ? (
                <CanonicalBranchDetails />
              ) : (
                <BranchDetails />
              )}
            </Authorized>
          }
        />
        <Route
          path="/notifications"
          element={
            <Authorized permission="inventory.read">
              {desktop ? (
                <CanonicalNotifications />
              ) : (
                <DesktopRequired title="Notifications" />
              )}
            </Authorized>
          }
        />
        <Route
          path="/audit-log"
          element={
            <Authorized permission="audit.read">
              {desktop ? (
                <CanonicalAuditLog />
              ) : (
                <DesktopRequired title="Security audit log" />
              )}
            </Authorized>
          }
        />
        <Route
          path="/backup"
          element={
            <Authorized permission="backup.manage">
              {desktop ? (
                <CanonicalBackup />
              ) : (
                <DesktopRequired title="Backup and restore" />
              )}
            </Authorized>
          }
        />
        <Route
          path="/sync"
          element={
            desktop ? (
              <CanonicalSync />
            ) : (
              <DesktopRequired title="Sync status" />
            )
          }
        />
        <Route
          path="/settings"
          element={
            <Authorized permission="settings.read">
              {desktop ? (
                <CanonicalSettings />
              ) : (
                <DesktopRequired title="Settings" />
              )}
            </Authorized>
          }
        />
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
    </RecoveryBoundary>
  );
}
