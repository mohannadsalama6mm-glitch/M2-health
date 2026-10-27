import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Pencil, ArrowLeftRight, ClipboardCheck, PackagePlus } from "lucide-react";
import {
  PageHeader,
  LinkButton,
  RouteTabs,
  PriceDisplay,
  Status,
  Button,
  SummaryStrip,
  Card,
  TableToolbar,
  SearchInput,
  Select,
  Table,
  Pagination,
  Alert,
  EmptyState,
  Modal,
  Input,
  Textarea,
  type Column,
} from "../../design-system";
import { isDesktopRuntime } from "../../lib/tauri/client";
import { useBranch } from "../../app/BranchContext";
import { useAuth } from "../../app/AuthContext";
import { useFeedback } from "../../app/FeedbackContext";
import { ScopeNotice } from "../../app/ScopeNotice";
import { useLocalization } from "../../localization/LocalizationContext";
import {
  getStockOverview,
  getStockSummary,
  listStockMovements,
  adjustStock,
  transferStock,
  setReorderLevel,
  toMajor,
  toMinor,
  MOVEMENT_TYPES,
} from "../../lib/tauri/inventory";
import { opening } from "../../lib/tauri/inventory-inspection";
import {
  createProductFull,
  getProductDetail,
  listProducts,
} from "../../lib/tauri/catalog";
import type {
  PackageDetail,
  ProductListItem,
} from "../../lib/tauri/catalog.types";
import type {
  StockOverviewPage,
  StockRow,
  StockSummary,
  StockMovementPage,
  StockMovementItem,
} from "../../lib/tauri/inventory";
const PAGE_SIZE = 10;
export const inventoryTabs = [
  { label: "Stock overview", to: "/inventory" },
  { label: "Movements", to: "/inventory/movements" },
  { label: "Stock counts", to: "/inventory/counts" },
  { label: "Expiry management", to: "/inventory/expiry" },
];
const statusLabel = (status: string) =>
  ({
    in_stock: "In stock",
    low: "Low stock",
    out_of_stock: "Out of stock",
    expiring: "Expiring",
    inactive: "Inactive",
  })[status] ?? status;
export const movementTypeLabel = (type: string) =>
  ({
    opening: "Opening",
    purchase: "Purchase",
    sale: "Sale",
    customer_return: "Customer return",
    supplier_return: "Supplier return",
    adjustment: "Adjustment",
    damage: "Damage",
    expired: "Expired",
    transfer_out: "Transfer out",
    transfer_in: "Transfer in",
    count_correction: "Count correction",
  })[type] ?? type;
const productName = (row: {
  nameEn: string | null;
  nameAr: string | null;
  scientificName: string | null;
}) => row.nameEn ?? row.nameAr ?? row.scientificName ?? "Unnamed product";
function SortHeader({
  label,
  active,
  direction,
  onToggle,
}: {
  label: string;
  active: boolean;
  direction: "asc" | "desc";
  onToggle: () => void;
}) {
  return (
    <button
      type="button"
      className="text-link"
      title={
        active
          ? `Sorted by ${label.toLowerCase()} (${direction}). Click to reverse.`
          : `Sort by ${label.toLowerCase()}`
      }
      onClick={onToggle}
    >
      {label}
      <span aria-hidden="true">
        {active ? (direction === "asc" ? " ↑" : " ↓") : ""}
      </span>
    </button>
  );
}
export function Inventory() {
  const desktop = isDesktopRuntime();
  const auth = useAuth();
  const notify = useFeedback();
  const { t } = useLocalization();
  const { branchId, branchName } = useBranch();
  const [search, setSearch] = useState("");
  const [status, setStatus] = useState("");
  const [sort, setSort] = useState("name");
  const [sortDirection, setSortDirection] = useState<"asc" | "desc">("asc");
  const [page, setPage] = useState(1);
  const [data, setData] = useState<StockOverviewPage | null>(null);
  const [summary, setSummary] = useState<StockSummary | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);
  const [addStockOpen, setAddStockOpen] = useState(false);
  const [adjustOpen, setAdjustOpen] = useState(false);
  const [transferOpen, setTransferOpen] = useState(false);
  const [reorderRow, setReorderRow] = useState<StockRow | null>(null);
  useEffect(() => {
    if (!desktop) return;
    let active = true;
    const timer = window.setTimeout(
      () => {
        if (!active) return;
        setLoading(true);
        setError("");
        Promise.all([
          getStockOverview({
            branchId,
            search: search || null,
            status: status || null,
            sort,
            sortDirection,
            limit: PAGE_SIZE,
            offset: (page - 1) * PAGE_SIZE,
          }),
          getStockSummary(branchId),
        ])
          .then(([overview, total]) => {
            if (!active) return;
            setData(overview);
            setSummary(total);
            setLoading(false);
          })
          .catch((err: unknown) => {
            if (!active) return;
            setError(
              err instanceof Error
                ? err.message
                : "The inventory could not be loaded.",
            );
            setLoading(false);
          });
      },
      search ? 250 : 0,
    );
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [desktop, branchId, search, status, sort, sortDirection, page, attempt]);
  const toggleSort = (key: string) => {
    if (sort === key) setSortDirection((d) => (d === "asc" ? "desc" : "asc"));
    else {
      setSort(key);
      setSortDirection("asc");
    }
  };
  const pages = data ? Math.max(1, Math.ceil(data.total / data.limit)) : 1;
  const columns: Column<StockRow>[] = [
    {
      key: "name",
      header: (
        <SortHeader
          label="Product / package"
          active={sort === "name"}
          direction={sortDirection}
          onToggle={() => toggleSort("name")}
        />
      ),
      render: (r) => (
        <Link className="text-link" to={`/catalog/${r.productId}`}>
          {productName(r)}
          <span className="cell-secondary">{r.packageLabel}</span>
          {r.batches.length > 0 && (
            <span
              className="cell-secondary"
              title="Active batches on this branch: batch number × quantity"
            >
              {r.batches
                .slice(0, 2)
                .map((b) => `${b.batchNumber}×${b.quantity}`)
                .join(" · ")}
              {r.batches.length > 2 ? ` · +${r.batches.length - 2} more` : ""}
            </span>
          )}
        </Link>
      ),
    },
    {
      key: "quantity",
      header: (
        <SortHeader
          label="In stock"
          active={sort === "quantity"}
          direction={sortDirection}
          onToggle={() => toggleSort("quantity")}
        />
      ),
      render: (r) => r.quantity,
    },
    {
      key: "reorder",
      header: (
        <SortHeader
          label="Reorder"
          active={sort === "reorder"}
          direction={sortDirection}
          onToggle={() => toggleSort("reorder")}
        />
      ),
      render: (r) => (
        <div className="row">
          {r.reorderLevel > 0 ? r.reorderLevel : "—"}
          <Button
            size="sm"
            variant="ghost"
            aria-label={`Set reorder level for ${productName(r)}`}
            onClick={() => setReorderRow(r)}
          >
            <Pencil size={14} />
          </Button>
        </div>
      ),
    },
    {
      key: "expiry",
      header: (
        <SortHeader
          label="Earliest expiry"
          active={sort === "expiry"}
          direction={sortDirection}
          onToggle={() => toggleSort("expiry")}
        />
      ),
      render: (r) =>
        r.minExpiryDays != null ? `${r.minExpiryDays} days` : "—",
    },
    {
      key: "price",
      header: (
        <SortHeader
          label="Selling price"
          active={sort === "price"}
          direction={sortDirection}
          onToggle={() => toggleSort("price")}
        />
      ),
      render: (r) =>
        r.sellingPriceMinor != null ? (
          <PriceDisplay amount={toMajor(r.sellingPriceMinor)} />
        ) : (
          <span className="muted">—</span>
        ),
    },
    {
      key: "cost",
      header: "Cost",
      render: (r) =>
        r.costPriceMinor != null ? (
          <PriceDisplay amount={toMajor(r.costPriceMinor)} />
        ) : (
          <span className="muted">—</span>
        ),
    },
    {
      key: "status",
      header: (
        <SortHeader
          label="Status"
          active={sort === "status"}
          direction={sortDirection}
          onToggle={() => toggleSort("status")}
        />
      ),
      render: (r) => <Status value={statusLabel(r.status)} />,
    },
  ];
  const rows = data?.items ?? [];
  return (
    <div className="feature-page">
      <PageHeader
        title={t("page.inventoryTitle")}
        description={t("page.inventoryCopy")}
        actions={
          <>
            {auth.hasPermission("inventory.adjust") && (
              <Button
                variant="primary"
                disabled={!desktop || !branchId}
                onClick={() => setAddStockOpen(true)}
              >
                <PackagePlus size={16} />
                {t("inventory.addStock")}
              </Button>
            )}
            <LinkButton to="/inventory/counts">
              <ClipboardCheck size={16} />
              {t("page.stockCountsTitle")}
            </LinkButton>
          </>
        }
      />
      <ScopeNotice />
      <RouteTabs items={inventoryTabs} />
      {desktop && (
        <Card>
          <div className="stack">
            <div>
              <h2>{t("inventory.actionGroupTitle")}</h2>
              <p className="muted">{t("inventory.actionGroupCopy")}</p>
            </div>
            <div className="row wrap">
              <LinkButton to="/purchases/new">
                {t("inventory.purchaseStock")}
              </LinkButton>
              <LinkButton to="/inventory/documents?new=RECEIPT">
                {t("inventory.receiveSupplierStock")}
              </LinkButton>
              <Button
                variant="secondary"
                disabled={!rows.length}
                onClick={() => setAdjustOpen(true)}
              >
                {t("inventory.adjustStock")}
              </Button>
              <Button
                variant="secondary"
                disabled={!rows.length}
                onClick={() => setTransferOpen(true)}
              >
                <ArrowLeftRight size={16} />
                {t("inventory.transferStock")}
              </Button>
              <LinkButton to="/inventory/levels">
                {t("inventory.configureReorder")}
              </LinkButton>
            </div>
          </div>
        </Card>
      )}
      {!desktop && (
        <Alert title="Desktop app required" tone="warning">
          Inventory reads from the local database. Open M² Health as the desktop
          app to manage stock.
        </Alert>
      )}
      <SummaryStrip
        items={[
          { label: "In stock", value: summary ? summary.inStock : "—" },
          { label: "Low stock", value: summary ? summary.low : "—" },
          { label: "Out of stock", value: summary ? summary.outOfStock : "—" },
          {
            label: "Inventory value · cost",
            value: summary ? (
              <PriceDisplay amount={toMajor(summary.valueMinor)} />
            ) : (
              "—"
            ),
          },
        ]}
      />
      {desktop && error && (
        <Alert title="The inventory could not be loaded" tone="danger">
          {error}
          <Button size="sm" onClick={() => setAttempt((v) => v + 1)}>
            Retry
          </Button>
        </Alert>
      )}
      {!desktop ? (
        <Card>
          <EmptyState
            title="Inventory requires the desktop app"
            description="Stock and batch data are stored in the local SQLite database and can only be managed inside M² Health."
          />
        </Card>
      ) : (
        <Card>
          <TableToolbar>
            <SearchInput
              label="Search inventory"
              placeholder="Search products…"
              value={search}
              onChange={(e) => {
                setSearch(e.target.value);
                setPage(1);
              }}
            />
            <div className="row">
              <Select
                label="Status"
                value={status}
                onChange={(e) => {
                  setStatus(e.target.value);
                  setPage(1);
                }}
              >
                <option value="">All stock statuses</option>
                {Object.entries(statusLabel).map(([key, label]) => (
                  <option key={key} value={key}>
                    {label}
                  </option>
                ))}
              </Select>
            </div>
          </TableToolbar>
          {loading && !data ? (
            <div className="stack" role="status">
              <Alert title="Loading inventory…" tone="info" />
            </div>
          ) : data && rows.length === 0 ? (
            <EmptyState
              title={search || status ? t("empty.inventoryFilteredTitle") : t("empty.inventoryTitle")}
              description={search || status ? t("empty.inventoryFilteredCopy") : t("empty.inventoryCopy")}
              action={search || status
                ? <Button onClick={() => { setSearch(""); setStatus(""); setPage(1); }}>{t("common.clearFilters")}</Button>
                : <div className="row wrap">
                    <Button variant="primary" onClick={() => setAddStockOpen(true)}>
                      {t("empty.inventoryAction")}
                    </Button>
                    <LinkButton to="/catalog/new">{t("inventory.createProduct")}</LinkButton>
                  </div>}
            />
          ) : (
            <Table
              label="Inventory"
              rows={rows}
              columns={columns}
              rowKey={(r) => r.packageId}
            />
          )}
          <div className="grid-footer">
            <span className="muted">
              {data ? `${data.total} packages · branch-scoped` : "—"}
            </span>
            <Pagination page={page} pages={pages} onChange={setPage} />
          </div>
        </Card>
      )}
      <AddStockDialog
        key={addStockOpen ? `add-stock-${branchId}` : "add-stock-closed"}
        open={addStockOpen}
        onOpenChange={setAddStockOpen}
        branchId={branchId}
        branchName={branchName}
        canCreateProduct={auth.hasPermission("catalog.manage")}
        canAdjust={rows.length > 0}
        notify={notify}
        onAdjust={() => {
          setAddStockOpen(false);
          setAdjustOpen(true);
        }}
        onDone={() => {
          setAddStockOpen(false);
          setAttempt((value) => value + 1);
        }}
        onError={(message) => setError(message)}
      />
      <AdjustStockDialog
        key={adjustOpen ? "adjust-open" : "adjust-closed"}
        open={adjustOpen}
        onOpenChange={setAdjustOpen}
        rows={rows}
        branchId={branchId}
        notify={notify}
        onDone={() => {
          setAdjustOpen(false);
          setAttempt((v) => v + 1);
        }}
        onError={(message) => setError(message)}
      />
      <TransferStockDialog
        key={transferOpen ? "transfer-open" : "transfer-closed"}
        open={transferOpen}
        onOpenChange={setTransferOpen}
        rows={rows}
        branchId={branchId}
        notify={notify}
        onDone={() => {
          setTransferOpen(false);
          setAttempt((v) => v + 1);
        }}
        onError={(message) => setError(message)}
      />
      <ReorderDialog
        key={reorderRow ? `reorder-${reorderRow.packageId}` : "reorder-closed"}
        open={!!reorderRow}
        onOpenChange={() => setReorderRow(null)}
        row={reorderRow}
        branchId={branchId}
        notify={notify}
        onDone={() => {
          setReorderRow(null);
          setAttempt((v) => v + 1);
        }}
        onError={(message) => setError(message)}
      />
    </div>
  );
}
type StockEntrySource = "opening" | "receipt" | "adjustment";

export function AddStockDialog({
  open,
  onOpenChange,
  branchId,
  branchName,
  canCreateProduct,
  canAdjust,
  notify,
  onAdjust,
  onDone,
  onError,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  branchId: string;
  branchName: string;
  canCreateProduct: boolean;
  canAdjust: boolean;
  notify: (message: string) => void;
  onAdjust: () => void;
  onDone: () => void;
  onError: (message: string) => void;
}) {
  const { locale, t } = useLocalization();
  const [source, setSource] = useState<StockEntrySource>("opening");
  const [creating, setCreating] = useState(false);
  const [search, setSearch] = useState("");
  const [products, setProducts] = useState<ProductListItem[]>([]);
  const [productId, setProductId] = useState("");
  const [packages, setPackages] = useState<PackageDetail[]>([]);
  const [packageId, setPackageId] = useState("");
  const [productNameInput, setProductNameInput] = useState("");
  const [packageLabel, setPackageLabel] = useState("");
  const [quantity, setQuantity] = useState("");
  const [reason, setReason] = useState("");
  const [lotNumber, setLotNumber] = useState("");
  const [expiryDate, setExpiryDate] = useState("");
  const [cost, setCost] = useState("");
  const [loadingProducts, setLoadingProducts] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [dialogError, setDialogError] = useState("");

  useEffect(() => {
    if (!open || source !== "opening" || creating) return;
    let active = true;
    const timer = window.setTimeout(() => {
      setLoadingProducts(true);
      listProducts({
        search: search || null,
        limit: 50,
        offset: 0,
        includeInactive: false,
      })
        .then((page) => {
          if (!active) return;
          setProducts(page.items);
          setLoadingProducts(false);
        })
        .catch(() => {
          if (!active) return;
          setDialogError(t("inventory.loadFailed"));
          setLoadingProducts(false);
        });
    }, search ? 200 : 0);
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [creating, open, search, source, t]);

  useEffect(() => {
    if (!open || creating || !productId) return;
    let active = true;
    getProductDetail(productId)
      .then((detail) => {
        if (!active) return;
        const activePackages = detail.packages.filter(
          (entry) => entry.package.isActive,
        );
        setPackages(activePackages);
        const preferred =
          activePackages.find((entry) => entry.package.isDefault) ??
          activePackages[0];
        setPackageId(preferred?.package.id ?? "");
      })
      .catch(() => {
        if (active) setDialogError(t("inventory.loadFailed"));
      });
    return () => {
      active = false;
    };
  }, [creating, open, productId, t]);

  const selectedProductLabel = (product: ProductListItem) =>
    product.nameEn ?? product.nameAr ?? product.scientificName ?? product.id;

  async function submit() {
    setDialogError("");
    const parsedQuantity = Number(quantity);
    if (!Number.isInteger(parsedQuantity) || parsedQuantity <= 0) {
      setDialogError(t("inventory.quantityRequired"));
      return;
    }
    if (!reason.trim()) {
      setDialogError(t("inventory.reasonRequired"));
      return;
    }
    const costMinor = cost.trim() ? toMinor(cost) : null;
    if (cost.trim() && costMinor === null) {
      setDialogError(t("inventory.costInvalid"));
      return;
    }
    let targetPackageId = packageId;
    setSubmitting(true);
    try {
      if (creating) {
        if (!productNameInput.trim())
          throw new Error(t("inventory.nameRequired"));
        if (!packageLabel.trim())
          throw new Error(t("inventory.packageRequired"));
        const saved = await createProductFull({
          commercialNameEn:
            locale === "en" ? productNameInput.trim() : null,
          commercialNameAr:
            locale === "ar" ? productNameInput.trim() : null,
          packages: [
            {
              packageLabel: packageLabel.trim(),
              isDefault: true,
              isActive: true,
              costPriceMinor: costMinor,
            },
          ],
        });
        const detail = await getProductDetail(saved.id);
        targetPackageId =
          detail.packages.find((entry) => entry.package.isDefault)?.package.id ??
          detail.packages[0]?.package.id ??
          "";
        setCreating(false);
        setProductId(saved.id);
        setPackages(detail.packages);
        setPackageId(targetPackageId);
        notify(t("inventory.productCreated"));
      }
      if (!targetPackageId) throw new Error(t("inventory.productRequired"));
      await opening({
        requestId: crypto.randomUUID(),
        batch: {
          branchId,
          productPackageId: targetPackageId,
          lotNumber: lotNumber.trim() || null,
          expiryDate: expiryDate || null,
          costPriceMinor: costMinor,
        },
        quantity: parsedQuantity,
        reason: reason.trim(),
        createdBy: null,
      });
      notify(t("inventory.stockAdded", { branch: branchName }));
      onDone();
    } catch (error: unknown) {
      const message =
        error instanceof Error ? error.message : t("inventory.saveFailed");
      setDialogError(message);
      onError(message);
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <Modal
      open={open}
      onOpenChange={onOpenChange}
      title={t("inventory.addStockTitle")}
      description={t("inventory.addStockCopy")}
    >
      <div className="stack">
        <Alert title={t("inventory.activeBranch", { branch: branchName })} tone="info" />
        <Select
          label={t("inventory.stockSource")}
          value={source}
          onChange={(event) => {
            setSource(event.target.value as StockEntrySource);
            setDialogError("");
          }}
        >
          <option value="opening">{t("inventory.initialStock")}</option>
          <option value="receipt">{t("inventory.supplierReceipt")}</option>
          <option value="adjustment">{t("inventory.adjustment")}</option>
        </Select>

        {source === "receipt" ? (
          <div className="stack">
            <p>{t("inventory.supplierReceiptCopy")}</p>
            <LinkButton primary to="/inventory/documents?new=RECEIPT">
              {t("inventory.openReceipt")}
            </LinkButton>
          </div>
        ) : source === "adjustment" ? (
          <div className="stack">
            <p>{t("inventory.adjustmentCopy")}</p>
            <Button variant="primary" disabled={!canAdjust} onClick={onAdjust}>
              {t("inventory.openAdjustment")}
            </Button>
          </div>
        ) : (
          <form
            className="stack"
            onSubmit={(event) => {
              event.preventDefault();
              void submit();
            }}
          >
            <p>{t("inventory.initialStockCopy")}</p>
            {dialogError && (
              <Alert title={t("inventory.saveFailed")} tone="danger">
                {dialogError}
              </Alert>
            )}
            <div className="row wrap" role="group" aria-label={t("inventory.entryType")}>
              <Button
                type="button"
                variant={!creating ? "primary" : "secondary"}
                onClick={() => {
                  setCreating(false);
                  setPackages([]);
                  setPackageId("");
                }}
              >
                {t("inventory.existingProduct")}
              </Button>
              {canCreateProduct && (
                <Button
                  type="button"
                  variant={creating ? "primary" : "secondary"}
                  onClick={() => {
                    setCreating(true);
                    setProductId("");
                    setPackages([]);
                    setPackageId("");
                  }}
                >
                  {t("inventory.newProduct")}
                </Button>
              )}
            </div>
            {creating ? (
              <>
                <Input
                  label={t("inventory.productName")}
                  required
                  value={productNameInput}
                  onChange={(event) => setProductNameInput(event.target.value)}
                />
                <Input
                  label={t("inventory.packageLabel")}
                  required
                  placeholder={t("inventory.packageExample")}
                  value={packageLabel}
                  onChange={(event) => setPackageLabel(event.target.value)}
                />
              </>
            ) : (
              <>
                <Input
                  label={t("inventory.searchProducts")}
                  value={search}
                  onChange={(event) => setSearch(event.target.value)}
                />
                <Select
                  label={t("inventory.selectProduct")}
                  value={productId}
                  disabled={loadingProducts}
                  onChange={(event) => {
                    setProductId(event.target.value);
                    setPackages([]);
                    setPackageId("");
                  }}
                >
                  <option value="">{t("inventory.chooseProduct")}</option>
                  {products.map((product) => (
                    <option key={product.id} value={product.id}>
                      {selectedProductLabel(product)}
                    </option>
                  ))}
                </Select>
                <Select
                  label={t("inventory.selectPackage")}
                  value={packageId}
                  disabled={!productId}
                  onChange={(event) => setPackageId(event.target.value)}
                >
                  <option value="">{t("inventory.choosePackage")}</option>
                  {packages.map((entry) => (
                    <option key={entry.package.id} value={entry.package.id}>
                      {entry.package.packageLabel}
                    </option>
                  ))}
                </Select>
              </>
            )}
            <Input
              label={t("inventory.quantity")}
              required
              type="number"
              min="1"
              step="1"
              value={quantity}
              onChange={(event) => setQuantity(event.target.value)}
            />
            <Input
              label={t("inventory.reason")}
              required
              placeholder={t("inventory.reasonExample")}
              value={reason}
              onChange={(event) => setReason(event.target.value)}
            />
            <details>
              <summary>{t("inventory.advancedDetails")}</summary>
              <div className="stack">
                <Input
                  label={t("inventory.batchNumber")}
                  value={lotNumber}
                  onChange={(event) => setLotNumber(event.target.value)}
                />
                <Input
                  label={t("inventory.expiryDate")}
                  type="date"
                  value={expiryDate}
                  onChange={(event) => setExpiryDate(event.target.value)}
                />
                <Input
                  label={t("inventory.purchaseCost")}
                  inputMode="decimal"
                  value={cost}
                  onChange={(event) => setCost(event.target.value)}
                />
              </div>
            </details>
            <p className="muted">{t("inventory.requiredHelp")}</p>
            <div className="row end">
              <Button
                type="button"
                variant="secondary"
                disabled={submitting}
                onClick={() => onOpenChange(false)}
              >
                {t("common.cancel")}
              </Button>
              <Button
                type="submit"
                variant="primary"
                loading={submitting}
                disabled={
                  submitting ||
                  !branchId ||
                  !quantity ||
                  !reason.trim() ||
                  (creating
                    ? !productNameInput.trim() || !packageLabel.trim()
                    : !productId || !packageId)
                }
              >
                {creating
                  ? t("inventory.createAndAdd")
                  : t("inventory.saveStock")}
              </Button>
            </div>
          </form>
        )}
      </div>
    </Modal>
  );
}

function AdjustStockDialog({
  open,
  onOpenChange,
  rows,
  branchId,
  notify,
  onDone,
  onError,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  rows: StockRow[];
  branchId: string;
  notify: (message: string) => void;
  onDone: () => void;
  onError: (message: string) => void;
}) {
  const [packageId, setPackageId] = useState(rows[0] ? rows[0].packageId : "");
  const [quantity, setQuantity] = useState("");
  const [reason, setReason] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const packageRows = rows.filter((r) => r.batches.length > 0);
  const selected = packageRows.find((r) => r.packageId === packageId);
  return (
    <Modal
      open={open}
      onOpenChange={onOpenChange}
      title="Adjust stock"
      description="Set a new target quantity for one package. The ledger posts a signed correction movement."
    >
      <div className="stack">
        <Select
          label="Product / package"
          value={packageId}
          onChange={(e) => setPackageId(e.target.value)}
        >
          {packageRows.map((r) => (
            <option key={r.packageId} value={r.packageId}>
              {productName(r)} — {r.packageLabel}
            </option>
          ))}
        </Select>
        {selected && (
          <p className="muted">
            Current stock: {selected.quantity} across {selected.batchCount}{" "}
            batch(es). If you decrease, stock is removed first-expiry-first-out.
          </p>
        )}
        <Input
          label="New stock quantity"
          type="number"
          min="0"
          value={quantity}
          onChange={(e) => setQuantity(e.target.value)}
          placeholder="0"
        />
        <Textarea
          label="Reason / reference"
          required
          value={reason}
          onChange={(e) => setReason(e.target.value)}
          placeholder="Cycle count, damaged box, manual correction…"
        />
        <div className="row end">
          <Button
            variant="secondary"
            disabled={submitting}
            onClick={() => onOpenChange(false)}
          >
            Cancel
          </Button>
          <Button
            variant="primary"
            loading={submitting}
            disabled={!packageId || quantity === "" || !reason.trim()}
            onClick={() => {
              setSubmitting(true);
              void adjustStock({
                branchId,
                productPackageId: packageId,
                newQuantity: Number(quantity),
                reason: reason.trim(),
                user: null,
              })
                .then(() => {
                  notify("Stock adjusted. The movement was posted.");
                  onDone();
                })
                .catch((err: unknown) => {
                  setSubmitting(false);
                  onError(
                    err instanceof Error
                      ? err.message
                      : "Stock could not be adjusted.",
                  );
                });
            }}
          >
            Apply
          </Button>
        </div>
      </div>
    </Modal>
  );
}
function TransferStockDialog({
  open,
  onOpenChange,
  rows,
  branchId,
  notify,
  onDone,
  onError,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  rows: StockRow[];
  branchId: string;
  notify: (message: string) => void;
  onDone: () => void;
  onError: (message: string) => void;
}) {
  const { branches } = useBranch();
  const [packageId, setPackageId] = useState(rows[0] ? rows[0].packageId : "");
  const [batchId, setBatchId] = useState("");
  const [quantity, setQuantity] = useState("");
  const [toBranchId, setToBranchId] = useState("");
  const [reason, setReason] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const packageRows = rows.filter((r) => r.batches.length > 0);
  const selected = packageRows.find((r) => r.packageId === packageId);
  const batches = selected ? selected.batches : [];
  const effectiveBatchId = batchId || (batches[0] ? batches[0].batchId : "");
  const selectedBatch = batches.find((b) => b.batchId === effectiveBatchId);
  const destinations = branches.filter((b) => b.id !== branchId && b.isActive);
  return (
    <Modal
      open={open}
      onOpenChange={onOpenChange}
      title="Transfer stock"
      description="Move stock between branches. Both sides of the movement are posted with a shared reference."
    >
      <div className="stack">
        <Select
          label="Product / package"
          value={packageId}
          onChange={(e) => {
            setPackageId(e.target.value);
            setBatchId("");
          }}
        >
          {packageRows.map((r) => (
            <option key={r.packageId} value={r.packageId}>
              {productName(r)} — {r.packageLabel}
            </option>
          ))}
        </Select>
        <Select
          label="Batch"
          value={effectiveBatchId}
          onChange={(e) => setBatchId(e.target.value)}
        >
          {batches.map((b) => (
            <option key={b.batchId} value={b.batchId}>
              {b.batchNumber} · {b.quantity} in stock
              {b.expiryDate ? ` · ${b.expiryDate}` : ""}
            </option>
          ))}
        </Select>
        {selectedBatch && (
          <p className="muted">
            Available on this batch: {selectedBatch.quantity}
          </p>
        )}
        <Input
          label="Transfer quantity"
          type="number"
          min="1"
          value={quantity}
          onChange={(e) => setQuantity(e.target.value)}
        />
        <Select
          label="Destination branch"
          value={toBranchId}
          onChange={(e) => setToBranchId(e.target.value)}
        >
          <option value="">Select a branch…</option>
          {destinations.map((b) => (
            <option key={b.id} value={b.id}>
              {b.name}
            </option>
          ))}
        </Select>
        <Textarea
          label="Reason / reference"
          value={reason}
          onChange={(e) => setReason(e.target.value)}
          placeholder="Inter-branch restock…"
        />
        <div className="row end">
          <Button
            variant="secondary"
            disabled={submitting}
            onClick={() => onOpenChange(false)}
          >
            Cancel
          </Button>
          <Button
            variant="primary"
            loading={submitting}
            disabled={
              !effectiveBatchId ||
              !quantity ||
              Number(quantity) <= 0 ||
              !toBranchId
            }
            onClick={() => {
              setSubmitting(true);
              void transferStock({
                fromBranchId: branchId,
                toBranchId,
                productPackageId: packageId,
                batchId: effectiveBatchId,
                quantity: Number(quantity),
                reason: reason.trim() || "Inter-branch transfer",
                user: null,
              })
                .then(() => {
                  notify("Stock transferred between branches.");
                  onDone();
                })
                .catch((err: unknown) => {
                  setSubmitting(false);
                  onError(
                    err instanceof Error
                      ? err.message
                      : "The transfer could not be posted.",
                  );
                });
            }}
          >
            Apply
          </Button>
        </div>
      </div>
    </Modal>
  );
}
function ReorderDialog({
  open,
  onOpenChange,
  row,
  branchId,
  notify,
  onDone,
  onError,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  row: StockRow | null;
  branchId: string;
  notify: (message: string) => void;
  onDone: () => void;
  onError: (message: string) => void;
}) {
  const [level, setLevel] = useState(row ? String(row.reorderLevel) : "0");
  const [submitting, setSubmitting] = useState(false);
  return (
    <Modal
      open={open}
      onOpenChange={onOpenChange}
      title="Reorder level"
      description="Set the threshold at which this package is flagged Low stock. Zero disables the threshold."
    >
      <div className="stack">
        {row && (
          <p className="muted">
            {productName(row)} — current stock {row.quantity}, current reorder{" "}
            {row.reorderLevel}
          </p>
        )}
        <Input
          label="Reorder level"
          type="number"
          min="0"
          value={level}
          onChange={(e) => setLevel(e.target.value)}
        />
        <div className="row end">
          <Button
            variant="secondary"
            disabled={submitting}
            onClick={() => onOpenChange(false)}
          >
            Cancel
          </Button>
          <Button
            variant="primary"
            loading={submitting}
            disabled={!row || level === ""}
            onClick={() => {
              if (!row) return;
              setSubmitting(true);
              void setReorderLevel({
                branchId,
                productPackageId: row.packageId,
                reorderLevel: Number(level),
              })
                .then(() => {
                  notify("Reorder level updated.");
                  onDone();
                })
                .catch((err: unknown) => {
                  setSubmitting(false);
                  onError(
                    err instanceof Error
                      ? err.message
                      : "The reorder level could not be updated.",
                  );
                });
            }}
          >
            Save
          </Button>
        </div>
      </div>
    </Modal>
  );
}
export function StockMovements() {
  const desktop = isDesktopRuntime();
  const { branchId } = useBranch();
  const [search, setSearch] = useState("");
  const [type, setType] = useState("");
  const [page, setPage] = useState(1);
  const [data, setData] = useState<StockMovementPage | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    if (!desktop) return;
    let active = true;
    const timer = window.setTimeout(
      () => {
        if (!active) return;
        setLoading(true);
        setError("");
        listStockMovements({
          branchId,
          movementType: type || null,
          search: search || null,
          sort: "occurredAt",
          sortDirection: "desc",
          limit: PAGE_SIZE,
          offset: (page - 1) * PAGE_SIZE,
        })
          .then((result) => {
            if (!active) return;
            setData(result);
            setLoading(false);
          })
          .catch((err: unknown) => {
            if (!active) return;
            setError(
              err instanceof Error
                ? err.message
                : "The stock ledger could not be loaded.",
            );
            setLoading(false);
          });
      },
      search ? 250 : 0,
    );
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [desktop, branchId, search, type, page, attempt]);
  const pages = data ? Math.max(1, Math.ceil(data.total / data.limit)) : 1;
  const columns: Column<StockMovementItem>[] = [
    {
      key: "date",
      header: "Date / reference",
      render: (r) => (
        <>
          {r.occurredAt.slice(0, 16).replace("T", " ")}
          <span className="cell-secondary">{r.id}</span>
        </>
      ),
    },
    {
      key: "type",
      header: "Type",
      render: (r) => <Status value={movementTypeLabel(r.movementType)} />,
    },
    {
      key: "product",
      header: "Product / package",
      render: (r) => (
        <>
          {r.product}
          <span className="cell-secondary">{r.pack}</span>
        </>
      ),
    },
    { key: "batch", header: "Batch", render: (r) => r.batch ?? "—" },
    { key: "branch", header: "Branch", render: (r) => r.branch },
    { key: "in", header: "In", render: (r) => r.incoming ?? "—" },
    { key: "out", header: "Out", render: (r) => r.outgoing ?? "—" },
    { key: "user", header: "User", render: (r) => r.user || "—" },
    { key: "reason", header: "Reason", render: (r) => r.reason || "—" },
  ];
  return (
    <div className="feature-page">
      <PageHeader
        title="Stock movements"
        description="A chronological, branch-scoped ledger of stock activity."
      />
      <RouteTabs items={inventoryTabs} />
      {!desktop && (
        <Alert title="Desktop app required" tone="warning">
          Movements are stored in the local database. Open M² Health as the
          desktop app to browse the ledger.
        </Alert>
      )}
      {desktop && error && (
        <Alert title="The stock ledger could not be loaded" tone="danger">
          {error}
          <Button size="sm" onClick={() => setAttempt((v) => v + 1)}>
            Retry
          </Button>
        </Alert>
      )}
      {!desktop ? (
        <Card>
          <EmptyState
            title="Movements require the desktop app"
            description="The stock ledger lives in the local SQLite database."
          />
        </Card>
      ) : (
        <Card>
          <TableToolbar>
            <SearchInput
              label="Search movements"
              placeholder="Search product, batch, reason…"
              value={search}
              onChange={(e) => {
                setSearch(e.target.value);
                setPage(1);
              }}
            />
            <Select
              label="Movement type"
              value={type}
              onChange={(e) => {
                setType(e.target.value);
                setPage(1);
              }}
            >
              <option value="">All movement types</option>
              {MOVEMENT_TYPES.map((t) => (
                <option key={t} value={t}>
                  {movementTypeLabel(t)}
                </option>
              ))}
            </Select>
          </TableToolbar>
          {loading && !data ? (
            <div className="stack" role="status">
              <Alert title="Loading movements…" tone="info" />
            </div>
          ) : (
            <Table
              label="Stock movements"
              rows={data?.items ?? []}
              columns={columns}
              rowKey={(r) => r.id}
            />
          )}
          <div className="grid-footer">
            <span className="muted">
              {data ? `${data.total} movements · branch-scoped` : "—"}
            </span>
            <Pagination page={page} pages={pages} onChange={setPage} />
          </div>
        </Card>
      )}
    </div>
  );
}
