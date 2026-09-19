import { useSearchParams } from "react-router-dom";
import { InventoryNavigation, inventoryLink } from "./InventoryWorkspace";
import { useEffect, useState } from "react";
import { useBranch } from "../../app/BranchContext";
import {
  Alert,
  Button,
  Card,
  Input,
  Select,
  PageHeader,
  Table,
  LinkButton,
} from "../../design-system";
import { isDesktopRuntime } from "../../lib/tauri/client";
import { listProducts, getProductDetail } from "../../lib/tauri/catalog";
import type {
  ProductListItem,
  PackageDetail,
} from "../../lib/tauri/catalog.types";
import {
  getAlertSettings,
  saveAlertSettings,
  evaluateAlerts,
  listAlerts,
  getAlert,
  dismissAlert,
  listMonitor,
  inspectFefo,
  type AlertSettings,
  type InventoryAlert,
  type Monitor,
  type FefoRow,
  type AlertDetail,
  type AlertType,
  type Severity,
  type AlertStatus,
} from "../../lib/tauri/inventory-alerts";
type Mode = "alerts" | "expiry" | "low-stock" | "fefo";
const message = (e: unknown) =>
  e instanceof Error ? e.message : "Inventory monitoring could not be loaded.";
const money = (n: number | null) =>
  n === null ? "—" : `${(n / 100).toFixed(2)} EGP`;
export function InventoryAlerts({ mode }: { mode: Mode }) {
  const branch = useBranch();
  const [context] = useSearchParams();
  // Remount on scope changes so a prior branch's rows and settings cannot leak.
  return (
    <InventoryAlertsScope
      key={`${branch.branchId}:${mode}:${context.toString()}`}
      mode={mode}
    />
  );
}
function InventoryAlertsScope({ mode }: { mode: Mode }) {
  const {
    branchId,
    branchName,
    branches,
    select,
    loading: branchLoading,
  } = useBranch();
  const [context] = useSearchParams();
  const [page, setPage] = useState(0);
  const [refresh, setRefresh] = useState(0);
  const [window, setWindow] = useState("expired");
  const [out, setOut] = useState(false);
  const [type, setType] = useState<AlertType | "">("");
  const [severity, setSeverity] = useState<Severity | "">("");
  const [status, setStatus] = useState<AlertStatus | "">("ACTIVE");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [settings, setSettings] = useState<AlertSettings | null>(null);
  const [rows, setRows] = useState<InventoryAlert[]>([]);
  const [monitor, setMonitor] = useState<Monitor[]>([]);
  const [fefo, setFefo] = useState<FefoRow[]>([]);
  const [total, setTotal] = useState(0);
  const [date, setDate] = useState("");
  const [detail, setDetail] = useState<AlertDetail | null>(null);
  const [error, setError] = useState("");
  const [pending, setPending] = useState(true);
  const [busy, setBusy] = useState(false);
  const [search, setSearch] = useState("");
  const [products, setProducts] = useState<ProductListItem[]>([]);
  const [packages, setPackages] = useState<PackageDetail[]>([]);
  const [productId, setProductId] = useState(context.get("product") || "");
  const [packageId, setPackageId] = useState(context.get("package") || "");
  useEffect(() => {
    if (!isDesktopRuntime() || branchLoading || !branchId) return;
    let current = true;
    getAlertSettings(branchId)
      .then((s) => {
        if (current) setSettings(s);
      })
      .catch((e) => {
        if (current) setError(message(e));
      });
    return () => {
      current = false;
    };
  }, [branchId, branchLoading]);
  useEffect(() => {
    if (!isDesktopRuntime() || mode !== "fefo") return;
    let current = true;
    const timer = setTimeout(() => {
      listProducts({ search, limit: 20, offset: 0 })
        .then((p) => {
          if (current) setProducts(p.items);
        })
        .catch((e) => {
          if (current) setError(message(e));
        });
    }, 200);
    return () => {
      current = false;
      clearTimeout(timer);
    };
  }, [search, mode]);
  useEffect(() => {
    if (!productId) return;
    let current = true;
    getProductDetail(productId)
      .then((p) => {
        if (current) setPackages(p.packages);
      })
      .catch((e) => {
        if (current) setError(message(e));
      });
    return () => {
      current = false;
    };
  }, [productId]);
  useEffect(() => {
    if (!isDesktopRuntime() || branchLoading || !branchId) return;
    let current = true;
    async function load() {
      // Evaluate in Rust on page/explicit refresh; filters only change the DB query.
      const evaluated = await evaluateAlerts(branchId);
      if (!current) return;
      setDate(evaluated.businessDate);
      if (mode === "alerts") {
        const result = await listAlerts({
          branchId,
          alertType: type || null,
          severity: severity || null,
          status: status || null,
          fromDate: from || null,
          toDate: to || null,
          limit: 20,
          offset: page * 20,
        });
        if (current) {
          setRows(result.items);
          setTotal(result.total);
        }
      } else if (mode === "fefo") {
        const result = packageId
          ? await inspectFefo(branchId, packageId, page * 20)
          : [];
        if (current) {
          setFefo(result);
          setTotal(page * 20 + result.length + (result.length === 20 ? 1 : 0));
        }
      } else {
        const ranges: Record<string, [number, number]> = {
          expired: [0, 90],
          "30": [0, 30],
          "60": [31, 60],
          "90": [61, 90],
        };
        const [minimumDays, maximumDays] = ranges[window];
        const result = await listMonitor(
          {
            branchId,
            minimumDays,
            maximumDays,
            expired: window === "expired",
            outOfStock: out,
            limit: 20,
            offset: page * 20,
          },
          mode === "low-stock",
        );
        if (current) {
          setMonitor(result.items);
          setTotal(result.total);
        }
      }
    }
    load()
      .catch((e) => {
        if (current) setError(message(e));
      })
      .finally(() => {
        if (current) setPending(false);
      });
    return () => {
      current = false;
    };
  }, [
    branchId,
    branchLoading,
    mode,
    page,
    refresh,
    window,
    out,
    type,
    severity,
    status,
    from,
    to,
    packageId,
  ]);
  function change(action: () => void) {
    setPending(true);
    setError("");
    setDetail(null);
    setPage(0);
    action();
    setRefresh((r) => r + 1);
  }
  async function act(action: () => Promise<unknown>) {
    if (busy) return;
    setBusy(true);
    setError("");
    try {
      await action();
      setPending(true);
      setRefresh((r) => r + 1);
    } catch (e) {
      setError(message(e));
    } finally {
      setBusy(false);
    }
  }
  if (!isDesktopRuntime())
    return (
      <div className="stack">
        <PageHeader
          title="Desktop inventory monitoring"
          description="Open the desktop app to read real inventory alerts."
        />
        <Alert title="Local database required" tone="info">
          No sample alerts are shown here.
        </Alert>
      </div>
    );
  return (
    <div className="stack">
      <PageHeader
        title={
          {
            alerts: "Inventory Alerts",
            expiry: "Expiry Monitoring",
            "low-stock": "Low Stock",
            fefo: "FEFO Inspection",
          }[mode]
        }
        description="Advisory monitoring from the inventory ledger. Business date: UTC calendar date. Stock changes require an explicit inventory document."
      />
      <InventoryNavigation />
      <nav className="toolbar" aria-label="Inventory monitoring">
        <LinkButton to="/inventory/alerts">Alerts</LinkButton>
        <LinkButton to="/inventory/expiry">Expiry</LinkButton>
        <LinkButton to="/inventory/low-stock">Low stock</LinkButton>
        <LinkButton to="/inventory/fefo">FEFO</LinkButton>
        <LinkButton to="/inventory/documents">Inventory documents</LinkButton>
      </nav>
      <div className="toolbar">
        <Select
          label="Monitoring branch"
          value={branchId}
          onChange={(e) => select(e.target.value)}
        >
          {branches.map((b) => (
            <option key={b.id} value={b.id}>
              {b.name}
            </option>
          ))}
        </Select>
        <Button
          disabled={busy || branchLoading}
          onClick={() => change(() => setRefresh((r) => r + 1))}
        >
          Refresh Alerts
        </Button>
        <span>Business date: {date || "—"} (UTC)</span>
      </div>
      {error && (
        <Alert title="Monitoring error" tone="danger">
          {error}
        </Alert>
      )}
      {mode === "alerts" && (
        <Card>
          <div className="toolbar">
            <Select
              label="Alert type"
              value={type}
              onChange={(e) =>
                change(() => setType(e.target.value as AlertType | ""))
              }
            >
              <option value="">All types</option>
              {[
                "BATCH_EXPIRED",
                "BATCH_EXPIRING",
                "LOW_STOCK",
                "OUT_OF_STOCK",
              ].map((v) => (
                <option key={v}>{v}</option>
              ))}
            </Select>
            <Select
              label="Severity"
              value={severity}
              onChange={(e) =>
                change(() => setSeverity(e.target.value as Severity | ""))
              }
            >
              <option value="">All severities</option>
              {["CRITICAL", "HIGH", "MEDIUM"].map((v) => (
                <option key={v}>{v}</option>
              ))}
            </Select>
            <Select
              label="Alert status"
              value={status}
              onChange={(e) =>
                change(() => setStatus(e.target.value as AlertStatus | ""))
              }
            >
              <option value="">All statuses</option>
              {["ACTIVE", "RESOLVED", "DISMISSED"].map((v) => (
                <option key={v}>{v}</option>
              ))}
            </Select>
            <Input
              label="Updated from"
              type="date"
              value={from}
              onChange={(e) => change(() => setFrom(e.target.value))}
            />
            <Input
              label="Updated through"
              type="date"
              value={to}
              onChange={(e) => change(() => setTo(e.target.value))}
            />
          </div>
        </Card>
      )}
      {mode === "expiry" && (
        <Select
          label="Expiry window"
          value={window}
          onChange={(e) => change(() => setWindow(e.target.value))}
        >
          <option value="expired">Expired</option>
          <option value="30">0–30 days</option>
          <option value="60">31–60 days</option>
          <option value="90">61–90 days</option>
        </Select>
      )}
      {mode === "low-stock" && (
        <>
          <p>
            Only explicitly configured inventory levels participate. Zero stock
            appears once as OUT_OF_STOCK.
          </p>
          <Select
            label="Stock condition"
            value={out ? "out" : "all"}
            onChange={(e) => change(() => setOut(e.target.value === "out"))}
          >
            <option value="all">At or below reorder level</option>
            <option value="out">Out of stock only</option>
          </Select>
        </>
      )}
      {mode === "fefo" && (
        <Card>
          <Input
            label="Search Catalog for FEFO"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
          <Select
            label="FEFO product"
            value={productId}
            onChange={(e) =>
              change(() => {
                setProductId(e.target.value);
                setPackages([]);
                setPackageId("");
              })
            }
          >
            <option value="">Choose product</option>
            {productId && !products.some((p) => p.id === productId) && (
              <option value={productId}>Selected Catalog product</option>
            )}
            {products.map((p) => (
              <option key={p.id} value={p.id}>
                {p.nameEn || p.nameAr || p.id}
              </option>
            ))}
          </Select>
          <Select
            label="FEFO package"
            value={packageId}
            onChange={(e) => change(() => setPackageId(e.target.value))}
          >
            <option value="">Choose package</option>
            {packages.map((p) => (
              <option key={p.package.id} value={p.package.id}>
                {p.package.packageLabel}
              </option>
            ))}
          </Select>
          <p>
            Known non-expired expiry first; receipt, creation time and ID break
            ties. Unknown expiry last. Inspection does not reserve stock.
          </p>
        </Card>
      )}
      <Card>
        {pending ? (
          <p role="status">Loading monitoring…</p>
        ) : error ? (
          <p>Refresh to retry.</p>
        ) : mode === "alerts" ? (
          <Table
            label="Inventory alerts"
            rows={rows}
            rowKey={(r) => r.id}
            columns={[
              {
                key: "product",
                header: "Product / package",
                render: (r) =>
                  `${r.productName || "Unnamed product"} · ${r.packageLabel}`,
              },
              { key: "type", header: "Type", render: (r) => r.alertType },
              {
                key: "severity",
                header: "Severity",
                render: (r) => r.severity,
              },
              { key: "status", header: "Status", render: (r) => r.status },
              {
                key: "last",
                header: "Last detected (UTC)",
                render: (r) => r.lastDetectedAt,
              },
              {
                key: "action",
                header: "Actions",
                render: (r) => (
                  <>
                    <Button
                      disabled={busy}
                      onClick={() => {
                        setBusy(true);
                        getAlert(branchId, r.id)
                          .then(setDetail)
                          .catch((e) => setError(message(e)))
                          .finally(() => setBusy(false));
                      }}
                    >
                      History
                    </Button>
                    {r.status === "ACTIVE" && (
                      <Button
                        disabled={busy}
                        onClick={() => act(() => dismissAlert(branchId, r.id))}
                      >
                        Dismiss
                      </Button>
                    )}
                    {r.alertType === "BATCH_EXPIRED" && (
                      <LinkButton
                        to={inventoryLink(
                          "documents",
                          r.productPackageId,
                          r.batchId || undefined,
                          r.productId,
                          { new: "WRITE_OFF", action: "EXPIRY" },
                        )}
                      >
                        Open expiry write-off workflow
                      </LinkButton>
                    )}
                  </>
                ),
              },
            ]}
          />
        ) : mode === "fefo" ? (
          <Table
            label="FEFO batches"
            rows={fefo}
            rowKey={(r) => r.batch.id}
            columns={[
              {
                key: "lot",
                header: "Lot / batch",
                render: (r) =>
                  `${r.batch.lotNumber || "No lot"} · ${r.batch.id}`,
              },
              {
                key: "expiry",
                header: "Expiry",
                render: (r) => r.batch.expiryDate || "Unknown (last)",
              },
              { key: "qty", header: "Available", render: (r) => r.quantity },
              {
                key: "received",
                header: "Received",
                render: (r) => r.batch.receivedAt || "Unknown",
              },
              {
                key: "cost",
                header: "Cost",
                render: (r) => money(r.batch.costPriceMinor),
              },
            ]}
          />
        ) : (
          <Table
            label={
              mode === "expiry" ? "Expiry batches" : "Configured low stock"
            }
            rows={monitor}
            rowKey={(r) => r.batchId || r.productPackageId}
            columns={[
              {
                key: "links",
                header: "Inspect / workflow",
                render: (r: Monitor) => (
                  <div className="row wrap">
                    <LinkButton
                      to={inventoryLink(
                        "batches",
                        r.productPackageId,
                        r.batchId || undefined,
                        r.productId,
                      )}
                    >
                      Batches
                    </LinkButton>
                    <LinkButton
                      to={inventoryLink(
                        "movements",
                        r.productPackageId,
                        r.batchId || undefined,
                      )}
                    >
                      Movements
                    </LinkButton>
                    {mode === "expiry" ? (
                      <LinkButton
                        to={inventoryLink(
                          "documents",
                          r.productPackageId,
                          r.batchId || undefined,
                          r.productId,
                          {
                            new: "WRITE_OFF",
                            action: window === "expired" ? "EXPIRY" : "DAMAGE",
                          },
                        )}
                      >
                        Write-off workflow
                      </LinkButton>
                    ) : (
                      <>
                        <LinkButton
                          to={inventoryLink(
                            "levels",
                            r.productPackageId,
                            undefined,
                            r.productId,
                          )}
                        >
                          Reorder
                        </LinkButton>
                        <LinkButton
                          to={inventoryLink(
                            "documents",
                            r.productPackageId,
                            undefined,
                            r.productId,
                            { new: "RECEIPT" },
                          )}
                        >
                          Receive
                        </LinkButton>
                      </>
                    )}
                  </div>
                ),
              },

              {
                key: "product",
                header: "Product / package",
                render: (r) =>
                  `${r.productName || "Unnamed product"} · ${r.packageLabel}`,
              },
              ...(mode === "expiry"
                ? [
                    {
                      key: "lot",
                      header: "Lot / batch",
                      render: (r: Monitor) =>
                        `${r.lotNumber || "No lot"} · ${r.batchId}`,
                    },
                    {
                      key: "expiry",
                      header: "Expiry",
                      render: (r: Monitor) => r.expiryDate,
                    },
                    {
                      key: "days",
                      header: "Days remaining",
                      render: (r: Monitor) => r.daysRemaining,
                    },
                    {
                      key: "cost",
                      header: "Cost",
                      render: (r: Monitor) => money(r.costPriceMinor),
                    },
                  ]
                : [
                    {
                      key: "branch",
                      header: "Branch",
                      render: () => branchName,
                    },
                    {
                      key: "level",
                      header: "Reorder / min / max",
                      render: (r: Monitor) =>
                        `${r.reorderLevel} / ${r.minimumStock ?? "—"} / ${r.maximumStock ?? "—"}`,
                    },
                  ]),
              {
                key: "qty",
                header: "Current stock",
                render: (r) => r.quantity,
              },
              {
                key: "status",
                header: "Alert status",
                render: (r) =>
                  r.status || "Not active in configured alert window",
              },
              {
                key: "severity",
                header: "Severity",
                render: (r) => r.severity || "—",
              },
            ]}
          />
        )}
        {!pending && !error && total === 0 && (
          <p>No matching inventory records.</p>
        )}
        <div className="grid-footer">
          <Button
            disabled={pending || page === 0}
            onClick={() => {
              setPending(true);
              setPage((p) => p - 1);
            }}
          >
            Previous
          </Button>
          <span>
            Page {page + 1}
            {mode !== "fefo" && ` · ${total} records`}
          </span>
          <Button
            disabled={pending || (page + 1) * 20 >= total}
            onClick={() => {
              setPending(true);
              setPage((p) => p + 1);
            }}
          >
            Next
          </Button>
        </div>
      </Card>
      {detail && (
        <Card>
          <h2>Alert history</h2>
          <p>
            {detail.alert.productName} · {detail.alert.alertType} · First
            detected {detail.alert.firstDetectedAt}
          </p>
          <p>Details: {JSON.stringify(detail.alert.details)}</p>
          <ol>
            {detail.history.map((h, i) => (
              <li key={i}>
                {h.occurredAt} · {h.status} · {h.severity} ·{" "}
                {JSON.stringify(h.details)}
              </li>
            ))}
          </ol>
          <Button onClick={() => setDetail(null)}>Close history</Button>
        </Card>
      )}
      {mode === "alerts" && settings && (
        <Card>
          <h2>Branch alert settings</h2>
          <p>
            Disabling a category resolves its current alerts at the next
            evaluation. Re-enabling reactivates any remaining conditions.
          </p>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              void act(() => saveAlertSettings(settings));
            }}
          >
            <Input
              label="Warning days"
              type="number"
              min={0}
              max={36500}
              required
              value={settings.expiryWarningDays}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  expiryWarningDays: Number(e.target.value),
                })
              }
            />
            <Input
              label="Critical days"
              type="number"
              min={0}
              max={settings.expiryWarningDays}
              required
              value={settings.criticalExpiryDays}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  criticalExpiryDays: Number(e.target.value),
                })
              }
            />
            <label>
              <input
                type="checkbox"
                checked={settings.expiryEnabled}
                onChange={(e) =>
                  setSettings({ ...settings, expiryEnabled: e.target.checked })
                }
              />{" "}
              Expiry alerts enabled
            </label>
            <label>
              <input
                type="checkbox"
                checked={settings.lowStockEnabled}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    lowStockEnabled: e.target.checked,
                  })
                }
              />{" "}
              Stock alerts enabled
            </label>
            <Button type="submit" disabled={busy}>
              Save alert settings
            </Button>
          </form>
        </Card>
      )}
    </div>
  );
}
