import { useEffect, useRef, useState } from "react";
import { useSearchParams } from "react-router-dom";
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
  Modal,
} from "../../design-system";
import { isDesktopRuntime } from "../../lib/tauri/client";
import {
  listProducts,
  getProductDetail,
  listManufacturers,
  listCategories,
  listRoutes,
  toMinor,
} from "../../lib/tauri/catalog";
import type {
  ProductListItem,
  PackageDetail,
} from "../../lib/tauri/catalog.types";
import {
  overview,
  batches,
  movements,
  opening,
  saveLevel,
  type InventoryRow,
  type BatchRow,
  type MovementRow,
} from "../../lib/tauri/inventory-inspection";
export function InventoryNavigation() {
  return (
    <nav className="row wrap" aria-label="Inventory workflows">
      {[
        ["", "Overview"],
        ["movements", "Movements"],
        ["documents", "Documents"],
        ["count-sessions", "Stock Counts"],
        ["expiry", "Expiry"],
        ["alerts", "Alerts"],
        ["low-stock", "Low Stock"],
        ["fefo", "FEFO"],
        ["opening", "Opening Stock"],
      ].map(([path, label]) => (
        <LinkButton key={path} to={`/inventory${path ? "/" + path : ""}`}>
          {label}
        </LinkButton>
      ))}
    </nav>
  );
}
export function inventoryLink(
  path: string,
  packageId: string,
  batchId?: string,
  productId?: string,
  extra?: Record<string, string>,
) {
  const q = new URLSearchParams({
    package: packageId,
    ...(batchId ? { batch: batchId } : {}),
    ...(productId ? { product: productId } : {}),
    ...extra,
  });
  return `/inventory/${path}?${q}`;
}
export function InventoryWorkspace({
  mode = "overview",
}: {
  mode?: "overview" | "batches" | "movements" | "opening" | "levels";
}) {
  const b = useBranch();
  const [q] = useSearchParams();
  return (
    <Workspace
      key={`${b.branchId}:${mode}:${q.get("package")}:${q.get("batch")}`}
      mode={mode}
    />
  );
}
const message = (e: unknown) =>
  e instanceof Error ? e.message : "Inventory request failed.";
function Workspace({ mode }: { mode: string }) {
  const { branchId, branchName, branches, select, loading } = useBranch();
  const [url] = useSearchParams();
  const [packageId, setPackageId] = useState(url.get("package"));
  const [batchId, setBatchId] = useState(url.get("batch"));
  const [search, setSearch] = useState("");
  const [state, setState] = useState("");
  const [manufacturer, setManufacturer] = useState("");
  const [category, setCategory] = useState("");
  const [route, setRoute] = useState("");
  const [kind, setKind] = useState("");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [reference, setReference] = useState("");
  const [lookups, setLookups] = useState<{ id: string; name: string }[][]>([
    [],
    [],
    [],
  ]);
  const [page, setPage] = useState(0);
  const [attempt, setAttempt] = useState(0);
  const [error, setError] = useState("");
  const [data, setData] = useState<{
    key: string;
    items: InventoryRow[] | BatchRow[] | MovementRow[];
    total: number;
  } | null>(null);
  const key = JSON.stringify([
    branchId,
    mode,
    packageId,
    batchId,
    search,
    state,
    manufacturer,
    category,
    route,
    kind,
    from,
    to,
    reference,
    page,
    attempt,
  ]);
  useEffect(() => {
    if (!isDesktopRuntime() || mode !== "overview") return;
    let active = true;
    Promise.all([listManufacturers(), listCategories(), listRoutes()])
      .then((r) => {
        if (active) setLookups(r);
      })
      .catch((e) => {
        if (active) setError(message(e));
      });
    return () => {
      active = false;
    };
  }, [mode]);
  useEffect(() => {
    if (
      !isDesktopRuntime() ||
      loading ||
      !branchId ||
      mode === "opening" ||
      mode === "levels"
    )
      return;
    let active = true;
    const timer = setTimeout(() => {
      const q = {
        branchId,
        productPackageId: packageId,
        batchId,
        search,
        stockState: state || null,
        manufacturerId: manufacturer || null,
        categoryId: category || null,
        routeId: route || null,
        movementType: kind || null,
        dateFrom: from || null,
        dateTo: to || null,
        documentReference: reference,
        limit: 20,
        offset: page * 20,
      };
      const request =
        mode === "batches"
          ? batches(q)
          : mode === "movements"
            ? movements(q)
            : overview(q);
      request
        .then((r) => {
          if (active) {
            setData({ key, items: r.items, total: r.total });
            setError("");
          }
        })
        .catch((e) => {
          if (active) {
            setData({ key, items: [], total: 0 });
            setError(message(e));
          }
        });
    }, 150);
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [
    key,
    branchId,
    loading,
    mode,
    packageId,
    batchId,
    search,
    state,
    manufacturer,
    category,
    route,
    kind,
    from,
    to,
    reference,
    page,
  ]);
  const current = data?.key === key ? data : null;
  const change = (f: () => void) => {
    f();
    setPage(0);
  };
  if (!isDesktopRuntime())
    return (
      <PageHeader
        title="Inventory requires the desktop app"
        description="Local SQLite inventory is unavailable in browser preview."
      />
    );
  return (
    <div className="stack">
      <PageHeader
        title={
          {
            overview: "Inventory Overview",
            batches: "Batch / Lot Inspection",
            movements: "Stock Movements",
            opening: "Opening Stock",
            levels: "Reorder Levels",
          }[mode] || "Inventory"
        }
        description="Canonical SQLite inventory. Quantities are derived from the immutable stock ledger."
      />
      <InventoryNavigation />
      <Select
        label="Inventory branch"
        showLabel
        value={branchId}
        onChange={(e) => select(e.target.value)}
      >
        {branches.map((b) => (
          <option key={b.id} value={b.id}>
            {b.name}
          </option>
        ))}
      </Select>
      {mode === "opening" || mode === "levels" ? (
        <InventoryEntry
          key={branchId}
          levels={mode === "levels"}
          initialPackage={packageId || ""}
          initialProduct={url.get("product") || ""}
        />
      ) : (
        <>
          {mode === "overview" && (
            <>
              <div className="row wrap">
                <Input
                  label="Search inventory"
                  value={search}
                  onChange={(e) => change(() => setSearch(e.target.value))}
                />
                <Select
                  label="Stock state"
                  showLabel
                  value={state}
                  onChange={(e) => change(() => setState(e.target.value))}
                >
                  <option value="">All stock states</option>
                  {["IN_STOCK", "LOW_STOCK", "OUT_OF_STOCK"].map((s) => (
                    <option key={s}>{s}</option>
                  ))}
                </Select>
                {[
                  ["Manufacturer", manufacturer, setManufacturer],
                  ["Category", category, setCategory],
                  ["Route", route, setRoute],
                ].map(([label, value, set], i) => (
                  <Select
                    key={label as string}
                    label={label as string}
                    showLabel
                    value={value as string}
                    onChange={(e) =>
                      change(() => (set as (s: string) => void)(e.target.value))
                    }
                  >
                    <option value="">All {label as string}</option>
                    {lookups[i].map((v) => (
                      <option key={v.id} value={v.id}>
                        {v.name}
                      </option>
                    ))}
                  </Select>
                ))}
              </div>
              <div className="row wrap">
                {[
                  ["RECEIPT", "Receive Stock"],
                  ["ADJUSTMENT", "Adjust Stock"],
                  ["WRITE_OFF", "Damage / Expiry Write-Off"],
                  ["TRANSFER", "Transfer Stock"],
                ].map(([kind, label]) => (
                  <LinkButton
                    key={kind}
                    to={`/inventory/documents?new=${kind}`}
                  >
                    {label}
                  </LinkButton>
                ))}
                <LinkButton to="/inventory/levels">
                  Configure reorder level
                </LinkButton>
              </div>
            </>
          )}
          {mode === "movements" && (
            <div className="row wrap">
              <Input
                label="Package ID filter"
                value={packageId || ""}
                onChange={(e) =>
                  change(() => setPackageId(e.target.value || null))
                }
              />
              <Input
                label="Batch ID filter"
                value={batchId || ""}
                onChange={(e) =>
                  change(() => setBatchId(e.target.value || null))
                }
              />
              <Select
                label="Movement type"
                showLabel
                value={kind}
                onChange={(e) => change(() => setKind(e.target.value))}
              >
                <option value="">All movements</option>
                {[
                  "OPENING",
                  "PURCHASE_RECEIPT",
                  "ADJUSTMENT_IN",
                  "ADJUSTMENT_OUT",
                  "DAMAGE",
                  "EXPIRY",
                  "TRANSFER_OUT",
                  "TRANSFER_IN",
                  "STOCK_COUNT_CORRECTION",
                ].map((k) => (
                  <option key={k}>{k}</option>
                ))}
              </Select>
              <Input
                label="Movement from"
                type="date"
                value={from}
                onChange={(e) => change(() => setFrom(e.target.value))}
              />
              <Input
                label="Movement through"
                type="date"
                value={to}
                onChange={(e) => change(() => setTo(e.target.value))}
              />
              <Input
                label="Document or count reference"
                value={reference}
                onChange={(e) => change(() => setReference(e.target.value))}
              />
              {packageId && (
                <LinkButton to="/inventory/movements">
                  Clear package/batch filter
                </LinkButton>
              )}
            </div>
          )}
          {error && (
            <Alert title="Inventory could not be loaded" tone="danger">
              {error}
            </Alert>
          )}
          <Card>
            {!current ? (
              <p role="status">Loading inventory…</p>
            ) : error ? (
              <p>Retry to load current data.</p>
            ) : mode === "overview" ? (
              <Table
                label="Canonical inventory overview"
                rows={current.items as InventoryRow[]}
                rowKey={(r) => r.productPackageId}
                columns={[
                  {
                    key: "product",
                    header: "Product / package",
                    render: (r) => (
                      <LinkButton
                        to={inventoryLink(
                          "batches",
                          r.productPackageId,
                          undefined,
                          r.productId,
                        )}
                      >
                        {r.productName} · {r.packageLabel}
                      </LinkButton>
                    ),
                  },
                  { key: "branch", header: "Branch", render: () => branchName },
                  {
                    key: "quantity",
                    header: "Current stock",
                    render: (r) => r.quantity,
                  },
                  {
                    key: "batches",
                    header: "Batches",
                    render: (r) => r.batchCount,
                  },
                  {
                    key: "expiry",
                    header: "Nearest expiry",
                    render: (r) => r.nearestExpiry || "Unknown / none",
                  },
                  {
                    key: "reorder",
                    header: "Reorder",
                    render: (r) => r.reorderLevel ?? "Not configured",
                  },
                  {
                    key: "state",
                    header: "Stock state",
                    render: (r) => (
                      <>
                        {r.stockState}
                        {(!r.packageActive || !r.productActive) &&
                          " · Inactive Catalog record"}
                      </>
                    ),
                  },
                  {
                    key: "actions",
                    header: "Inspect",
                    render: (r) => (
                      <div className="row wrap">
                        <LinkButton
                          to={inventoryLink("movements", r.productPackageId)}
                        >
                          Movements
                        </LinkButton>
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
                            "fefo",
                            r.productPackageId,
                            undefined,
                            r.productId,
                          )}
                        >
                          FEFO
                        </LinkButton>
                      </div>
                    ),
                  },
                ]}
              />
            ) : mode === "batches" ? (
              <Table
                label="Canonical inventory batches"
                rows={current.items as BatchRow[]}
                rowKey={(r) => r.id}
                columns={[
                  {
                    key: "product",
                    header: "Product / package",
                    render: (r) => `${r.productName} · ${r.packageLabel}`,
                  },
                  {
                    key: "batch",
                    header: "Lot / batch",
                    render: (r) => (
                      <>
                        {r.lotNumber || "Unspecified"}
                        <small className="muted"> {r.id}</small>
                      </>
                    ),
                  },
                  { key: "branch", header: "Branch", render: () => branchName },
                  {
                    key: "qty",
                    header: "Available quantity",
                    render: (r) => r.quantity,
                  },
                  {
                    key: "expiry",
                    header: "Expiry",
                    render: (r) => r.expiryDate || "Unknown",
                  },
                  {
                    key: "received",
                    header: "Received",
                    render: (r) => r.receivedAt || "Unspecified",
                  },
                  {
                    key: "cost",
                    header: "Historical cost",
                    render: (r) =>
                      r.costPriceMinor === null
                        ? "Unknown"
                        : `${(r.costPriceMinor / 100).toFixed(2)} EGP`,
                  },
                  {
                    key: "active",
                    header: "Active",
                    render: (r) => (r.isActive ? "Yes" : "No"),
                  },
                  {
                    key: "actions",
                    header: "Actions",
                    render: (r) => (
                      <div className="row wrap">
                        <LinkButton
                          to={inventoryLink(
                            "movements",
                            r.productPackageId,
                            r.id,
                          )}
                        >
                          Movements
                        </LinkButton>
                        <LinkButton
                          to={inventoryLink(
                            "documents",
                            r.productPackageId,
                            r.id,
                            r.productId,
                            { new: "WRITE_OFF" },
                          )}
                        >
                          Write-off
                        </LinkButton>
                        <LinkButton
                          to={inventoryLink(
                            "fefo",
                            r.productPackageId,
                            undefined,
                            r.productId,
                          )}
                        >
                          FEFO
                        </LinkButton>
                      </div>
                    ),
                  },
                ]}
              />
            ) : (
              <Table
                label="Canonical stock movements"
                rows={current.items as MovementRow[]}
                rowKey={(r) => r.id}
                columns={[
                  {
                    key: "time",
                    header: "Timestamp (UTC)",
                    render: (r) => r.createdAt,
                  },
                  { key: "branch", header: "Branch", render: () => branchName },
                  {
                    key: "product",
                    header: "Product / package",
                    render: (r) => `${r.productName} · ${r.packageLabel}`,
                  },
                  {
                    key: "batch",
                    header: "Batch",
                    render: (r) => (
                      <LinkButton
                        to={inventoryLink(
                          "batches",
                          r.productPackageId,
                          r.batchId,
                        )}
                      >
                        {r.batchId}
                      </LinkButton>
                    ),
                  },
                  {
                    key: "type",
                    header: "Type",
                    render: (r) => r.movementType,
                  },
                  {
                    key: "qty",
                    header: "Signed quantity",
                    render: (r) =>
                      r.quantityDelta > 0
                        ? `+${r.quantityDelta}`
                        : r.quantityDelta,
                  },
                  {
                    key: "reference",
                    header: "Origin",
                    render: (r) =>
                      r.documentId ? (
                        <LinkButton to={`/inventory/documents/${r.documentId}`}>
                          {r.documentNumber}
                        </LinkButton>
                      ) : r.countSessionId ? (
                        <LinkButton
                          to={`/inventory/count-sessions/${r.countSessionId}`}
                        >
                          {r.countNumber}
                        </LinkButton>
                      ) : (
                        r.referenceId || "Opening / direct canonical operation"
                      ),
                  },
                  {
                    key: "reason",
                    header: "Reason / actor",
                    render: (r) =>
                      `${r.reason} · ${r.createdBy || "Not recorded"}`,
                  },
                ]}
              />
            )}
            {current && !error && current.total === 0 && (
              <p>
                No matching inventory records. Packages appear after a batch,
                level, movement or document introduces them to this branch.
              </p>
            )}
            <div className="grid-footer">
              <Button
                disabled={!current || page === 0}
                onClick={() => setPage((p) => p - 1)}
              >
                Previous inventory
              </Button>
              <span>
                Page {page + 1} · {current?.total ?? "…"} records
              </span>
              <Button
                disabled={!current || (page + 1) * 20 >= current.total}
                onClick={() => setPage((p) => p + 1)}
              >
                Next inventory
              </Button>
              <Button onClick={() => setAttempt((a) => a + 1)}>
                Refresh inventory
              </Button>
            </div>
          </Card>
        </>
      )}
    </div>
  );
}
function InventoryEntry({
  levels,
  initialPackage,
  initialProduct,
}: {
  levels: boolean;
  initialPackage: string;
  initialProduct: string;
}) {
  const { branchId } = useBranch();
  const [search, setSearch] = useState("");
  const [products, setProducts] = useState<ProductListItem[]>([]);
  const [product, setProduct] = useState(initialProduct);
  const [packages, setPackages] = useState<PackageDetail[]>([]);
  const [pkg, setPkg] = useState(initialPackage);
  const [quantity, setQuantity] = useState("");
  const [minimum, setMinimum] = useState("");
  const [maximum, setMaximum] = useState("");
  const [lot, setLot] = useState("");
  const [expiry, setExpiry] = useState("");
  const [cost, setCost] = useState("");
  const [reason, setReason] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [done, setDone] = useState("");
  const [confirm, setConfirm] = useState(false);
  const lock = useRef(false);
  const request = useRef(crypto.randomUUID());
  useEffect(() => {
    let active = true;
    const t = setTimeout(
      () =>
        listProducts({ search, limit: 20, offset: 0, includeInactive: false })
          .then((p) => {
            if (active) setProducts(p.items);
          })
          .catch((e) => {
            if (active) setError(message(e));
          }),
      200,
    );
    return () => {
      active = false;
      clearTimeout(t);
    };
  }, [search]);
  useEffect(() => {
    if (!product) return;
    let active = true;
    getProductDetail(product)
      .then((d) => {
        if (active) setPackages(d.packages);
      })
      .catch((e) => {
        if (active) setError(message(e));
      });
    return () => {
      active = false;
    };
  }, [product]);
  useEffect(() => {
    if (!levels || !pkg) return;
    let active = true;
    overview({ branchId, productPackageId: pkg, limit: 1, offset: 0 })
      .then((p) => {
        if (active) {
          setQuantity(p.items[0]?.reorderLevel?.toString() || "");
          setMinimum(p.items[0]?.minimumStock?.toString() || "");
          setMaximum(p.items[0]?.maximumStock?.toString() || "");
        }
      })
      .catch((e) => {
        if (active) setError(message(e));
      });
    return () => {
      active = false;
    };
  }, [levels, pkg, branchId]);
  async function submit() {
    if (lock.current) return;
    lock.current = true;
    setBusy(true);
    setError("");
    setConfirm(false);
    try {
      if (!pkg) throw new Error("Choose a package.");
      if (levels) {
        await saveLevel({
          branchId,
          productPackageId: pkg,
          reorderLevel: Number(quantity),
          minimumStock: minimum === "" ? null : Number(minimum),
          maximumStock: maximum === "" ? null : Number(maximum),
        });
        setDone("Reorder level saved.");
      } else {
        const minor = cost.trim() ? toMinor(cost) : null;
        if (cost.trim() && minor === null)
          throw new Error("Enter a valid cost.");
        const m = await opening({
          requestId: request.current,
          batch: {
            branchId,
            productPackageId: pkg,
            lotNumber: lot || null,
            expiryDate: expiry || null,
            costPriceMinor: minor,
          },
          quantity: Number(quantity),
          reason,
          createdBy: null,
        });
        setDone(`Opening stock recorded: ${m.id}`);
      }
    } catch (e) {
      setError(message(e));
    } finally {
      lock.current = false;
      setBusy(false);
    }
  }
  return (
    <Card>
      <div className="stack">
        <p>
          {levels
            ? "Only saved, explicit reorder levels affect low-stock classification. Blank minimum/maximum remain unset."
            : "Opening Stock creates a new canonical batch and opening movement. Use an Adjustment document for corrections to existing stock."}
        </p>
        {error && (
          <Alert title="Inventory action failed" tone="danger">
            {error}
          </Alert>
        )}
        {done && <Alert title={done} tone="success" />}
        <form
          onSubmit={(e) => {
            e.preventDefault();
            if (levels) void submit();
            else setConfirm(true);
          }}
        >
          <fieldset disabled={busy || (!levels && !!done)}>
            <div className="row wrap">
              <Input
                label="Search Catalog package"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
              />
              <Select
                label="Inventory product"
                showLabel
                value={product}
                onChange={(e) => {
                  setProduct(e.target.value);
                  setPackages([]);
                  setPkg("");
                  setQuantity("");
                  setDone("");
                }}
              >
                <option value="">Choose product</option>
                {product && !products.some((p) => p.id === product) && (
                  <option value={product}>Selected Catalog product</option>
                )}
                {products.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.nameEn || p.nameAr}
                  </option>
                ))}
              </Select>
              <Select
                label="Inventory package"
                showLabel
                value={pkg}
                onChange={(e) => {
                  setPkg(e.target.value);
                  setQuantity("");
                  setMinimum("");
                  setMaximum("");
                  setDone("");
                }}
              >
                <option value="">Choose package</option>
                {packages.map((p) => (
                  <option key={p.package.id} value={p.package.id}>
                    {p.package.packageLabel}
                  </option>
                ))}
              </Select>
              <Input
                label={levels ? "Reorder level" : "Opening quantity"}
                required
                type="number"
                min={levels ? 0 : 1}
                step={1}
                value={quantity}
                onChange={(e) => setQuantity(e.target.value)}
              />
              {levels ? (
                <>
                  <Input
                    label="Minimum stock"
                    type="number"
                    min={0}
                    value={minimum}
                    onChange={(e) => setMinimum(e.target.value)}
                  />
                  <Input
                    label="Maximum stock"
                    type="number"
                    min={0}
                    value={maximum}
                    onChange={(e) => setMaximum(e.target.value)}
                  />
                </>
              ) : (
                <>
                  <Input
                    label="Opening lot"
                    value={lot}
                    onChange={(e) => setLot(e.target.value)}
                  />
                  <Input
                    label="Opening expiry"
                    type="date"
                    value={expiry}
                    onChange={(e) => setExpiry(e.target.value)}
                  />
                  <Input
                    label="Opening cost (EGP)"
                    value={cost}
                    onChange={(e) => setCost(e.target.value)}
                  />
                  <Input
                    label="Opening reason"
                    required
                    value={reason}
                    onChange={(e) => setReason(e.target.value)}
                  />
                </>
              )}
            </div>
            <Button type="submit" disabled={!pkg || quantity === ""}>
              {levels ? "Save reorder level" : "Review opening stock"}
            </Button>
          </fieldset>
        </form>
        {pkg && (
          <LinkButton to={inventoryLink("movements", pkg)}>
            Inspect package movements
          </LinkButton>
        )}
        <Modal
          open={confirm}
          onOpenChange={setConfirm}
          title="Record opening stock?"
          description="This creates a permanent opening movement and a new batch. It does not edit existing stock."
        >
          <p>
            Quantity: {quantity} · Reason: {reason}
          </p>
          <Button onClick={() => setConfirm(false)}>Back</Button>
          <Button disabled={busy} onClick={() => void submit()}>
            Confirm opening stock
          </Button>
        </Modal>
      </div>
    </Card>
  );
}
