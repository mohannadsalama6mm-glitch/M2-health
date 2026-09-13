import { useEffect, useState, type FormEvent } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { Plus } from "lucide-react";
import {
  PageHeader,
  LinkButton,
  BackLink,
  Card,
  SectionHeader,
  TableToolbar,
  SearchInput,
  DataGrid,
  PriceDisplay,
  Status,
  Select,
  Input,
  Textarea,
  Button,
  Table,
  DetailList,
  FormSection,
  Modal,
  Alert,
  Badge,
  EmptyState,
} from "../../design-system";
import { isDesktopRuntime } from "../../lib/tauri/client";
import { useBranch } from "../../app/BranchContext";
import { useDemo } from "../../app/DemoContext";
import {
  completePurchase,
  voidPurchase,
  listPurchases,
  getPurchase,
  purchasePosSearch,
} from "../../lib/tauri/purchases";
import { listSuppliers } from "../../lib/tauri/partners";
import { toMajor, toMinor } from "../../lib/tauri/inventory";
import type { Supplier } from "../../lib/tauri/partners.types";
import type {
  PurchaseDetail,
  PurchaseProduct,
  PurchaseRow,
} from "../../lib/tauri/purchases.types";
const formatTime = (iso: string) =>
  iso ? iso.slice(0, 16).replace("T", " ") : "—";
const statusLabel = (status: string) =>
  status === "completed" ? "Completed" : "Void";
export function Purchases() {
  const desktop = isDesktopRuntime();
  const { branchId } = useBranch();
  const [rows, setRows] = useState<PurchaseRow[]>([]);
  const [search, setSearch] = useState("");
  const [status, setStatus] = useState("");
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
        listPurchases({
          branchId,
          search: search || null,
          status: status || null,
        })
          .then((result) => {
            if (!active) return;
            setRows(result.items);
            setLoading(false);
          })
          .catch((err: unknown) => {
            if (!active) return;
            setError(
              err instanceof Error
                ? err.message
                : "Purchases could not be loaded.",
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
  }, [desktop, branchId, search, status, attempt]);
  if (!desktop)
    return (
      <EmptyState
        title="Connect to the desktop app to view purchases."
        description=""
      />
    );
  return (
    <div className="feature-page">
      <PageHeader
        title="Purchases"
        description="Supplier orders and receiving history, sourced from the local database."
        actions={
          <LinkButton primary to="/purchases/new">
            <Plus size={16} />
            Complete purchase
          </LinkButton>
        }
      />
      {error && (
        <Alert title="Purchases could not be loaded" tone="danger">
          {error}
          <Button size="sm" onClick={() => setAttempt((v) => v + 1)}>
            Retry
          </Button>
        </Alert>
      )}
      <TableToolbar>
        <SearchInput
          label="Search purchases"
          placeholder="Search by purchase or invoice number…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <div className="row">
          <Select
            label="Status"
            value={status}
            onChange={(e) => setStatus(e.target.value)}
          >
            <option value="">All statuses</option>
            <option value="completed">Completed</option>
            <option value="void">Void</option>
          </Select>
        </div>
      </TableToolbar>
      {loading && rows.length === 0 ? (
        <Card>
          <SectionHeader title="Loading purchases…" />
        </Card>
      ) : (
        <DataGrid
          label="Purchases"
          rows={rows}
          rowKey={(p) => p.id}
          searchText={(p) =>
            `${p.purchaseNumber} ${p.invoiceNumber} ${p.supplierName ?? ""}`
          }
          columns={[
            {
              key: "purchase",
              header: "Purchase / supplier",
              render: (p) => (
                <Link className="text-link" to={`/purchases/${p.id}`}>
                  {p.purchaseNumber}
                  <span className="cell-secondary">
                    {p.supplierName ?? "Supplier —"}
                  </span>
                </Link>
              ),
            },
            {
              key: "invoice",
              header: "Invoice",
              render: (p) => p.invoiceNumber || "—",
            },
            {
              key: "date",
              header: "Date",
              render: (p) => formatTime(p.completedAt),
            },
            {
              key: "status",
              header: "Status",
              render: (p) => <Status value={statusLabel(p.status)} />,
            },
            { key: "items", header: "Items", render: (p) => p.itemCount },
            {
              key: "total",
              header: "Total",
              render: (p) => <PriceDisplay amount={toMajor(p.totalMinor)} />,
            },
            {
              key: "paid",
              header: "Paid",
              render: (p) => <PriceDisplay amount={toMajor(p.paidMinor)} />,
            },
            { key: "user", header: "User", render: (p) => p.user || "—" },
          ]}
        />
      )}
    </div>
  );
}
export function PurchaseDetails() {
  const { id } = useParams();
  const desktop = isDesktopRuntime();
  const { notify } = useDemo();
  const [detail, setDetail] = useState<PurchaseDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);
  const [voidOpen, setVoidOpen] = useState(false);
  const [voidReason, setVoidReason] = useState("");
  const [voidSubmitting, setVoidSubmitting] = useState(false);
  useEffect(() => {
    if (!desktop || !id) return;
    let active = true;
    getPurchase(id)
      .then((result) => {
        if (!active) return;
        setDetail(result);
        setError("");
        setLoading(false);
      })
      .catch((err: unknown) => {
        if (!active) return;
        setError(
          err instanceof Error ? err.message : "Purchase could not be loaded.",
        );
        setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [desktop, id, attempt]);
  if (!desktop)
    return (
      <EmptyState
        title="Connect to the desktop app to view purchases."
        description=""
      />
    );
  if (loading)
    return (
      <Card>
        <SectionHeader title="Loading purchase…" />
      </Card>
    );
  if (error)
    return (
      <>
        <BackLink to="/purchases" label="Purchases" />
        <Alert title="Purchase could not be loaded" tone="danger">
          {error}
          <Button size="sm" onClick={() => setAttempt((v) => v + 1)}>
            Retry
          </Button>
        </Alert>
      </>
    );
  if (!detail)
    return (
      <EmptyState
        title="Purchase not found"
        description="The purchase could not be loaded."
        action={<LinkButton to="/purchases">Return to purchases</LinkButton>}
      />
    );
  const { purchase, branchName, supplierName, items } = detail;
  const handleVoid = () => {
    if (!voidReason.trim()) return;
    setVoidSubmitting(true);
    voidPurchase({ purchaseId: purchase.id, reason: voidReason })
      .then(() => {
        notify("Purchase voided and stock reversed.");
        setAttempt((v) => v + 1);
        setVoidOpen(false);
        setVoidReason("");
        setVoidSubmitting(false);
      })
      .catch((err: unknown) => {
        setVoidSubmitting(false);
        notify(
          err instanceof Error
            ? err.message
            : "Purchase could not be voided.",
        );
      });
  };
  return (
    <>
      <BackLink to="/purchases" label="Purchases" />
      <div className="feature-page">
        <PageHeader
          title={purchase.purchaseNumber}
          description={`${branchName} · ${statusLabel(purchase.status)}`}
          actions={
            <>
              <Status value={statusLabel(purchase.status)} />
              {purchase.status === "completed" && (
                <Button variant="danger" onClick={() => setVoidOpen(true)}>
                  Void purchase
                </Button>
              )}
            </>
          }
        />
        <div className="content-stack">
          <Card>
            <DetailList
              items={[
                { label: "Branch", value: branchName },
                { label: "Supplier", value: supplierName ?? "—" },
                {
                  label: "Invoice number",
                  value: purchase.invoiceNumber || "—",
                },
                {
                  label: "Purchase number",
                  value: purchase.purchaseNumber,
                },
                { label: "Completed", value: formatTime(purchase.completedAt) },
                {
                  label: "Payment method",
                  value: purchase.paymentMethod || "—",
                },
                {
                  label: "Subtotal",
                  value: (
                    <PriceDisplay amount={toMajor(purchase.subtotalMinor)} />
                  ),
                },
                {
                  label: "Discount",
                  value: (
                    <PriceDisplay amount={toMajor(purchase.discountMinor)} />
                  ),
                },
                {
                  label: "Tax",
                  value: <PriceDisplay amount={toMajor(purchase.taxMinor)} />,
                },
                {
                  label: "Total",
                  value: <PriceDisplay amount={toMajor(purchase.totalMinor)} />,
                },
                {
                  label: "Paid",
                  value: <PriceDisplay amount={toMajor(purchase.paidMinor)} />,
                },
                {
                  label: "Change",
                  value: (
                    <PriceDisplay amount={toMajor(purchase.changeMinor)} />
                  ),
                },
                { label: "User", value: purchase.user || "—" },
                { label: "Note", value: purchase.note || "—" },
              ]}
            />
          </Card>
          {purchase.status === "void" && (
            <Alert title="This purchase was voided" tone="danger">
              Voided by {purchase.voidedBy || "—"}
              {purchase.voidReason ? `: ${purchase.voidReason}` : ""}
            </Alert>
          )}
          <Card>
            <Table
              label="Purchase items"
              rows={items}
              rowKey={(l) => l.id}
              columns={[
                {
                  key: "product",
                  header: "Product / package",
                  render: (l) => (
                    <>
                      {l.productName}
                      <span className="cell-secondary">{l.packageLabel}</span>
                    </>
                  ),
                },
                { key: "quantity", header: "Quantity", render: (l) => l.quantity },
                {
                  key: "cost",
                  header: "Unit cost",
                  render: (l) => (
                    <PriceDisplay amount={toMajor(l.unitCostMinor)} />
                  ),
                },
                {
                  key: "lineTotal",
                  header: "Line total",
                  render: (l) => (
                    <PriceDisplay amount={toMajor(l.lineTotalMinor)} />
                  ),
                },
                {
                  key: "batches",
                  header: "Lots",
                  render: (l) =>
                    l.batches.length > 0 ? (
                      <Badge>{l.batches.length}</Badge>
                    ) : (
                      "—"
                    ),
                },
              ]}
            />
            <p className="muted">
              Stock was added to local lots when this purchase was completed.
            </p>
          </Card>
        </div>
        <Modal
          open={voidOpen}
          onOpenChange={setVoidOpen}
          title="Void this purchase?"
          description="Voiding reverses the posted stock and ledger movements."
        >
          <div className="content-stack">
            <Input
              label="Void reason"
              placeholder="Required"
              value={voidReason}
              onChange={(e) => setVoidReason(e.target.value)}
            />
            <div className="row end">
              <Button onClick={() => setVoidOpen(false)}>Cancel</Button>
              <Button
                variant="danger"
                loading={voidSubmitting}
                disabled={!voidReason.trim()}
                onClick={handleVoid}
              >
                Void purchase
              </Button>
            </div>
          </div>
        </Modal>
      </div>
    </>
  );
}
interface Line {
  productPackageId: string;
  productName: string;
  packageLabel: string;
  quantity: number;
  unitCostMinor: number;
}
export function PurchaseForm() {
  const desktop = isDesktopRuntime();
  const { branchId } = useBranch();
  const { notify } = useDemo();
  const navigate = useNavigate();
  const [suppliers, setSuppliers] = useState<Supplier[]>([]);
  const [supplierId, setSupplierId] = useState("");
  const [search, setSearch] = useState("");
  const [products, setProducts] = useState<PurchaseProduct[]>([]);
  const [lines, setLines] = useState<Line[]>([]);
  const [discount, setDiscount] = useState("0");
  const [tax, setTax] = useState("0");
  const [payment, setPayment] = useState("Cash");
  const [paid, setPaid] = useState("");
  const [invoice, setInvoice] = useState("");
  const [note, setNote] = useState("");
  const [error, setError] = useState("");
  const [submitting, setSubmitting] = useState(false);
  useEffect(() => {
    if (!desktop) return;
    let active = true;
    listSuppliers({ isActive: true })
      .then((result) => {
        if (active) setSuppliers(result.items);
      })
      .catch(() => {
        if (active) setSuppliers([]);
      });
    return () => {
      active = false;
    };
  }, [desktop]);
  useEffect(() => {
    if (!desktop) return;
    let active = true;
    const timer = window.setTimeout(
      () => {
        if (!active) return;
        purchasePosSearch({ branchId, search: search || null, limit: 50 })
          .then((result) => {
            if (active) setProducts(result.items);
          })
          .catch(() => {
            if (active) setProducts([]);
          });
      },
      search ? 250 : 0,
    );
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [desktop, branchId, search]);
  const subtotalMinor = lines.reduce(
    (sum, l) => sum + l.unitCostMinor * l.quantity,
    0,
  );
  const discountMinor = toMinor(discount) ?? 0;
  const taxMinor = toMinor(tax) ?? 0;
  const totalMinor = Math.max(0, subtotalMinor - discountMinor + taxMinor);
  const paidMinor = toMinor(paid) ?? 0;
  const changeMinor = Math.max(0, paidMinor - totalMinor);
  const canSubmit =
    lines.length > 0 &&
    lines.every((l) => l.quantity >= 1 && l.unitCostMinor >= 0) &&
    paidMinor >= totalMinor;
  const addLine = (p: PurchaseProduct) => {
    setLines((current) =>
      current.some((l) => l.productPackageId === p.packageId)
        ? current.map((l) =>
            l.productPackageId === p.packageId
              ? { ...l, quantity: l.quantity + 1 }
              : l,
          )
        : [
            ...current,
            {
              productPackageId: p.packageId,
              productName: p.productName,
              packageLabel: p.packageLabel,
              quantity: 1,
              unitCostMinor: p.costPriceMinor ?? 0,
            },
          ],
    );
  };
  const handleSubmit = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!canSubmit || submitting) return;
    setSubmitting(true);
    setError("");
    completePurchase({
      branchId,
      supplierId: supplierId || null,
      invoiceNumber: invoice.trim() || undefined,
      discountMinor,
      taxMinor,
      paidMinor,
      paymentMethod: payment.toLowerCase(),
      user: null,
      note: note.trim() || undefined,
      lines: lines.map((l) => ({
        productPackageId: l.productPackageId,
        quantity: l.quantity,
        unitCostMinor: l.unitCostMinor,
      })),
    })
      .then((result) => {
        notify("Purchase completed. Stock has been posted to local lots.");
        navigate(`/purchases/${result.purchase.id}`);
      })
      .catch((err: unknown) => {
        setSubmitting(false);
        setError(
          err instanceof Error
            ? err.message
            : "The purchase could not be completed.",
        );
      });
  };
  if (!desktop)
    return (
      <EmptyState
        title="Connect to the desktop app to complete a purchase."
        description=""
      />
    );
  return (
    <>
      <BackLink to="/purchases" label="Purchases" />
      <div className="feature-page">
        <PageHeader
          title="Complete purchase"
          description="Select products, set quantities, and pay the supplier. Completing posts stock and ledger movements immediately."
        />
        <form className="content-stack" onSubmit={handleSubmit}>
          {error && (
            <Alert title="Purchase could not be completed" tone="danger">
              {error}
            </Alert>
          )}
          <FormSection title="Supplier">
            <Select
              label="Supplier"
              showLabel
              value={supplierId}
              onChange={(e) => setSupplierId(e.target.value)}
            >
              <option value="">No supplier</option>
              {suppliers.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.name}
                </option>
              ))}
            </Select>
          </FormSection>
          <Card>
            <SectionHeader title="Add products" />
            <SearchInput
              label="Search products"
              placeholder="Search name…"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
            <Table
              label="Product search results"
              rows={products}
              rowKey={(p) => p.packageId}
              columns={[
                {
                  key: "product",
                  header: "Product / package",
                  render: (p) => (
                    <>
                      {p.productName}
                      <span className="cell-secondary">{p.packageLabel}</span>
                    </>
                  ),
                },
                {
                  key: "pack",
                  header: "Pack size",
                  render: (p) => p.packSize ?? "—",
                },
                {
                  key: "cost",
                  header: "Cost",
                  render: (p) =>
                    p.costPriceMinor != null ? (
                      <PriceDisplay amount={toMajor(p.costPriceMinor)} />
                    ) : (
                      <span className="muted">—</span>
                    ),
                },
                {
                  key: "add",
                  header: "Action",
                  render: (p) => (
                    <Button size="sm" variant="outline" onClick={() => addLine(p)}>
                      Add
                    </Button>
                  ),
                },
              ]}
            />
          </Card>
          <Card>
            <div className="row spread">
              <h3>Purchase lines</h3>
              <span className="muted">
                {lines.length} line{lines.length === 1 ? "" : "s"}
              </span>
            </div>
            <Table
              label="Purchase lines"
              rows={lines}
              rowKey={(l) => l.productPackageId}
              columns={[
                {
                  key: "product",
                  header: "Product / package",
                  render: (l) => (
                    <>
                      {l.productName}
                      <span className="cell-secondary">{l.packageLabel}</span>
                    </>
                  ),
                },
                {
                  key: "quantity",
                  header: "Quantity",
                  render: (l) => (
                    <Input
                      hideLabel
                      label={`Quantity ${l.productPackageId}`}
                      type="number"
                      min={1}
                      value={l.quantity}
                      onChange={(e) =>
                        setLines(
                          lines.map((v) =>
                            v.productPackageId === l.productPackageId
                              ? {
                                  ...v,
                                  quantity: Math.max(1, Number(e.target.value)),
                                }
                              : v,
                          ),
                        )
                      }
                    />
                  ),
                },
                {
                  key: "cost",
                  header: "Unit cost (EGP)",
                  render: (l) => (
                    <Input
                      hideLabel
                      label={`Unit cost ${l.productPackageId}`}
                      type="number"
                      min={0}
                      step="0.01"
                      value={toMajor(l.unitCostMinor)}
                      onChange={(e) =>
                        setLines(
                          lines.map((v) =>
                            v.productPackageId === l.productPackageId
                              ? {
                                  ...v,
                                  unitCostMinor: toMinor(e.target.value) ?? 0,
                                }
                              : v,
                          ),
                        )
                      }
                    />
                  ),
                },
                {
                  key: "lineTotal",
                  header: "Line total",
                  render: (l) => (
                    <PriceDisplay
                      amount={toMajor(l.unitCostMinor * l.quantity)}
                    />
                  ),
                },
                {
                  key: "remove",
                  header: "Action",
                  render: (l) => (
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() =>
                        setLines(
                          lines.filter(
                            (v) => v.productPackageId !== l.productPackageId,
                          ),
                        )
                      }
                    >
                      Remove
                    </Button>
                  ),
                },
              ]}
            />
          </Card>
          <Card>
            <SectionHeader title="Totals" />
            <div className="form-grid">
              <Input
                label="Discount (EGP)"
                type="number"
                min={0}
                step="0.01"
                value={discount}
                onChange={(e) => setDiscount(e.target.value)}
              />
              <Input
                label="Tax (EGP)"
                type="number"
                min={0}
                step="0.01"
                value={tax}
                onChange={(e) => setTax(e.target.value)}
              />
            </div>
            <DetailList
              items={[
                {
                  label: "Subtotal",
                  value: <PriceDisplay amount={toMajor(subtotalMinor)} />,
                },
                {
                  label: "Discount",
                  value: <PriceDisplay amount={toMajor(discountMinor)} />,
                },
                {
                  label: "Tax",
                  value: <PriceDisplay amount={toMajor(taxMinor)} />,
                },
                {
                  label: "Total",
                  value: <PriceDisplay amount={toMajor(totalMinor)} />,
                },
                {
                  label: "Paid",
                  value: <PriceDisplay amount={toMajor(paidMinor)} />,
                },
                {
                  label: "Change",
                  value: <PriceDisplay amount={toMajor(changeMinor)} />,
                },
              ]}
            />
          </Card>
          <FormSection title="Payment & completion">
            <Select
              label="Payment method"
              showLabel
              value={payment}
              onChange={(e) => setPayment(e.target.value)}
            >
              <option>Cash</option>
              <option>Card</option>
              <option>Other</option>
            </Select>
            <Input
              label="Paid amount (EGP)"
              type="number"
              min={0}
              step="0.01"
              value={paid}
              onChange={(e) => setPaid(e.target.value)}
            />
            <Input
              label="Invoice number"
              value={invoice}
              onChange={(e) => setInvoice(e.target.value)}
            />
            <Textarea
              label="Notes"
              value={note}
              onChange={(e) => setNote(e.target.value)}
            />
          </FormSection>
          <div className="form-actions">
            <LinkButton to="/purchases">Cancel</LinkButton>
            <Button
              type="submit"
              variant="primary"
              loading={submitting}
              disabled={!canSubmit}
            >
              Complete purchase
            </Button>
          </div>
          {!canSubmit && (
            <p className="muted">
              {lines.length === 0
                ? "Add at least one product line above."
                : "Enter a paid amount at least equal to the total to complete the purchase."}
            </p>
          )}
        </form>
      </div>
    </>
  );
}
export function ReceivePurchase() {
  const { id } = useParams();
  return (
    <>
      <BackLink to={`/purchases/${id}`} label="Purchase" />
      <div className="feature-page">
        <PageHeader
          title="Receive purchase"
          description="Receiving is handled when the purchase is completed."
        />
        <Alert title="No separate receiving step" tone="info">
          Receiving is part of completing a purchase. This order has already
          posted its stock.
        </Alert>
        <EmptyState
          title="Stock already received"
          description="Receiving is part of completing a purchase. This order has already posted its stock to local lots."
          action={<LinkButton to={`/purchases/${id}`}>View purchase</LinkButton>}
        />
      </div>
    </>
  );
}