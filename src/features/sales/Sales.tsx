import { useEffect, useRef, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { ShoppingCart, Plus, Pause, Play, Trash2, Printer } from "lucide-react";
import {
  Card,
  SectionHeader,
  SearchInput,
  Input,
  Select,
  Button,
  Table,
  QuantityStepper,
  PriceDisplay,
  Badge,
  EmptyState,
  Modal,
  ConfirmationModal,
  OutputPreview,
  DetailList,
  DataGrid,
  Status,
  BackLink,
  LinkButton,
  Alert,
  PageHeader,
} from "../../design-system";
import { isDesktopRuntime } from "../../lib/tauri/client";
import { useBranch } from "../../app/BranchContext";
import { useDemo } from "../../app/DemoContext";
import {
  completeSale,
  voidSale,
  listSales,
  getSale,
  posSearch,
} from "../../lib/tauri/sales";
import { listCustomers } from "../../lib/tauri/partners";
import { toMajor, toMinor } from "../../lib/tauri/inventory";
import type {
  PosProduct,
  Sale,
  SaleRow,
  SaleDetail,
} from "../../lib/tauri/sales.types";
import type { Customer } from "../../lib/tauri/partners.types";
import "./sales.css";
interface CartLine {
  packageId: string;
  productName: string;
  packageLabel: string;
  sellingPriceMinor: number;
  quantity: number;
}
export function Sales() {
  const desktop = isDesktopRuntime();
  const { branchId } = useBranch();
  const { notify } = useDemo();
  const [query, setQuery] = useState("");
  const [barcode, setBarcode] = useState("");
  const [customer, setCustomer] = useState("Walk-in customer");
  const [customerId, setCustomerId] = useState<string | null>(null);
  const [customersList, setCustomersList] = useState<Customer[]>([]);
  const [payment, setPayment] = useState("Cash");
  const [cash, setCash] = useState(0);
  const [discount, setDiscount] = useState(0);
  const [dialog, setDialog] = useState("");
  const [receipt, setReceipt] = useState<Sale | null>(null);
  const [searchError, setSearchError] = useState("");
  const [cart, setCart] = useState<CartLine[]>([]);
  const [held, setHeld] = useState<CartLine[][]>([]);
  const [products, setProducts] = useState<PosProduct[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);
  const [submitting, setSubmitting] = useState(false);
  const [historySales, setHistorySales] = useState<SaleRow[]>([]);
  const searchRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!desktop) return;
    let active = true;
    const timer = window.setTimeout(
      () => {
        if (!active) return;
        setLoading(true);
        setError("");
        posSearch({ branchId, search: query || null })
          .then((result) => {
            if (!active) return;
            setProducts(result.items);
            setLoading(false);
          })
          .catch((err: unknown) => {
            if (!active) return;
            setError(
              err instanceof Error
                ? err.message
                : "Products could not be loaded.",
            );
            setLoading(false);
          });
      },
      query ? 250 : 0,
    );
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [desktop, branchId, query, attempt]);
  useEffect(() => {
    if (!desktop) return;
    let active = true;
    listCustomers({ isActive: true })
      .then((result) => {
        if (active) setCustomersList(result.items);
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  }, [desktop, attempt]);
  useEffect(() => {
    if (!desktop || dialog !== "history") return;
    let active = true;
    listSales({ branchId })
      .then((result) => {
        if (active) setHistorySales(result.items);
      })
      .catch(() => {
        if (active) setHistorySales([]);
      });
    return () => {
      active = false;
    };
  }, [desktop, dialog, branchId]);
  if (!desktop)
    return (
      <EmptyState
        title="Connect to the desktop app to use the POS."
        description=""
      />
    );
  const subtotalMinor = cart.reduce(
    (sum, l) => sum + l.sellingPriceMinor * l.quantity,
    0,
  );
  const discountMinor = toMinor(String(discount)) ?? 0;
  const totalMinor = Math.max(0, subtotalMinor - discountMinor);
  const cashMinor = payment === "Cash" ? (toMinor(String(cash)) ?? 0) : 0;
  const changeMinor = Math.max(0, cashMinor - totalMinor);
  const add = (p: PosProduct) => {
    if (p.status === "out_of_stock") {
      notify("This product is out of stock.");
      return;
    }
    setCart((current) =>
      current.some((l) => l.packageId === p.packageId)
        ? current.map((l) =>
            l.packageId === p.packageId
              ? { ...l, quantity: l.quantity + 1 }
              : l,
          )
        : [
            ...current,
            {
              packageId: p.packageId,
              productName: p.productName,
              packageLabel: p.packageLabel,
              sellingPriceMinor: p.sellingPriceMinor ?? 0,
              quantity: 1,
            },
          ],
    );
    setSearchError("");
  };
  const handleBarcode = () => {
    posSearch({ branchId, search: barcode })
      .then((result) => {
        const match = result.items[0];
        if (match) {
          add(match);
          setBarcode("");
        } else {
          setSearchError("No product matches this barcode.");
        }
      })
      .catch(() => setSearchError("No product matches this barcode."));
  };
  const complete = () => {
    setSubmitting(true);
    completeSale({
      branchId,
      customerId,
      paymentMethod: payment === "Account" ? "other" : payment.toLowerCase(),
      paidMinor: payment === "Cash" ? cashMinor : totalMinor,
      discountMinor,
      customerName: customer,
      lines: cart.map((l) => ({
        productPackageId: l.packageId,
        quantity: l.quantity,
      })),
    })
      .then((result) => {
        setReceipt(result.sale);
        setCart([]);
        setCash(0);
        setDiscount(0);
        setDialog("");
        setAttempt((v) => v + 1);
        notify("Sale completed successfully.");
        setSubmitting(false);
      })
      .catch((err: unknown) => {
        setSubmitting(false);
        notify(
          err instanceof Error ? err.message : "Sale could not be completed.",
        );
      });
  };
  return (
    <div className="feature-page">
      <PageHeader
        title="POS / Sales"
        description="Find a product, build a basket, and preview checkout."
        actions={
          <>
            <Button onClick={() => setDialog("history")}>Sales history</Button>
            <Button disabled={!held.length} onClick={() => setDialog("held")}>
              <Play size={15} />
              Held sales ({held.length})
            </Button>
          </>
        }
      />
      {error && (
        <Alert title="Products could not be loaded" tone="danger">
          {error}
          <Button size="sm" onClick={() => setAttempt((v) => v + 1)}>
            Retry
          </Button>
        </Alert>
      )}
      <div className="pos-layout">
        <div className="content-stack">
          <Card>
            <SectionHeader
              title="Product lookup"
              icon={ShoppingCart}
              action={<Badge>Enter to add first match</Badge>}
            />
            <div ref={searchRef}>
              <SearchInput
                label="POS product search"
                placeholder="Search name or barcode…"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && products[0]) {
                    add(products[0]);
                    setQuery("");
                  }
                }}
              />
            </div>
            <div className="barcode-entry">
              <Input
                label="Barcode input"
                value={barcode}
                placeholder="Try 6221001000011"
                onChange={(e) => setBarcode(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleBarcode();
                }}
              />
              <Button onClick={handleBarcode}>Add barcode</Button>
            </div>
            {searchError && (
              <Alert title="Product not found" tone="warning">
                {searchError}
              </Alert>
            )}
          </Card>
          <Card>
            <SectionHeader
              title="Available products"
              action={<span className="muted">{products.length} matches</span>}
            />
            {loading && !products.length ? (
              <div className="stack" role="status">
                <Alert title="Loading products…" tone="info" />
              </div>
            ) : (
              <Table
                label="POS products"
                rows={products}
                rowKey={(p) => p.packageId}
                columns={[
                  {
                    key: "name",
                    header: "Product",
                    render: (p) => (
                      <>
                        {p.productName}
                        <span className="cell-secondary">
                          {p.packageLabel}
                        </span>
                      </>
                    ),
                  },
                  {
                    key: "stock",
                    header: "Available",
                    render: (p) => (
                      <span className="row">
                        {p.quantity}
                        <Badge
                          dot
                          tone={
                            p.status === "in_stock" ? "success" : "danger"
                          }
                        >
                          {p.status === "in_stock" ? "In stock" : "Out"}
                        </Badge>
                      </span>
                    ),
                  },
                  {
                    key: "price",
                    header: "Price",
                    render: (p) =>
                      p.sellingPriceMinor != null ? (
                        <PriceDisplay amount={toMajor(p.sellingPriceMinor)} />
                      ) : (
                        <span className="muted">—</span>
                      ),
                  },
                  {
                    key: "add",
                    header: "Add",
                    render: (p) => (
                      <Button
                        size="sm"
                        disabled={p.status === "out_of_stock"}
                        aria-label={`Add ${p.productName} to cart`}
                        onClick={() => add(p)}
                      >
                        <Plus size={15} />
                      </Button>
                    ),
                  },
                ]}
              />
            )}
          </Card>
        </div>
        <Card className="pos-cart">
          <SectionHeader
            title="Current sale"
            action={<Badge>{cart.length} lines</Badge>}
          />
          <Select
            label="Sale customer"
            value={customerId ?? ""}
            onChange={(e) => {
              const choice = customersList.find((c) => c.id === e.target.value);
              setCustomer(choice?.name ?? "Walk-in customer");
              setCustomerId(choice?.id ?? null);
            }}
          >
            <option value="">Walk-in customer</option>
            {customersList.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </Select>
          <div className="cart-lines">
            {!cart.length ? (
              <EmptyState
                title="Ready for your next sale"
                description="Search or enter a barcode to add an item."
                action={
                  <Button
                    onClick={() =>
                      searchRef.current?.querySelector("input")?.focus()
                    }
                  >
                    Focus product search
                  </Button>
                }
              />
            ) : (
              cart.map((l) => (
                <div className="cart-line" key={l.packageId}>
                  <div className="row spread">
                    <strong>{l.productName}</strong>
                    <Button
                      size="sm"
                      aria-label={`Remove ${l.productName}`}
                      onClick={() =>
                        setCart(
                          cart.filter((c) => c.packageId !== l.packageId),
                        )
                      }
                    >
                      <Trash2 size={14} />
                    </Button>
                  </div>
                  <p className="muted">
                    {l.packageLabel} ·{" "}
                    <PriceDisplay amount={toMajor(l.sellingPriceMinor)} />
                  </p>
                  <div className="row spread">
                    <QuantityStepper
                      min={1}
                      value={l.quantity}
                      onChange={(quantity) =>
                        setCart(
                          cart.map((c) =>
                            c.packageId === l.packageId
                              ? { ...c, quantity }
                              : c,
                          ),
                        )
                      }
                    />
                    <PriceDisplay
                      amount={toMajor(l.sellingPriceMinor * l.quantity)}
                    />
                  </div>
                </div>
              ))
            )}
          </div>
          <div className="checkout-summary">
            <DetailList
              items={[
                {
                  label: "Subtotal",
                  value: <PriceDisplay amount={toMajor(subtotalMinor)} />,
                },
                { label: "Tax", value: "0.00 EGP" },
              ]}
            />
            <Input
              type="number"
              min={0}
              max={toMajor(subtotalMinor)}
              label="Sale discount (EGP)"
              value={discount}
              onChange={(e) => setDiscount(Math.max(0, Number(e.target.value)))}
            />
            <div className="row spread checkout-total">
              <strong>Grand total</strong>
              <span className="receipt-total">
                <PriceDisplay amount={toMajor(totalMinor)} />
              </span>
            </div>
            <div className="form-grid">
              <Select
                showLabel
                label="Payment method"
                value={payment}
                onChange={(e) => setPayment(e.target.value)}
              >
                <option>Cash</option>
                <option>Card</option>
                <option>Account</option>
              </Select>
              <Input
                label="Cash received (EGP)"
                type="number"
                min={0}
                disabled={payment !== "Cash"}
                value={cash}
                onChange={(e) => setCash(Math.max(0, Number(e.target.value)))}
              />
            </div>
            <div className="row spread">
              <span className="muted">Change</span>
              <PriceDisplay amount={toMajor(changeMinor)} />
            </div>
            {discountMinor > subtotalMinor && (
              <Alert title="Discount exceeds subtotal" tone="danger">
                Reduce the discount before checkout.
              </Alert>
            )}
            <Button
              variant="primary"
              loading={submitting}
              disabled={
                !cart.length ||
                discountMinor > subtotalMinor ||
                (payment === "Cash" && cashMinor < totalMinor) ||
                (payment === "Account" && customer === "Walk-in customer")
              }
              onClick={() => setDialog("complete")}
            >
              Complete sale
            </Button>
            <div className="row">
              <Button
                disabled={!cart.length}
                onClick={() => {
                  setHeld([...held, cart.map((l) => ({ ...l }))]);
                  setCart([]);
                  setCash(0);
                  setDiscount(0);
                  notify("Basket held for this UI session.");
                }}
              >
                <Pause size={15} />
                Hold sale
              </Button>
              <Button
                variant="ghost"
                disabled={!cart.length}
                onClick={() => setDialog("cancel")}
              >
                Cancel / void
              </Button>
            </div>
          </div>
        </Card>
      </div>
      <Modal
        open={dialog === "held"}
        onOpenChange={() => setDialog("")}
        title="Held sales"
        description="Baskets are available only until this page session is reloaded."
      >
        <div className="stack">
          {held.map((h, i) => (
            <Card key={i}>
              <SectionHeader
                title={`Held basket ${i + 1}`}
                subtitle={`${h.length} product lines`}
                action={
                  <Button
                    disabled={!!cart.length}
                    onClick={() => {
                      setCart(h);
                      setHeld(held.filter((_, n) => n !== i));
                      setDialog("");
                    }}
                  >
                    Resume
                  </Button>
                }
              />
            </Card>
          ))}
          {!!cart.length && (
            <Alert
              title="Hold or clear the current basket first"
              tone="warning"
            >
              Resuming a held sale will not overwrite a current basket.
            </Alert>
          )}
        </div>
      </Modal>
      <Modal
        open={dialog === "complete"}
        onOpenChange={() => setDialog("")}
        title="Review checkout"
        description="Review the sale details before posting to the ledger."
      >
        <div className="stack">
          <DetailList
            items={[
              { label: "Customer", value: customer },
              { label: "Payment", value: payment },
              {
                label: "Total",
                value: <PriceDisplay amount={toMajor(totalMinor)} />,
              },
              {
                label: "Items",
                value: cart.reduce((s, l) => s + l.quantity, 0),
              },
            ]}
          />
          <Button variant="primary" loading={submitting} onClick={complete}>
            Confirm sale
          </Button>
        </div>
      </Modal>
      <ConfirmationModal
        open={dialog === "cancel"}
        onOpenChange={() => setDialog("")}
        title="Void this basket?"
        description="This removes the current in-memory basket. Held baskets remain available."
        onConfirm={() => {
          setCart([]);
          setCash(0);
          setDiscount(0);
          notify("Basket cleared.");
        }}
      />
      <OutputPreview
        title="Receipt preview"
        open={!!receipt}
        onClose={() => setReceipt(null)}
      >
        <DetailList
          items={[
            { label: "Receipt", value: receipt?.receiptNumber },
            { label: "Customer", value: receipt?.customerName },
            {
              label: "Total",
              value: <PriceDisplay amount={toMajor(receipt?.totalMinor ?? 0)} />,
            },
            { label: "Payment", value: receipt?.paymentMethod },
          ]}
        />
      </OutputPreview>
      <Modal
        open={dialog === "history"}
        onOpenChange={() => setDialog("")}
        title="Recent sales"
        description="Sales completed from this branch."
      >
        <div className="stack">
          {!historySales.length ? (
            <EmptyState
              title="No sales found"
              description="No sales have been recorded for this branch yet."
            />
          ) : (
            historySales.map((s) => (
              <Link
                key={s.id}
                to={`/sales/${s.id}`}
                className="text-link"
                onClick={() => setDialog("")}
              >
                #{s.receiptNumber} · {s.customerName} ·{" "}
                {toMajor(s.totalMinor).toFixed(2)} EGP
              </Link>
            ))
          )}
        </div>
      </Modal>
    </div>
  );
}
export function SaleDetails() {
  const { id } = useParams();
  const desktop = isDesktopRuntime();
  const { notify } = useDemo();
  const [print, setPrint] = useState(false);
  const [detail, setDetail] = useState<SaleDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);
  const [voidOpen, setVoidOpen] = useState(false);
  const [voidReason, setVoidReason] = useState("");
  const [voidSubmitting, setVoidSubmitting] = useState(false);
  useEffect(() => {
    if (!desktop || !id) return;
    let active = true;
    getSale(id)
      .then((result) => {
        if (!active) return;
        setDetail(result);
        setError("");
        setLoading(false);
      })
      .catch((err: unknown) => {
        if (!active) return;
        setError(
          err instanceof Error ? err.message : "Sale could not be loaded.",
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
        title="Connect to the desktop app to use the POS."
        description=""
      />
    );
  if (loading)
    return (
      <Card>
        <SectionHeader title="Loading sale…" />
      </Card>
    );
  if (error)
    return (
      <>
        <BackLink to="/sales" label="POS / Sales" />
        <Alert title="Sale could not be loaded" tone="danger">
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
        title="Sale not found"
        description="The sale could not be loaded."
        action={<LinkButton to="/sales">Return to POS</LinkButton>}
      />
    );
  const { sale, items, returns } = detail;
  const handleVoid = () => {
    setVoidSubmitting(true);
    voidSale({ saleId: id!, reason: voidReason })
      .then(() => {
        notify("Sale voided.");
        setAttempt((v) => v + 1);
        setVoidOpen(false);
        setVoidReason("");
        setVoidSubmitting(false);
      })
      .catch((err: unknown) => {
        setVoidSubmitting(false);
        notify(err instanceof Error ? err.message : "Could not void sale.");
      });
  };
  return (
    <>
      <BackLink to="/sales" label="POS / Sales" />
      <div className="feature-page">
        <PageHeader
          title={`Sale #${sale.receiptNumber}`}
          description={`${sale.customerName} · ${sale.paymentMethod}`}
          actions={
            <>
              <Button onClick={() => setPrint(true)}>
                <Printer size={15} />
                Print receipt
              </Button>
              {sale.status === "completed" && (
                <Button variant="danger" onClick={() => setVoidOpen(true)}>
                  Void sale
                </Button>
              )}
            </>
          }
        />
        <Card>
          <DetailList
            items={[
              { label: "Branch", value: detail.branchName },
              { label: "Customer", value: sale.customerName },
              { label: "Payment method", value: sale.paymentMethod },
              { label: "User", value: sale.user || "—" },
              {
                label: "Time",
                value: sale.completedAt
                  ? sale.completedAt.slice(0, 16).replace("T", " ")
                  : "—",
              },
              { label: "Status", value: <Status value={sale.status} /> },
              {
                label: "Total",
                value: <PriceDisplay amount={toMajor(sale.totalMinor)} />,
              },
              {
                label: "Paid",
                value: <PriceDisplay amount={toMajor(sale.paidMinor)} />,
              },
              {
                label: "Change",
                value: <PriceDisplay amount={toMajor(sale.changeMinor)} />,
              },
              {
                label: "Discount",
                value: <PriceDisplay amount={toMajor(sale.discountMinor)} />,
              },
            ]}
          />
        </Card>
        <DataGrid
          label="Sale lines"
          rows={items}
          rowKey={(l) => l.id}
          searchText={(l) => `${l.productName} ${l.packageLabel}`}
          columns={[
            {
              key: "product",
              header: "Product",
              render: (l) => (
                <>
                  {l.productName}
                  <span className="cell-secondary">{l.packageLabel}</span>
                </>
              ),
            },
            { key: "qty", header: "Quantity", render: (l) => l.quantity },
            {
              key: "price",
              header: "Unit price",
              render: (l) => (
                <PriceDisplay amount={toMajor(l.sellingPriceMinor)} />
              ),
            },
            {
              key: "total",
              header: "Line total",
              render: (l) => (
                <PriceDisplay amount={toMajor(l.lineTotalMinor)} />
              ),
            },
          ]}
        />
        {items.some((l) => l.batches.length > 0) && (
          <Card>
            <SectionHeader title="Batch allocations" />
            <Table
              label="Batch allocations"
              rows={items.flatMap((l) =>
                l.batches.map((b) => ({
                  productName: l.productName,
                  packageLabel: l.packageLabel,
                  ...b,
                })),
              )}
              rowKey={(b) => b.batchId}
              columns={[
                {
                  key: "product",
                  header: "Product",
                  render: (b) => (
                    <>
                      {b.productName}
                      <span className="cell-secondary">{b.packageLabel}</span>
                    </>
                  ),
                },
                {
                  key: "batch",
                  header: "Batch",
                  render: (b) => (
                    <>
                      {b.batchNumber}
                      {b.expiryDate && (
                        <span className="cell-secondary">
                          Exp {b.expiryDate}
                        </span>
                      )}
                    </>
                  ),
                },
                { key: "qty", header: "Quantity", render: (b) => b.quantity },
              ]}
            />
          </Card>
        )}
        {returns.length > 0 && (
          <Card>
            <SectionHeader title="Returns" />
            <Table
              label="Returns"
              rows={returns}
              rowKey={(r) => r.id}
              columns={[
                {
                  key: "id",
                  header: "Return",
                  render: (r) => (
                    <>
                      {r.id}
                      <span className="cell-secondary">
                        {r.returnedAt.slice(0, 16).replace("T", " ")}
                      </span>
                    </>
                  ),
                },
                { key: "reason", header: "Reason", render: (r) => r.reason },
                { key: "user", header: "User", render: (r) => r.user || "—" },
                {
                  key: "total",
                  header: "Refund",
                  render: (r) => (
                    <PriceDisplay amount={toMajor(r.totalRefundMinor)} />
                  ),
                },
              ]}
            />
          </Card>
        )}
        <Modal
          open={voidOpen}
          onOpenChange={setVoidOpen}
          title="Void this sale?"
          description="This will mark the sale as void in the ledger. This action cannot be undone."
        >
          <div className="stack">
            <Input
              label="Reason for void"
              value={voidReason}
              onChange={(e) => setVoidReason(e.target.value)}
              placeholder="Enter reason…"
            />
            <div className="row end">
              <Button
                disabled={voidSubmitting}
                onClick={() => setVoidOpen(false)}
              >
                Cancel
              </Button>
              <Button
                variant="danger"
                loading={voidSubmitting}
                disabled={!voidReason.trim()}
                onClick={handleVoid}
              >
                Void sale
              </Button>
            </div>
          </div>
        </Modal>
        <OutputPreview
          title="Receipt preview"
          open={print}
          onClose={() => setPrint(false)}
        >
          <DetailList
            items={[
              { label: "Receipt", value: sale.receiptNumber },
              { label: "Customer", value: sale.customerName },
              {
                label: "Total",
                value: <PriceDisplay amount={toMajor(sale.totalMinor)} />,
              },
              { label: "Payment", value: sale.paymentMethod },
              { label: "Status", value: <Status value={sale.status} /> },
            ]}
          />
        </OutputPreview>
      </div>
    </>
  );
}