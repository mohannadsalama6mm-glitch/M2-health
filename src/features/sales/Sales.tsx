import { useRef, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { ShoppingCart, Plus, Pause, Play, Trash2, Printer } from "lucide-react";
import {
  FeaturePage,
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
} from "../../design-system";
import { useDemo } from "../../app/DemoContext";
import type { Product, Sale } from "../../mock/types";
import "./sales.css";
export function Sales() {
  const {
    products,
    cart,
    setCart,
    held,
    setHeld,
    customers,
    sales,
    setSales,
    notify,
  } = useDemo();
  const [query, setQuery] = useState(""),
    [barcode, setBarcode] = useState(""),
    [customer, setCustomer] = useState("Walk-in customer"),
    [payment, setPayment] = useState("Cash"),
    [cash, setCash] = useState(0),
    [discount, setDiscount] = useState(0),
    [dialog, setDialog] = useState(""),
    [receipt, setReceipt] = useState<Sale | null>(null),
    [searchError, setSearchError] = useState("");
  const searchRef = useRef<HTMLDivElement>(null);
  const lines = cart.flatMap((l) => {
    const p = products.find((p) => p.id === l.productId);
    return p ? [{ ...l, product: p }] : [];
  });
  const subtotal = lines.reduce(
    (sum, l) => sum + l.product.price * l.quantity,
    0,
  );
  const total = Math.max(0, subtotal - discount);
  const results = products.filter(
    (p) =>
      p.status === "Active" &&
      `${p.name} ${p.scientific} ${(p.barcodes ?? [p.barcode]).join(" ")}`
        .toLowerCase()
        .includes(query.toLowerCase()),
  );
  const add = (p: Product) => {
    if (p.stock === 0) {
      notify("This demo product is out of stock.");
      return;
    }
    setCart((current) =>
      current.some((l) => l.productId === p.id)
        ? current.map((l) =>
            l.productId === p.id ? { ...l, quantity: l.quantity + 1 } : l,
          )
        : [...current, { productId: p.id, quantity: 1 }],
    );
    setSearchError("");
  };
  const complete = () => {
    const next: Sale = {
      id: `DEMO-${Date.now().toString().slice(-6)}`,
      time: "Just now (demo)",
      customer,
      items: cart.reduce((s, l) => s + l.quantity, 0),
      total,
      payment,
      status: "Completed",
      lines: cart.map((l) => ({ ...l })),
    };
    setSales([next, ...sales]);
    setReceipt(next);
    setCart([]);
    setCash(0);
    setDiscount(0);
    setDialog("");
    notify(
      "Demo sale completed in memory. No stock or accounting records were posted.",
    );
  };
  return (
    <FeaturePage
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
    >
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
                placeholder="Search name, ingredient, or barcode…"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && results[0]) {
                    add(results[0]);
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
                  if (e.key === "Enter") {
                    const p = products.find(
                      (p) =>
                        (p.barcodes ?? [p.barcode]).includes(barcode) &&
                        p.status === "Active",
                    );
                    if (p) {
                      add(p);
                      setBarcode("");
                    } else
                      setSearchError(
                        "No active demo product matches this barcode.",
                      );
                  }
                }}
              />
              <Button
                onClick={() => {
                  const p = products.find(
                    (p) =>
                      (p.barcodes ?? [p.barcode]).includes(barcode) &&
                      p.status === "Active",
                  );
                  if (p) {
                    add(p);
                    setBarcode("");
                  } else
                    setSearchError(
                      "No active demo product matches this barcode.",
                    );
                }}
              >
                Add barcode
              </Button>
            </div>
            {searchError && (
              <Alert title="Product not found" tone="warning">
                {searchError}
              </Alert>
            )}
            <p className="muted">
              Keyboard-wedge scanner input is prepared. No hardware connection
              is active.
            </p>
          </Card>
          <Card>
            <SectionHeader
              title="Available products"
              action={<span className="muted">{results.length} matches</span>}
            />
            <Table
              label="POS products"
              rows={results}
              rowKey={(p) => p.id}
              columns={[
                {
                  key: "name",
                  header: "Product",
                  render: (p) => (
                    <>
                      {p.name}
                      <span className="cell-secondary">{p.pack}</span>
                    </>
                  ),
                },
                { key: "stock", header: "Available", render: (p) => p.stock },
                {
                  key: "price",
                  header: "Price",
                  render: (p) => <PriceDisplay amount={p.price} />,
                },
                {
                  key: "add",
                  header: "Add",
                  render: (p) => (
                    <Button
                      size="sm"
                      disabled={!p.stock}
                      aria-label={`Add ${p.name} to cart`}
                      onClick={() => add(p)}
                    >
                      <Plus size={15} />
                    </Button>
                  ),
                },
              ]}
            />
          </Card>
        </div>
        <Card className="pos-cart">
          <SectionHeader
            title="Current sale"
            action={<Badge>{cart.length} lines</Badge>}
          />
          <Select
            showLabel
            label="Sale customer"
            value={customer}
            onChange={(e) => setCustomer(e.target.value)}
          >
            <option>Walk-in customer</option>
            {customers.map((c) => (
              <option key={c.id}>{c.name}</option>
            ))}
          </Select>
          <div className="cart-lines">
            {!lines.length ? (
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
              lines.map((l) => (
                <div className="cart-line" key={l.productId}>
                  <div className="row spread">
                    <strong>{l.product.name}</strong>
                    <Button
                      size="sm"
                      aria-label={`Remove ${l.product.name}`}
                      onClick={() =>
                        setCart(cart.filter((c) => c.productId !== l.productId))
                      }
                    >
                      <Trash2 size={14} />
                    </Button>
                  </div>
                  <p className="muted">
                    {l.product.pack} · <PriceDisplay amount={l.product.price} />
                  </p>
                  <div className="row spread">
                    <QuantityStepper
                      min={1}
                      value={l.quantity}
                      onChange={(quantity) =>
                        setCart(
                          cart.map((c) =>
                            c.productId === l.productId
                              ? { ...c, quantity }
                              : c,
                          ),
                        )
                      }
                    />
                    <PriceDisplay amount={l.quantity * l.product.price} />
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
                  value: <PriceDisplay amount={subtotal} />,
                },
                { label: "Tax", value: "0.00 EGP · demo only" },
              ]}
            />
            <Input
              type="number"
              min={0}
              max={subtotal}
              label="Sale discount (EGP)"
              value={discount}
              onChange={(e) => setDiscount(Math.max(0, Number(e.target.value)))}
            />
            <div className="row spread checkout-total">
              <strong>Grand total</strong>
              <span className="receipt-total">
                <PriceDisplay amount={total} />
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
              <PriceDisplay
                amount={payment === "Cash" ? Math.max(0, cash - total) : 0}
              />
            </div>
            {discount > subtotal && (
              <Alert title="Discount exceeds subtotal" tone="danger">
                Reduce the discount before checkout.
              </Alert>
            )}
            <Button
              variant="primary"
              disabled={
                !lines.length ||
                discount > subtotal ||
                (payment === "Cash" && cash < total) ||
                (payment === "Account" && customer === "Walk-in customer")
              }
              onClick={() => setDialog("complete")}
            >
              Complete sale · demo
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
        description="UI demonstration only. No payment is charged and no inventory is changed."
      >
        <div className="stack">
          <DetailList
            items={[
              { label: "Customer", value: customer },
              { label: "Payment", value: payment },
              { label: "Total", value: <PriceDisplay amount={total} /> },
              {
                label: "Items",
                value: cart.reduce((s, l) => s + l.quantity, 0),
              },
            ]}
          />
          <Button variant="primary" onClick={complete}>
            Confirm demo sale
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
          notify("Demo basket cleared.");
        }}
      />
      <OutputPreview
        title="Receipt preview"
        open={!!receipt}
        onClose={() => setReceipt(null)}
      >
        <DetailList
          items={[
            { label: "Invoice", value: receipt?.id },
            { label: "Customer", value: receipt?.customer },
            {
              label: "Total",
              value: <PriceDisplay amount={receipt?.total ?? 0} />,
            },
            { label: "Payment", value: receipt?.payment },
          ]}
        />
        <p className="muted">No fiscal receipt was issued.</p>
      </OutputPreview>
      <Modal
        open={dialog === "history"}
        onOpenChange={() => setDialog("")}
        title="Recent sales"
        description="Fictional sales and transactions completed in this UI session."
      >
        <div className="stack">
          {sales.map((s) => (
            <Link
              key={s.id}
              to={`/sales/${s.id}`}
              className="text-link"
              onClick={() => setDialog("")}
            >
              #{s.id} · {s.customer} · {s.total.toFixed(2)} EGP
            </Link>
          ))}
        </div>
      </Modal>
    </FeaturePage>
  );
}
export function SaleDetails() {
  const { id } = useParams();
  const { sales, products } = useDemo();
  const [print, setPrint] = useState(false);
  const sale = sales.find((s) => s.id === id);
  if (!sale)
    return (
      <EmptyState
        title="Sale not found"
        description="The sale may have been cleared when the UI was reloaded."
        action={<LinkButton to="/sales">Return to POS</LinkButton>}
      />
    );
  return (
    <>
      <BackLink to="/sales" label="POS / Sales" />
      <FeaturePage
        title={`Sale #${sale.id}`}
        description="Demo transaction details · no fiscal record"
        actions={
          <Button onClick={() => setPrint(true)}>
            <Printer size={15} />
            Print receipt
          </Button>
        }
      >
        <Card>
          <DetailList
            items={[
              { label: "Customer", value: sale.customer },
              { label: "Payment method", value: sale.payment },
              { label: "Time", value: sale.time },
              { label: "Status", value: <Status value={sale.status} /> },
              { label: "Total", value: <PriceDisplay amount={sale.total} /> },
            ]}
          />
        </Card>
        <DataGrid
          label="Sale lines"
          rows={sale.lines}
          rowKey={(l) => l.productId}
          searchText={(l) =>
            products.find((p) => p.id === l.productId)?.name ?? ""
          }
          columns={[
            {
              key: "product",
              header: "Product",
              render: (l) => products.find((p) => p.id === l.productId)?.name,
            },
            { key: "qty", header: "Quantity", render: (l) => l.quantity },
            {
              key: "price",
              header: "Current catalog unit price",
              render: (l) => (
                <PriceDisplay
                  amount={
                    products.find((p) => p.id === l.productId)?.price ?? 0
                  }
                />
              ),
            },
          ]}
        />
        <OutputPreview
          title="Receipt preview"
          open={print}
          onClose={() => setPrint(false)}
        >
          <DetailList
            items={[
              { label: "Invoice", value: sale.id },
              { label: "Total", value: <PriceDisplay amount={sale.total} /> },
            ]}
          />
        </OutputPreview>
      </FeaturePage>
    </>
  );
}
