import { useEffect, useRef, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { useBranch } from "../../app/BranchContext";
import { phase6, type SaleFinanceTrace } from "../../lib/tauri/phase6";
import {
  Alert,
  Button,
  Card,
  Input,
  Select,
  Table,
  Modal,
  PageHeader,
  LinkButton,
} from "../../design-system";
import * as api from "../../lib/tauri/canonical-sales";
import { canonicalPartners } from "../../lib/tauri/canonical-partners";
import type { Customer } from "../../lib/tauri/canonical-partners";
import type {
  Cart,
  Product,
  Sale,
  Preview,
  Method,
  PostInput,
  ReturnInput,
} from "../../lib/tauri/canonical-sales";
import "./sales.css";
const errorText = (e: unknown) =>
  e instanceof Error ? e.message : "Sales operation failed.";
const money = (v: number) => `${(v / 100).toFixed(2)} EGP`;
function minor(s: string) {
  if (!/^\d+(\.\d{0,2})?$/.test(s))
    throw new Error("Enter a non-negative amount with at most two decimals.");
  const [whole, fraction = ""] = s.split(".");
  const n = Number(whole) * 100 + Number(fraction.padEnd(2, "0"));
  if (!Number.isSafeInteger(n))
    throw new Error("Amount exceeds the safe range.");
  return n;
}
const empty = (branchId: string): Cart => ({
  branchId,
  customerId: null,
  customerName: null,
  notes: null,
  discountMinor: 0,
  taxMinor: 0,
  lines: [],
});
function Navigation() {
  return (
    <nav className="row wrap" aria-label="Sales workflows">
      <LinkButton to="/sales">New Sale</LinkButton>
      <LinkButton to="/sales/history">Sales History / Held</LinkButton>
      <LinkButton to="/inventory">Inventory</LinkButton>
    </nav>
  );
}
export function CanonicalSales() {
  const { pathname } = useLocation();
  const { branchId, loading } = useBranch();
  if (loading) return <p role="status">Loading sales branch…</p>;
  if (pathname === "/sales/history") return <History key={branchId} />;
  const id = pathname.split("/")[2];
  if (id) return <Detail key={id} id={id} />;
  return <POS key={branchId} />;
}
function POS() {
  const { branchId, branchName } = useBranch();
  const navigate = useNavigate();
  const [cart, setCart] = useState<Cart>(() => empty(branchId));
  const [customerSearch, setCustomerSearch] = useState("");
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [customerError, setCustomerError] = useState("");
  const [sale, setSale] = useState<Sale | null>(null);
  const [names, setNames] = useState<Record<string, string>>({});
  const [search, setSearch] = useState(""),
    [page, setPage] = useState(0),
    [results, setResults] = useState<{
      key: string;
      items: Product[];
      total: number;
    } | null>(null),
    [searchError, setSearchError] = useState("");
  const queryKey = JSON.stringify([branchId, search, page]);
  const [totals, setTotals] = useState<{ key: string; value: Preview } | null>(
      null,
    ),
    [previewError, setPreviewError] = useState("");
  const cartKey = JSON.stringify(cart);
  const [error, setError] = useState(""),
    [message, setMessage] = useState(""),
    [busy, setBusy] = useState(false),
    [payment, setPayment] = useState(false),
    [method, setMethod] = useState<Method>("CASH"),
    [amount, setAmount] = useState(""),
    [reference, setReference] = useState("");
  const lock = useRef(false),
    draftId = useRef<string>(crypto.randomUUID()),
    pending = useRef<PostInput | null>(null);
  const [pendingPost, setPendingPost] = useState(false);
  useEffect(() => {
    let active = true;
    const timer = setTimeout(() => {
      canonicalPartners
        .listCustomers({
          search: customerSearch || null,
          isActive: true,
          limit: 8,
          offset: 0,
        })
        .then((result) => {
          if (active) {
            setCustomers(result.items);
            setCustomerError("");
          }
        })
        .catch((e) => {
          if (active) setCustomerError(errorText(e));
        });
    }, 150);
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [customerSearch]);
  useEffect(() => {
    let active = true;
    const t = setTimeout(
      () =>
        api
          .searchProducts({ branchId, search, limit: 20, offset: page * 20 })
          .then((r) => {
            if (active) {
              setResults({ key: queryKey, ...r });
              setSearchError("");
            }
          })
          .catch((e) => {
            if (active) {
              setSearchError(errorText(e));
              setResults(null);
            }
          }),
      150,
    );
    return () => {
      active = false;
      clearTimeout(t);
    };
  }, [branchId, search, page, queryKey]);
  useEffect(() => {
    let active = true;
    const t = setTimeout(() => {
      if (!cart.lines.length) {
        setTotals(null);
        setPreviewError("");
        return;
      }
      api
        .preview(cart)
        .then((r) => {
          if (active) {
            setTotals({ key: cartKey, value: r });
            setPreviewError("");
          }
        })
        .catch((e) => {
          if (active) {
            setTotals(null);
            setPreviewError(errorText(e));
          }
        });
    }, 180);
    return () => {
      active = false;
      clearTimeout(t);
    };
  }, [cart, cartKey]);
  const current = results?.key === queryKey ? results : null;
  const total = totals?.key === cartKey ? totals.value : null;
  function change(next: Cart) {
    if (pending.current) return;
    setCart(next);
    setMessage("");
    setError("");
  }
  function moneyChange(text: string, apply: (value: number) => void) {
    try {
      apply(minor(text || "0"));
    } catch (e) {
      setError(errorText(e));
    }
  }
  function add(p: Product) {
    if (p.currentPriceMinor === null || pending.current) return;
    setNames((n) => ({
      ...n,
      [p.productPackageId]: `${p.productName} · ${p.packageLabel}`,
    }));
    const existing = cart.lines.find(
      (l) => l.productPackageId === p.productPackageId,
    );
    change({
      ...cart,
      lines: existing
        ? cart.lines.map((l) =>
            l === existing ? { ...l, quantity: l.quantity + 1 } : l,
          )
        : [
            ...cart.lines,
            {
              productPackageId: p.productPackageId,
              quantity: 1,
              unitPriceMinor: p.currentPriceMinor,
              discountMinor: 0,
            },
          ],
    });
  }
  async function persist() {
    const s = await api.saveDraft({
      id: sale?.id ?? draftId.current,
      expectedRevision: sale?.revision ?? null,
      cart,
    });
    setSale(s);
    setCart(s.cart);
    return s;
  }
  async function action(fn: () => Promise<void>) {
    if (lock.current) return;
    lock.current = true;
    setBusy(true);
    setError("");
    try {
      await fn();
    } catch (e) {
      setError(errorText(e));
    } finally {
      lock.current = false;
      setBusy(false);
    }
  }
  async function post() {
    await action(async () => {
      let p = pending.current;
      if (!p) {
        const s = await persist();
        p = {
          id: s.id,
          expectedRevision: s.revision,
          requestId: crypto.randomUUID(),
          payments: [
            {
              paymentMethod: method,
              amountMinor: minor(amount),
              reference: reference || null,
            },
          ],
        };
        pending.current = p;
        setPendingPost(true);
      }
      const posted = await api.post(p);
      navigate(`/sales/${posted.id}`);
    });
  }
  useEffect(() => {
    const id = new URLSearchParams(window.location.search).get("resume");
    if (!id) return;
    let active = true;
    api
      .get(id)
      .then((s) => {
        if (!active) return;
        if (s.branchId !== branchId)
          throw new Error("Select the held sale branch before resuming.");
        if (!["DRAFT", "HELD"].includes(s.status))
          throw new Error(
            "Sale is already posted or voided. Open its history.",
          );
        return s.status === "HELD"
          ? api.resume({ id: s.id, expectedRevision: s.revision })
          : s;
      })
      .then((s) => {
        if (active && s) {
          setSale(s);
          setCart(s.cart);
          draftId.current = s.id;
        }
      })
      .catch((e) => {
        if (active) setError(errorText(e));
      });
    return () => {
      active = false;
    };
  }, [branchId]);
  return (
    <div className="stack">
      <PageHeader
        title="Point of Sale"
        description={`${branchName} · Cart and held sales do not reserve stock.`}
      />
      <Navigation />
      <Button
        disabled={busy || pendingPost}
        onClick={() => {
          setCart(empty(branchId));
          setSale(null);
          draftId.current = crypto.randomUUID();
          setNames({});
          setMessage("");
          navigate("/sales");
        }}
      >
        New Cart
      </Button>
      {error && (
        <Alert title="Sale action failed" tone="danger">
          {error}
        </Alert>
      )}
      {message && (
        <Alert title="Saved" tone="success">
          {message}
        </Alert>
      )}
      <div className="pos-layout canonical-pos">
        <Card>
          <Input
            label="POS search / barcode"
            value={search}
            onChange={(e) => {
              setSearch(e.target.value);
              setPage(0);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter" && current?.items[0]) {
                e.preventDefault();
                add(current.items[0]);
              }
            }}
            autoFocus
            placeholder="Scan barcode, English, Arabic or scientific name"
          />
          {searchError ? (
            <Alert title="Product search failed" tone="danger">
              {searchError}
            </Alert>
          ) : !current ? (
            <p role="status">Searching SQLite Catalog…</p>
          ) : (
            <>
              <Table
                label="POS Catalog results"
                rows={current.items}
                rowKey={(p) => p.productPackageId}
                columns={[
                  {
                    key: "name",
                    header: "Product / package",
                    render: (p) => (
                      <>
                        {p.productName}
                        <small className="muted">
                          {" "}
                          {p.packageLabel} {p.arabicName}
                        </small>
                      </>
                    ),
                  },
                  {
                    key: "stock",
                    header: "Eligible stock",
                    render: (p) => `${p.availableQuantity} · ${p.stockStatus}`,
                  },
                  {
                    key: "price",
                    header: "Price",
                    render: (p) =>
                      p.currentPriceMinor === null
                        ? "No price"
                        : money(p.currentPriceMinor),
                  },
                  {
                    key: "add",
                    header: "Cart",
                    render: (p) => (
                      <Button
                        disabled={
                          busy || pendingPost || p.currentPriceMinor === null
                        }
                        onClick={() => add(p)}
                      >
                        Add
                      </Button>
                    ),
                  },
                ]}
              />
              {!current.total && <p>No matching Catalog packages.</p>}
              <div className="row">
                <Button disabled={page === 0} onClick={() => setPage(page - 1)}>
                  Previous products
                </Button>
                <span>{current.total} packages</span>
                <Button
                  disabled={(page + 1) * 20 >= current.total}
                  onClick={() => setPage(page + 1)}
                >
                  Next products
                </Button>
              </div>
            </>
          )}
        </Card>
        <Card>
          <h2>Cart {sale?.status === "HELD" ? "· Held" : ""}</h2>
          <fieldset disabled={busy || pendingPost}>
            <Table
              label="Sale cart"
              rows={cart.lines}
              rowKey={(l) => l.productPackageId}
              columns={[
                {
                  key: "name",
                  header: "Package",
                  render: (l) =>
                    names[l.productPackageId] ||
                    l.displayName ||
                    l.productPackageId,
                },
                {
                  key: "qty",
                  header: "Qty",
                  render: (l) => (
                    <Input
                      hideLabel
                      label={`Quantity ${l.productPackageId}`}
                      type="number"
                      min="1"
                      step="1"
                      value={l.quantity}
                      onChange={(e) =>
                        change({
                          ...cart,
                          lines: cart.lines.map((x) =>
                            x === l
                              ? { ...x, quantity: Number(e.target.value) }
                              : x,
                          ),
                        })
                      }
                      onKeyDown={(e) => {
                        if (e.key === "Delete") {
                          e.preventDefault();
                          change({
                            ...cart,
                            lines: cart.lines.filter((x) => x !== l),
                          });
                        }
                      }}
                    />
                  ),
                },
                {
                  key: "price",
                  header: "Price",
                  render: (l) => money(l.unitPriceMinor ?? 0),
                },
                {
                  key: "discount",
                  header: "Discount (EGP)",
                  render: (l) => (
                    <Input
                      hideLabel
                      label={`Line discount ${l.productPackageId}`}
                      type="number"
                      min="0"
                      step="0.01"
                      value={l.discountMinor / 100}
                      onChange={(e) =>
                        moneyChange(e.target.value, (value) =>
                          change({
                            ...cart,
                            lines: cart.lines.map((x) =>
                              x === l ? { ...x, discountMinor: value } : x,
                            ),
                          }),
                        )
                      }
                    />
                  ),
                },
                {
                  key: "remove",
                  header: "",
                  render: (l) => (
                    <Button
                      onClick={() =>
                        change({
                          ...cart,
                          lines: cart.lines.filter((x) => x !== l),
                        })
                      }
                    >
                      Remove
                    </Button>
                  ),
                },
              ]}
            />
            {!cart.lines.length && <p>Add a package to begin.</p>}
            <Input
              label="Sale discount (EGP)"
              type="number"
              min="0"
              step="0.01"
              value={cart.discountMinor / 100}
              onChange={(e) =>
                moneyChange(e.target.value, (value) =>
                  change({ ...cart, discountMinor: value }),
                )
              }
            />
            <Input
              label="Tax amount (EGP, optional)"
              type="number"
              min="0"
              step="0.01"
              value={cart.taxMinor / 100}
              onChange={(e) =>
                moneyChange(e.target.value, (value) =>
                  change({ ...cart, taxMinor: value }),
                )
              }
            />
            <div className="customer-picker">
              <Input
                label="Customer search (optional)"
                value={customerSearch}
                onChange={(e) => setCustomerSearch(e.target.value)}
                placeholder="Name, code, phone or email"
              />
              {cart.customerId ? (
                <div className="selected-customer" aria-label="Selected customer">
                  <strong>{cart.customerName}</strong>
                  <Button
                    size="sm"
                    onClick={() => {
                      change({ ...cart, customerId: null, customerName: null });
                      setCustomerSearch("");
                    }}
                  >
                    Remove customer
                  </Button>
                </div>
              ) : (
                <div className="customer-results" aria-label="Active customer results">
                  {customers.map((customer) => (
                    <Button
                      key={customer.id}
                      size="sm"
                      variant="secondary"
                      onClick={() => {
                        change({
                          ...cart,
                          customerId: customer.id,
                          customerName: customer.displayName,
                        });
                        setCustomerSearch(customer.displayName);
                      }}
                    >
                      {customer.displayName} · {customer.code}
                    </Button>
                  ))}
                  {!customers.length && !customerError && <small>No active customers found. Continue as walk-in.</small>}
                </div>
              )}
              {customerError && <small role="alert">{customerError}</small>}
            </div>
            <Input
              label="Sale notes"
              value={cart.notes ?? ""}
              onChange={(e) =>
                change({ ...cart, notes: e.target.value || null })
              }
            />
          </fieldset>
          {previewError ? (
            <Alert title="Cart requires attention" tone="warning">
              {previewError}
            </Alert>
          ) : cart.lines.length && !total ? (
            <p role="status">Validating totals and FEFO…</p>
          ) : total ? (
            <div aria-label="Validated totals">
              <p>Subtotal {money(total.subtotalMinor)}</p>
              <p>
                Discount {money(total.discountMinor)} · Tax{" "}
                {money(total.taxMinor)}
              </p>
              <strong>Total {money(total.totalMinor)}</strong>
            </div>
          ) : null}
          <div className="row wrap">
            <Button
              disabled={busy || pendingPost}
              onClick={() =>
                action(async () => {
                  await persist();
                  setMessage("Draft persisted. No stock changed.");
                })
              }
            >
              Save Draft
            </Button>
            <Button
              disabled={busy || pendingPost}
              onClick={() =>
                action(async () => {
                  const s = await persist();
                  const h = await api.hold({
                    id: s.id,
                    expectedRevision: s.revision,
                  });
                  setSale(h);
                  setMessage("Held sale persisted. No stock reserved.");
                })
              }
            >
              Hold Sale
            </Button>
            <Button
              disabled={busy || (!total && !pendingPost)}
              onClick={() => {
                if (!pendingPost)
                  setAmount(((total?.totalMinor ?? 0) / 100).toFixed(2));
                setPayment(true);
              }}
            >
              {pendingPost ? "Retry Payment / Post" : "Payment / Post"}
            </Button>
          </div>
          {pendingPost && (
            <div>
              <p>
                Retry keeps the same request and payment. To edit, first reload
                the saved sale from history.
              </p>
              <Button
                disabled={busy}
                onClick={() =>
                  action(async () => {
                    const saved = await api.get(sale!.id);
                    if (
                      saved.status === "POSTED" ||
                      saved.status === "VOIDED"
                    ) {
                      navigate(`/sales/${saved.id}`);
                      return;
                    }
                    setSale(saved);
                    setCart(saved.cart);
                    pending.current = null;
                    setPendingPost(false);
                    setPayment(false);
                  })
                }
              >
                Reload Saved Cart
              </Button>
            </div>
          )}
        </Card>
      </div>
      <Modal
        description="Review the details before confirming."
        title="Post sale payment"
        open={payment}
        onOpenChange={(v) => {
          if (!busy) setPayment(v);
        }}
      >
        <fieldset disabled={busy || pendingPost}>
          <Select
            label="Payment method"
            showLabel
            value={method}
            onChange={(e) => setMethod(e.target.value as Method)}
          >
            <option>CASH</option>
            <option>CARD</option>
            <option>OTHER</option>
          </Select>
          <Input
            label="Payment amount (EGP)"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
          />
          <Input
            label="Payment reference"
            value={reference}
            onChange={(e) => setReference(e.target.value)}
          />
        </fieldset>
        <p>
          Record the exact total. Cash tender/change is not added to the payment
          amount. No payment provider is connected.
        </p>
        {error && <p role="alert">{error}</p>}
        <Button disabled={busy} onClick={post}>
          {busy
            ? "Posting…"
            : pendingPost
              ? "Retry same sale request"
              : "Confirm Post Sale"}
        </Button>
      </Modal>
    </div>
  );
}
function History() {
  const { branchId } = useBranch();
  const [search, setSearch] = useState(""),
    [status, setStatus] = useState(""),
    [method, setMethod] = useState(""),
    [from, setFrom] = useState(""),
    [to, setTo] = useState(""),
    [page, setPage] = useState(0),
    [attempt, setAttempt] = useState(0),
    [data, setData] = useState<{
      key: string;
      items: Sale[];
      total: number;
    } | null>(null),
    [error, setError] = useState("");
  const key = JSON.stringify([
    branchId,
    search,
    status,
    method,
    from,
    to,
    page,
    attempt,
  ]);
  useEffect(() => {
    let active = true;
    const t = setTimeout(
      () =>
        api
          .list({
            branchId,
            search,
            status: status || null,
            paymentMethod: (method || null) as Method | null,
            dateFrom: from || null,
            dateTo: to || null,
            limit: 20,
            offset: page * 20,
          })
          .then((r) => {
            if (active) {
              setData({ key, ...r });
              setError("");
            }
          })
          .catch((e) => {
            if (active) {
              setData(null);
              setError(errorText(e));
            }
          }),
      150,
    );
    return () => {
      active = false;
      clearTimeout(t);
    };
  }, [branchId, search, status, method, from, to, page, key]);
  const current = data?.key === key ? data : null;
  return (
    <div className="stack">
      <PageHeader
        title="Sales History"
        description="Persisted drafts, held carts and posted sales."
      />
      <Navigation />
      <Card>
        <div className="row wrap">
          <Input
            label="Sale number / customer search"
            value={search}
            onChange={(e) => {
              setSearch(e.target.value);
              setPage(0);
            }}
          />
          <Select
            label="Sale status"
            showLabel
            value={status}
            onChange={(e) => {
              setStatus(e.target.value);
              setPage(0);
            }}
          >
            <option value="">All statuses</option>
            {["DRAFT", "HELD", "POSTED", "VOIDED"].map((s) => (
              <option key={s}>{s}</option>
            ))}
          </Select>
          <Select
            label="History payment method"
            showLabel
            value={method}
            onChange={(e) => {
              setMethod(e.target.value);
              setPage(0);
            }}
          >
            <option value="">All methods</option>
            {["CASH", "CARD", "OTHER"].map((s) => (
              <option key={s}>{s}</option>
            ))}
          </Select>
          <Input
            label="Sales from"
            type="date"
            value={from}
            onChange={(e) => {
              setFrom(e.target.value);
              setPage(0);
            }}
          />
          <Input
            label="Sales through"
            type="date"
            value={to}
            onChange={(e) => {
              setTo(e.target.value);
              setPage(0);
            }}
          />
          <Button onClick={() => setAttempt((a) => a + 1)}>
            Refresh sales
          </Button>
        </div>
        {error ? (
          <Alert title="Sales history failed" tone="danger">
            {error}
          </Alert>
        ) : !current ? (
          <p role="status">Loading sales…</p>
        ) : (
          <>
            <Table
              label="Canonical sales history"
              rows={current.items}
              rowKey={(s) => s.id}
              columns={[
                {
                  key: "number",
                  header: "Sale",
                  render: (s) => (
                    <LinkButton to={`/sales/${s.id}`}>
                      {s.saleNumber || `Draft ${s.id.slice(0, 8)}`}
                    </LinkButton>
                  ),
                },
                { key: "date", header: "Date", render: (s) => s.createdAt },
                {
                  key: "customer",
                  header: "Customer",
                  render: (s) => s.customerName || "Walk-in",
                },
                {
                  key: "total",
                  header: "Total",
                  render: (s) => money(s.totalMinor),
                },
                {
                  key: "payment",
                  header: "Payment",
                  render: (s) =>
                    `${s.paymentStatus} · ${s.paymentMethods || "—"} · Refunded ${money(s.refundedMinor || 0)}`,
                },
                { key: "status", header: "Status", render: (s) => s.status },
              ]}
            />
            {!current.total && <p>No matching sales.</p>}
            <div className="row">
              <Button disabled={!page} onClick={() => setPage(page - 1)}>
                Previous sales
              </Button>
              <span>{current.total} sales</span>
              <Button
                disabled={(page + 1) * 20 >= current.total}
                onClick={() => setPage(page + 1)}
              >
                Next sales
              </Button>
            </div>
          </>
        )}
      </Card>
    </div>
  );
}
function Detail({ id }: { id: string }) {
  const { branchName, select, branchId } = useBranch();
  const navigate = useNavigate();
  const [sale, setSale] = useState<Sale | null>(null),
    [error, setError] = useState(""),
    [busy, setBusy] = useState(false),
    [attempt, setAttempt] = useState(0),
    [receipt, setReceipt] = useState(false),
    [returnOpen, setReturnOpen] = useState(false),
    [voidOpen, setVoidOpen] = useState(false),
    [reason, setReason] = useState(""),
    [quantities, setQuantities] = useState<Record<string, number>>({}),
    [refund, setRefund] = useState<number | null>(null),
    [returnFrozen, setReturnFrozen] = useState(false),
    [voidFrozen, setVoidFrozen] = useState(false),
    [financeTrace, setFinanceTrace] = useState<SaleFinanceTrace | null>(null),
    [financeError, setFinanceError] = useState("");
  const lock = useRef(false),
    returnPending = useRef<{ input: ReturnInput; requestId: string } | null>(
      null,
    ),
    voidPending = useRef<{
      id: string;
      requestId: string;
      reason: string;
    } | null>(null);
  useEffect(() => {
    let active = true;
    api
      .get(id)
      .then((s) => {
        if (active) {
          setSale(s);
          setError("");
        }
      })
      .catch((e) => {
        if (active) {
          setSale(null);
          setError(errorText(e));
        }
      });
    return () => {
      active = false;
    };
  }, [id, attempt]);
  useEffect(() => {
    let active = true;
    phase6.saleTrace(id).then((value) => {
      if (active) { setFinanceTrace(value); setFinanceError(""); }
    }).catch((e) => active && setFinanceError(errorText(e)));
    return () => { active = false; };
  }, [id, attempt, sale?.status, sale?.refundedMinor]);
  async function action(fn: () => Promise<void>) {
    if (lock.current) return;
    lock.current = true;
    setBusy(true);
    setError("");
    try {
      await fn();
    } catch (e) {
      setError(errorText(e));
    } finally {
      lock.current = false;
      setBusy(false);
    }
  }
  function returnInput(): ReturnInput {
    return {
      id: crypto.randomUUID(),
      saleId: id,
      reason,
      lines: Object.entries(quantities)
        .filter(([, q]) => q > 0)
        .map(([saleItemId, quantity]) => ({ saleItemId, quantity })),
    };
  }
  if (!sale)
    return (
      <div className="stack">
        <Navigation />
        {error ? (
          <Alert title="Sale could not be loaded" tone="danger">
            {error}
            <Button onClick={() => setAttempt((a) => a + 1)}>Retry sale</Button>
          </Alert>
        ) : (
          <p role="status">Loading sale…</p>
        )}
      </div>
    );
  return (
    <div className="stack">
      <PageHeader
        title={sale.saleNumber || "Saved Sale"}
        description={`${sale.status} · ${sale.createdAt}`}
      />
      <Navigation />
      {error && (
        <Alert title="Sale action failed" tone="danger">
          {error}
        </Alert>
      )}
      <Card>
        <p>
          Branch: {sale.branchId === branchId ? branchName : sale.branchId} ·
          Customer: {sale.customerName || "Walk-in"}
        </p>
        <p>
          Total {money(sale.totalMinor)} · Refunded {money(sale.refundedMinor)}{" "}
          · {sale.paymentStatus}
        </p>
        {sale.status === "DRAFT" || sale.status === "HELD" ? (
          <>
            <p>{sale.cart.lines.length} saved packages. No stock posted.</p>
            <Button
              onClick={() => {
                select(sale.branchId);
                navigate(`/sales?resume=${sale.id}`);
              }}
            >
              Resume Cart
            </Button>
          </>
        ) : (
          <>
            <Table
              label="Sale item snapshots"
              rows={sale.items}
              rowKey={(i) => i.id}
              columns={[
                {
                  key: "name",
                  header: "Product / package",
                  render: (i) =>
                    `${i.productNameSnapshot} · ${i.packageLabelSnapshot}`,
                },
                {
                  key: "qty",
                  header: "Sold / returned / remaining",
                  render: (i) =>
                    `${i.quantity} / ${i.returnedQuantity} / ${i.remainingQuantity}`,
                },
                {
                  key: "price",
                  header: "Price",
                  render: (i) => money(i.unitPriceMinor),
                },
                {
                  key: "discount",
                  header: "Line / sale discount",
                  render: (i) =>
                    `${money(i.discountMinor)} / ${money(i.saleDiscountMinor)}`,
                },
                { key: "tax", header: "Tax", render: (i) => money(i.taxMinor) },
                {
                  key: "total",
                  header: "Paid line total",
                  render: (i) => money(i.lineTotalMinor),
                },
              ]}
            />
            {sale.items.map((i) => (
              <div key={i.id}>
                <h3>{i.productNameSnapshot} · Batch allocations</h3>
                <Table
                  label={`Allocations ${i.id}`}
                  rows={i.allocations}
                  rowKey={(a) => a.id || a.batchId}
                  columns={[
                    {
                      key: "batch",
                      header: "Batch / lot",
                      render: (a) =>
                        `${a.batchId} · ${a.lotNumber || "Unknown lot"}`,
                    },
                    {
                      key: "qty",
                      header: "Quantity",
                      render: (a) => a.quantity,
                    },
                    {
                      key: "cost",
                      header: "Historical unit cost",
                      render: (a) =>
                        a.unitCostMinor === null
                          ? "Unknown"
                          : money(a.unitCostMinor),
                    },
                    {
                      key: "move",
                      header: "Movement",
                      render: (a) => (
                        <LinkButton
                          to={`/inventory/movements?package=${i.productPackageId}&batch=${a.batchId}`}
                        >
                          {a.movementId}
                        </LinkButton>
                      ),
                    },
                  ]}
                />
              </div>
            ))}
            <h3>Payments</h3>
            {sale.payments.map((p, i) => (
              <p key={i}>
                {p.paymentMethod} · {money(p.amountMinor)} ·{" "}
                {p.reference || "No reference"}
              </p>
            ))}
            <div className="row wrap">
              <Button onClick={() => setReceipt(true)}>Receipt Preview</Button>
              <Button
                disabled={
                  sale.status !== "POSTED" ||
                  busy ||
                  sale.items.every((i) => i.returnedQuantity === i.quantity)
                }
                onClick={() => setReturnOpen(true)}
              >
                Return Items
              </Button>
              <Button
                disabled={
                  sale.status !== "POSTED" ||
                  busy ||
                  sale.returns.some((r) => r.status === "POSTED")
                }
                onClick={() => setVoidOpen(true)}
              >
                Void Sale
              </Button>
            </div>
          </>
        )}
      </Card>
      {(sale.status === "POSTED" || sale.status === "VOIDED") && (
        <Card>
          <h2>Finance trace</h2>
          {financeError && <Alert title="Finance trace could not be loaded" tone="danger">{financeError}</Alert>}
          {!financeError && !financeTrace && <p role="status">Loading Finance trace…</p>}
          {financeTrace && <>
            <p>{financeTrace.coverageNote}</p>
            {financeTrace.journals.length === 0 ? <p className="muted">No related Finance journals.</p> : financeTrace.journals.map((journal) => <p key={journal.id}><LinkButton to={`/finance/journals/${journal.id}`}>{journal.journalNumber}</LinkButton> · {journal.sourceType} · {money(journal.debitTotalMinor)} · {journal.cogsComplete ? "Cost complete" : "Cost incomplete"}</p>)}
          </>}
        </Card>
      )}
      <Card>
        <h2>Returns and reversals</h2>
        {!sale.returns.length && !sale.voids.length && (
          <p>No returns or voids.</p>
        )}
        {sale.returns.map((r) => (
          <div key={r.id}>
            <strong>
              Return {r.id} · {r.status} · {money(r.refundMinor)}
            </strong>
            <p>{r.reason}</p>
            {r.allocations?.map((a) => (
              <p key={a.movementId}>
                {a.originalAllocationId} → {a.quantity} · SALE_RETURN{" "}
                {a.movementId}
              </p>
            ))}
            {r.status === "DRAFT" && (
              <Button
                disabled={busy}
                onClick={() => {
                  returnPending.current = {
                    input: {
                      id: r.id,
                      saleId: id,
                      reason: r.reason,
                      lines: r.lines,
                    },
                    requestId: crypto.randomUUID(),
                  };
                  setReturnFrozen(true);
                  setReason(r.reason);
                  setQuantities(
                    Object.fromEntries(
                      r.lines.map((l) => [l.saleItemId, l.quantity]),
                    ),
                  );
                  setRefund(r.refundMinor);
                  setReturnOpen(true);
                }}
              >
                Resume Return
              </Button>
            )}
          </div>
        ))}
        {sale.voids.map((v) => (
          <p key={v.id}>
            Void {v.id} · {v.reason} · Refund {money(v.refundMinor)}
          </p>
        ))}
        {sale.voidAllocations.map((a) => (
          <p key={a.movementId}>
            {a.originalAllocationId} → reversal {a.movementId}
          </p>
        ))}
      </Card>
      <Modal
        description="Review the details before confirming."
        title="Receipt preview"
        open={receipt}
        onOpenChange={setReceipt}
      >
        <h2>
          M² Health · {sale.branchId === branchId ? branchName : sale.branchId}
        </h2>
        <p>
          {sale.saleNumber} · {sale.postedAt}
        </p>
        {sale.items.map((i) => (
          <p key={i.id}>
            {i.productNameSnapshot} · {i.quantity} × {money(i.unitPriceMinor)} ·
            net {money(i.lineTotalMinor)}
          </p>
        ))}
        <p>
          Discount {money(sale.discountMinor)} · Tax {money(sale.taxMinor)}
        </p>
        <strong>Total {money(sale.totalMinor)}</strong>
        {sale.payments.map((p, i) => (
          <p key={i}>
            {p.paymentMethod}: {money(p.amountMinor)}
          </p>
        ))}
        <p>Preview only · hardware printing is not connected.</p>
      </Modal>
      <Modal
        description="Review the details before confirming."
        title="Return sold items"
        open={returnOpen}
        onOpenChange={(v) => {
          if (!busy) setReturnOpen(v);
        }}
      >
        <fieldset disabled={busy || returnFrozen}>
          {sale.items.map((i) => (
            <Input
              key={i.id}
              label={`Return quantity ${i.productNameSnapshot}`}
              type="number"
              min="0"
              max={i.remainingQuantity}
              step="1"
              value={quantities[i.id] ?? 0}
              onChange={(e) => {
                setQuantities({
                  ...quantities,
                  [i.id]: Number(e.target.value),
                });
                setRefund(null);
              }}
            />
          ))}
          <Input
            label="Return reason"
            value={reason}
            onChange={(e) => {
              setReason(e.target.value);
              setRefund(null);
            }}
          />
        </fieldset>
        <Button
          disabled={busy || returnFrozen}
          onClick={() =>
            action(async () => {
              setRefund((await api.previewReturn(returnInput())).refundMinor);
            })
          }
        >
          Calculate Refund
        </Button>
        {refund !== null && <p>Validated refund: {money(refund)}</p>}
        {error && <p role="alert">{error}</p>}
        <Button
          disabled={busy || refund === null}
          onClick={() =>
            action(async () => {
              let pending = returnPending.current;
              if (!pending) {
                pending = {
                  input: returnInput(),
                  requestId: crypto.randomUUID(),
                };
                returnPending.current = pending;
                setReturnFrozen(true);
              }
              await api.createReturn(pending.input);
              const s = await api.postReturn({
                id: pending.input.id,
                requestId: pending.requestId,
              });
              setSale(s);
              setReturnOpen(false);
              returnPending.current = null;
              setReturnFrozen(false);
              setRefund(null);
              setQuantities({});
            })
          }
        >
          {busy ? "Posting return…" : "Confirm Return"}
        </Button>
      </Modal>
      <Modal
        description="Review the details before confirming."
        title="Void posted sale"
        open={voidOpen}
        onOpenChange={(v) => {
          if (!busy) setVoidOpen(v);
        }}
      >
        <p>
          This preserves the sale and payments, restores its exact batch
          allocations and records a full refund. Sales with posted returns
          cannot be voided.
        </p>
        <Input
          label="Void reason"
          value={reason}
          disabled={busy || voidFrozen}
          onChange={(e) => setReason(e.target.value)}
        />
        {error && <p role="alert">{error}</p>}
        <Button
          disabled={busy || !reason.trim()}
          onClick={() =>
            action(async () => {
              voidPending.current ??= {
                id,
                requestId: crypto.randomUUID(),
                reason,
              };
              setVoidFrozen(true);
              setSale(await api.voidSale(voidPending.current));
              setVoidOpen(false);
            })
          }
        >
          {busy ? "Voiding…" : "Confirm Void"}
        </Button>
      </Modal>
    </div>
  );
}
