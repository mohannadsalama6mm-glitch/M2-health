import { InventoryWorkspace } from "../features/inventory/InventoryWorkspace";
import { CanonicalSales } from "../features/sales/CanonicalSales";
import { CanonicalPartners } from "../features/relationships/CanonicalPartners";
import { useEffect, useState, type ReactNode } from "react";
import { Navigate, useLocation } from "react-router-dom";
import {
  Alert,
  Button,
  Card,
  LinkButton,
  PageHeader,
  Table,
} from "../design-system";
import { isDesktopRuntime } from "../lib/tauri/client";
import {
  recoveryStatus,
  coreInventory,
  coreMovements,
  type CoreBalance,
  type CoreMovement,
} from "../lib/tauri/recovery";
import { useBranch } from "./BranchContext";

/** Fail closed: an unavailable capability service never enables prototype workflows. */
export function RecoveryBoundary({ children }: { children: ReactNode }) {
  const { pathname } = useLocation();
  const [enabled, setEnabled] = useState<boolean | null>(null);
  const [documentsEnabled, setDocumentsEnabled] = useState(false);
  const [countsEnabled, setCountsEnabled] = useState(false);
  const [alertsEnabled, setAlertsEnabled] = useState(false);
  const [inventoryUiEnabled, setInventoryUiEnabled] = useState(false);
  const [salesEnabled, setSalesEnabled] = useState(false);
  const [partnersEnabled, setPartnersEnabled] = useState(false);
  const [error, setError] = useState("");
  useEffect(() => {
    if (!isDesktopRuntime()) return;
    let active = true;
    recoveryStatus()
      .then((s) => {
        if (active) {
          setEnabled(s.laterWorkflowsEnabled);
          setDocumentsEnabled(s.inventoryWorkflowsEnabled === true);
          setCountsEnabled(s.stockCountsEnabled === true);
          setAlertsEnabled(s.inventoryAlertsEnabled === true);
          setInventoryUiEnabled(s.inventoryUiEnabled === true);
          setSalesEnabled(s.salesEnabled === true);
          setPartnersEnabled(s.partnersEnabled === true);
        }
      })
      .catch((e: unknown) => {
        if (active)
          setError(
            e instanceof Error
              ? e.message
              : "Could not verify available workflows.",
          );
      });
    return () => {
      active = false;
    };
  }, []);
  const affected =
    /^\/(inventory|sales|purchases|suppliers|customers)(\/|$)/.test(pathname);
  if (salesEnabled && /^\/sales(\/|$)/.test(pathname))
    return <CanonicalSales />;
  if (partnersEnabled && /^\/(suppliers|customers)(\/|$)/.test(pathname))
    return <CanonicalPartners />;
  if (inventoryUiEnabled && pathname === "/inventory")
    return <InventoryWorkspace />;
  if (
    inventoryUiEnabled &&
    /^\/inventory\/(batches|movements|opening|levels)$/.test(pathname)
  )
    return (
      <InventoryWorkspace
        mode={
          pathname.split("/")[2] as
            "batches" | "movements" | "opening" | "levels"
        }
      />
    );
  if (!affected || !isDesktopRuntime() || enabled === true) return children;
  if (error)
    return (
      <Alert title="Workflow availability could not be verified" tone="danger">
        {error}
      </Alert>
    );
  if (enabled === null)
    return <Alert title="Checking available workflows…" tone="info" />;
  if (
    alertsEnabled &&
    /^\/inventory\/(alerts|expiry|low-stock|fefo)$/.test(pathname)
  )
    return children;
  if (documentsEnabled && /^\/inventory\/documents(\/|$)/.test(pathname))
    return children;
  if (countsEnabled && /^\/inventory\/count-sessions(\/|$)/.test(pathname))
    return children;
  if (countsEnabled && pathname === "/inventory/counts")
    return <Navigate to="/inventory/count-sessions" replace />;
  if (pathname === "/inventory" || pathname === "/inventory/movements")
    return (
      <RecoveredInventory
        key={pathname}
        movements={pathname.endsWith("/movements")}
      />
    );
  return (
    <div className="stack">
      <PageHeader
        title="Workflow temporarily unavailable"
        description="This workflow is awaiting integration and verification."
      />
      <Alert title="Posting is disabled" tone="info">
        Your existing records are preserved. Catalog and the verified Inventory
        foundation remain available.
      </Alert>
    </div>
  );
}

function RecoveredInventory({ movements }: { movements: boolean }) {
  const { branchId, loading: branchLoading } = useBranch();
  const [page, setPage] = useState(0);
  const [data, setData] = useState<{
    branch: string;
    page: number;
    balances: CoreBalance[];
    movements: CoreMovement[];
  } | null>(null);
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    if (branchLoading || !branchId) return;
    let active = true;
    Promise.all([
      coreInventory(branchId, page * 20),
      coreMovements(branchId, page * 20),
    ])
      .then(([balances, rows]) => {
        if (active) {
          setData({ branch: branchId, page, balances, movements: rows });
          setError("");
        }
      })
      .catch((e: unknown) => {
        if (active)
          setError(
            e instanceof Error ? e.message : "Inventory could not be loaded.",
          );
      });
    return () => {
      active = false;
    };
  }, [branchId, branchLoading, page, attempt]);
  const current = data?.branch === branchId && data.page === page ? data : null;
  return (
    <div className="stack">
      <PageHeader
        title={movements ? "Stock Movements" : "Inventory"}
        description="Verified stock balances and movement history for the selected branch."
      />
      <LinkButton to="/inventory/alerts">
        Inventory alerts and monitoring
      </LinkButton>
      <LinkButton to="/inventory/documents">Inventory documents</LinkButton>
      <LinkButton to="/inventory/count-sessions">
        Stock count sessions
      </LinkButton>
      {error && (
        <Alert title="Inventory could not be loaded" tone="danger">
          {error}
        </Alert>
      )}
      <Card>
        {!current ? (
          <p role="status">Loading inventory…</p>
        ) : movements ? (
          <Table
            label="Verified stock movements"
            rows={current.movements}
            rowKey={(r) => r.id}
            columns={[
              {
                key: "type",
                header: "Movement",
                render: (r) => r.movementType,
              },
              {
                key: "quantity",
                header: "Quantity",
                render: (r) => r.quantityDelta,
              },
              { key: "reason", header: "Reason", render: (r) => r.reason },
              { key: "date", header: "Date", render: (r) => r.createdAt },
            ]}
          />
        ) : (
          <Table
            label="Verified inventory balances"
            rows={current.balances}
            rowKey={(r) => r.productPackageId}
            columns={[
              {
                key: "package",
                header: "Package ID",
                render: (r) => r.productPackageId,
              },
              {
                key: "quantity",
                header: "Current stock",
                render: (r) => r.quantity,
              },
            ]}
          />
        )}
        <div className="grid-footer">
          <Button disabled={page === 0} onClick={() => setPage(page - 1)}>
            Previous
          </Button>
          <span>Page {page + 1}</span>
          <Button
            disabled={
              !current ||
              (movements ? current.movements : current.balances).length < 20
            }
            onClick={() => setPage(page + 1)}
          >
            Next
          </Button>
          <Button onClick={() => setAttempt(attempt + 1)}>Refresh</Button>
        </div>
      </Card>
    </div>
  );
}
