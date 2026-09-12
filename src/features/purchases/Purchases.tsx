import { useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { Plus } from "lucide-react";
import {
  FeaturePage,
  LinkButton,
  BackLink,
  DataGrid,
  PriceDisplay,
  Status,
  SummaryStrip,
  Card,
  DetailList,
  Table,
  Button,
  ConfirmationModal,
  EmptyState,
  FormSection,
  Select,
  Input,
  Textarea,
  Alert,
} from "../../design-system";
import { useDemo } from "../../app/DemoContext";
import { purchaseLines } from "../../mock/fixtures";
export function Purchases() {
  const { purchases } = useDemo();
  return (
    <FeaturePage
      title="Purchases"
      description="Plan supplier orders and track receiving progress."
      actions={
        <LinkButton primary to="/purchases/new">
          <Plus size={16} />
          Create purchase
        </LinkButton>
      }
    >
      <SummaryStrip
        items={[
          {
            label: "Open orders",
            value: purchases.filter((p) =>
              ["Ordered", "Partially Received"].includes(p.status),
            ).length,
          },
          {
            label: "Drafts",
            value: purchases.filter((p) => p.status === "Draft").length,
          },
          {
            label: "Outstanding · demo",
            value: (
              <PriceDisplay
                amount={purchases
                  .filter((p) => p.status !== "Cancelled")
                  .reduce((s, p) => s + p.total - p.paid, 0)}
              />
            ),
          },
        ]}
      />
      <DataGrid
        label="Purchases"
        rows={purchases}
        rowKey={(p) => p.id}
        searchText={(p) => `${p.id} ${p.supplier}`}
        filters={[
          {
            label: "Statuses",
            options: [
              "Draft",
              "Ordered",
              "Partially Received",
              "Received",
              "Cancelled",
            ],
            value: (p) => p.status,
          },
        ]}
        columns={[
          {
            key: "id",
            header: "Invoice / supplier",
            render: (p) => (
              <Link className="text-link" to={`/purchases/${p.id}`}>
                {p.id}
                <span className="cell-secondary">{p.supplier}</span>
              </Link>
            ),
          },
          { key: "date", header: "Date", render: (p) => p.date },
          {
            key: "status",
            header: "Status",
            render: (p) => <Status value={p.status} />,
          },
          { key: "items", header: "Items", render: (p) => p.items },
          {
            key: "total",
            header: "Total",
            render: (p) => <PriceDisplay amount={p.total} />,
          },
          {
            key: "paid",
            header: "Paid",
            render: (p) => <PriceDisplay amount={p.paid} />,
          },
          {
            key: "balance",
            header: "Balance",
            render: (p) => <PriceDisplay amount={p.total - p.paid} />,
          },
        ]}
      />
    </FeaturePage>
  );
}
export function PurchaseDetails() {
  const { id } = useParams();
  const { purchases, setPurchases, products, notify } = useDemo();
  const [cancel, setCancel] = useState(false);
  const p = purchases.find((p) => p.id === id);
  if (!p)
    return (
      <EmptyState
        title="Purchase not found"
        description="This order is not in the current UI session."
        action={<LinkButton to="/purchases">Purchases</LinkButton>}
      />
    );
  return (
    <>
      <BackLink to="/purchases" label="Purchases" />
      <FeaturePage
        title={p.id}
        description={`${p.supplier} · supplier purchase order`}
        actions={
          <>
            <Status value={p.status} />
            {["Ordered", "Partially Received"].includes(p.status) && (
              <LinkButton primary to={`/purchases/${p.id}/receive`}>
                Receive purchase
              </LinkButton>
            )}
            {p.status === "Draft" && (
              <Button
                variant="primary"
                onClick={() => {
                  setPurchases(
                    purchases.map((v) =>
                      v.id === p.id ? { ...v, status: "Ordered" } : v,
                    ),
                  );
                  notify(
                    "Demo order marked ordered. Nothing was sent to the supplier.",
                  );
                }}
              >
                Mark ordered
              </Button>
            )}
            <Button
              disabled={["Cancelled", "Received"].includes(p.status)}
              onClick={() => setCancel(true)}
            >
              Cancel order
            </Button>
          </>
        }
      >
        <div className="content-stack">
          <Card>
            <DetailList
              items={[
                { label: "Supplier", value: p.supplier },
                { label: "Order date", value: p.date },
                { label: "Expected delivery", value: p.delivery },
                { label: "Supplier invoice", value: `INV-${p.id.slice(3)}` },
                { label: "Paid", value: <PriceDisplay amount={p.paid} /> },
                {
                  label: "Balance",
                  value: <PriceDisplay amount={p.total - p.paid} />,
                },
              ]}
            />
          </Card>
          <Card>
            <Table
              label="Purchase order lines"
              rows={p.lines ?? purchaseLines}
              rowKey={(l) => l.productId}
              columns={[
                {
                  key: "name",
                  header: "Product",
                  render: (l) =>
                    products.find((v) => v.id === l.productId)?.name,
                },
                {
                  key: "pack",
                  header: "Package",
                  render: (l) =>
                    products.find((v) => v.id === l.productId)?.pack,
                },
                { key: "qty", header: "Ordered", render: (l) => l.ordered },
                {
                  key: "received",
                  header: "Received",
                  render: (l) =>
                    p.status === "Received" ? l.ordered : l.received,
                },
                {
                  key: "cost",
                  header: "Cost",
                  render: (l) => <PriceDisplay amount={l.cost} />,
                },
              ]}
            />
            <p className="muted">
              UI-only order lines. No supplier or stock ledger is connected.
            </p>
          </Card>
        </div>
        <ConfirmationModal
          open={cancel}
          onOpenChange={setCancel}
          title="Cancel this purchase?"
          description="Only the demo order status changes. No supplier communication or financial posting occurs."
          onConfirm={() => {
            setPurchases(
              purchases.map((v) =>
                v.id === p.id ? { ...v, status: "Cancelled" } : v,
              ),
            );
            notify("Demo order cancelled.");
          }}
        />
      </FeaturePage>
    </>
  );
}
export function PurchaseForm() {
  const { products, suppliers, purchases, setPurchases, notify } = useDemo();
  const navigate = useNavigate();
  const [lines, setLines] = useState([
    { productId: products[0].id, quantity: 1, cost: products[0].cost },
  ]);
  return (
    <>
      <BackLink to="/purchases" label="Purchases" />
      <FeaturePage
        title="Create purchase"
        description="Prepare a supplier order. This draft is held only in memory."
      >
        <form
          className="content-stack"
          onSubmit={(e) => {
            e.preventDefault();
            const f = new FormData(e.currentTarget);
            const id = `PO-${Date.now().toString().slice(-5)}`;
            const subtotal = lines.reduce((s, l) => s + l.quantity * l.cost, 0);
            setPurchases([
              {
                id,
                supplier: String(f.get("supplier")),
                date: "2026-09-11",
                delivery: String(f.get("delivery")),
                status: "Draft",
                items: lines.length,
                total: Math.max(
                  0,
                  subtotal - Number(f.get("discount")) + Number(f.get("tax")),
                ),
                paid: 0,
                lines: lines.map((l) => ({
                  productId: l.productId,
                  ordered: l.quantity,
                  received: 0,
                  cost: l.cost,
                })),
                notes: String(f.get("notes")),
                discount: Number(f.get("discount")),
                tax: Number(f.get("tax")),
              },
              ...purchases,
            ]);
            notify("Purchase draft created in this preview.");
            navigate(`/purchases/${id}`);
          }}
        >
          <FormSection title="Supplier & delivery">
            <Select showLabel name="supplier" label="Purchase supplier">
              {suppliers.map((s) => (
                <option key={s.id}>{s.name}</option>
              ))}
            </Select>
            <Input
              name="delivery"
              label="Expected delivery"
              type="date"
              defaultValue="2026-09-13"
              required
            />
          </FormSection>
          <Card>
            <div className="row spread">
              <h3>Order lines</h3>
              <Button
                onClick={() =>
                  setLines([
                    ...lines,
                    {
                      productId: products[0].id,
                      quantity: 1,
                      cost: products[0].cost,
                    },
                  ])
                }
              >
                Add line
              </Button>
            </div>
            <Table
              label="New purchase lines"
              rows={lines.map((l, i) => ({ ...l, index: i }))}
              rowKey={(l) => String(l.index)}
              columns={[
                {
                  key: "product",
                  header: "Product / package",
                  render: (l) => (
                    <Select
                      label={`Purchase product ${l.index + 1}`}
                      value={l.productId}
                      onChange={(e) =>
                        setLines(
                          lines.map((v, i) =>
                            i === l.index
                              ? {
                                  ...v,
                                  productId: e.target.value,
                                  cost:
                                    products.find(
                                      (p) => p.id === e.target.value,
                                    )?.cost ?? 0,
                                }
                              : v,
                          ),
                        )
                      }
                    >
                      {products.map((p) => (
                        <option key={p.id} value={p.id}>
                          {p.name} · {p.pack}
                        </option>
                      ))}
                    </Select>
                  ),
                },
                {
                  key: "quantity",
                  header: "Quantity",
                  render: (l) => (
                    <Input
                      hideLabel
                      label={`Order quantity ${l.index + 1}`}
                      type="number"
                      min={1}
                      required
                      value={l.quantity}
                      onChange={(e) =>
                        setLines(
                          lines.map((v, i) =>
                            i === l.index
                              ? { ...v, quantity: Number(e.target.value) }
                              : v,
                          ),
                        )
                      }
                    />
                  ),
                },
                {
                  key: "cost",
                  header: "Cost",
                  render: (l) => (
                    <Input
                      hideLabel
                      label={`Order cost ${l.index + 1}`}
                      type="number"
                      min={0}
                      step="0.01"
                      required
                      value={l.cost}
                      onChange={(e) =>
                        setLines(
                          lines.map((v, i) =>
                            i === l.index
                              ? { ...v, cost: Number(e.target.value) }
                              : v,
                          ),
                        )
                      }
                    />
                  ),
                },
                {
                  key: "remove",
                  header: "Action",
                  render: (l) => (
                    <Button
                      disabled={lines.length === 1}
                      onClick={() =>
                        setLines(lines.filter((_, i) => i !== l.index))
                      }
                    >
                      Remove
                    </Button>
                  ),
                },
              ]}
            />
          </Card>
          <FormSection title="Order totals & notes">
            <Input
              name="discount"
              label="Discount (EGP)"
              type="number"
              min={0}
              defaultValue={0}
            />
            <Input
              name="tax"
              label="Tax (EGP) · demo field"
              type="number"
              min={0}
              defaultValue={0}
            />
            <Textarea name="notes" label="Supplier notes" />
          </FormSection>
          <div className="form-actions">
            <LinkButton to="/purchases">Cancel</LinkButton>
            <Button type="submit" variant="primary">
              Save demo draft
            </Button>
          </div>
        </form>
      </FeaturePage>
    </>
  );
}
export function ReceivePurchase() {
  const { id } = useParams();
  const { purchases, setPurchases, products, notify } = useDemo();
  const p = purchases.find((p) => p.id === id);
  const [rows, setRows] = useState(
      (p?.lines ?? purchaseLines).map((l) => ({
        ...l,
        received: l.ordered,
        damaged: 0,
        batch: l.batch ?? "",
        expiry: l.expiry ?? "2027-09-01",
      })),
    ),
    [error, setError] = useState(""),
    [confirm, setConfirm] = useState(false);
  const navigate = useNavigate();
  if (!p)
    return (
      <EmptyState
        title="Purchase not found"
        description="Choose an order from Purchases."
      />
    );
  return (
    <>
      <BackLink to={`/purchases/${id}`} label={p.id} />
      <FeaturePage
        title="Receive purchase"
        description={`${p.id} · ${p.supplier}`}
      >
        <div className="workflow-steps">
          <span>1. Order</span>
          <strong>2. Receive & inspect</strong>
          <span>3. Review</span>
        </div>
        {!["Ordered", "Partially Received"].includes(p.status) ? (
          <Alert title="This order cannot be received" tone="warning">
            Only ordered or partially received demo purchases can enter
            receiving.
          </Alert>
        ) : (
          <form
            className="content-stack"
            onSubmit={(e) => {
              e.preventDefault();
              if (
                rows.some(
                  (r) => r.received > r.ordered || r.damaged > r.received,
                )
              ) {
                setError(
                  "Received quantity cannot exceed ordered quantity; damaged quantity cannot exceed received quantity.",
                );
                return;
              }
              setError("");
              setConfirm(true);
            }}
          >
            {error && (
              <Alert title="Review receiving quantities" tone="danger">
                {error}
              </Alert>
            )}
            <Card>
              <Table
                label="Receiving lines"
                rows={rows}
                rowKey={(r) => r.productId}
                columns={[
                  {
                    key: "name",
                    header: "Product / ordered",
                    render: (r) => (
                      <>
                        {products.find((p) => p.id === r.productId)?.name}
                        <span className="cell-secondary">
                          Ordered: {r.ordered}
                        </span>
                      </>
                    ),
                  },
                  {
                    key: "qty",
                    header: "Received",
                    render: (r) => (
                      <Input
                        hideLabel
                        label={`Received ${r.productId}`}
                        type="number"
                        min={0}
                        max={r.ordered}
                        required
                        value={r.received}
                        onChange={(e) =>
                          setRows(
                            rows.map((v) =>
                              v.productId === r.productId
                                ? { ...v, received: Number(e.target.value) }
                                : v,
                            ),
                          )
                        }
                      />
                    ),
                  },
                  {
                    key: "damage",
                    header: "Damaged",
                    render: (r) => (
                      <Input
                        hideLabel
                        label={`Damaged ${r.productId}`}
                        type="number"
                        min={0}
                        max={r.received}
                        required
                        value={r.damaged}
                        onChange={(e) =>
                          setRows(
                            rows.map((v) =>
                              v.productId === r.productId
                                ? { ...v, damaged: Number(e.target.value) }
                                : v,
                            ),
                          )
                        }
                      />
                    ),
                  },
                  {
                    key: "batch",
                    header: "Batch / lot",
                    render: (r) => (
                      <Input
                        hideLabel
                        label={`Batch ${r.productId}`}
                        required
                        value={r.batch}
                        onChange={(e) =>
                          setRows(
                            rows.map((v) =>
                              v.productId === r.productId
                                ? { ...v, batch: e.target.value }
                                : v,
                            ),
                          )
                        }
                      />
                    ),
                  },
                  {
                    key: "expiry",
                    header: "Expiry date",
                    render: (r) => (
                      <Input
                        hideLabel
                        label={`Expiry ${r.productId}`}
                        type="date"
                        required
                        value={r.expiry}
                        onChange={(e) =>
                          setRows(
                            rows.map((v) =>
                              v.productId === r.productId
                                ? { ...v, expiry: e.target.value }
                                : v,
                            ),
                          )
                        }
                      />
                    ),
                  },
                  {
                    key: "cost",
                    header: "Unit cost",
                    render: (r) => <PriceDisplay amount={r.cost} />,
                  },
                ]}
              />
            </Card>
            <FormSection title="Receiving notes">
              <Textarea label="Delivery / inspection notes" />
            </FormSection>
            <div className="form-actions">
              <Button
                type="submit"
                variant="primary"
                disabled={rows.every((r) => r.received === 0)}
              >
                Review receiving
              </Button>
            </div>
          </form>
        )}
        <ConfirmationModal
          open={confirm}
          onOpenChange={setConfirm}
          title="Confirm demo receiving?"
          description="Order status will change in memory. Batch, stock, supplier balances and accounting are not posted."
          onConfirm={() => {
            setPurchases(
              purchases.map((v) =>
                v.id === id
                  ? {
                      ...v,
                      lines: rows,
                      status: rows.every(
                        (r) => r.received - r.damaged === r.ordered,
                      )
                        ? "Received"
                        : "Partially Received",
                    }
                  : v,
              ),
            );
            notify("Demo receiving reviewed. No inventory was posted.");
            navigate(`/purchases/${id}`);
          }}
        />
      </FeaturePage>
    </>
  );
}
