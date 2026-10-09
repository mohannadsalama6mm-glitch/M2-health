import { useEffect, useMemo, useRef, useState } from "react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { PackagePlus, Pencil, Plus, Trash2 } from "lucide-react";
import {
  Alert,
  BackLink,
  Button,
  Card,
  DetailList,
  EmptyState,
  Input,
  Modal,
  PageHeader,
  SearchInput,
  Select,
  Status,
  Table,
  TableToolbar,
  Textarea,
} from "../../design-system";
import { canonicalPartners, type Supplier } from "../../lib/tauri/canonical-partners";
import {
  canonicalPurchasing as api,
  type PurchaseDraftFields,
  type PurchaseOrder,
  type PurchasePackage,
  type PurchaseReceipt,
  type PurchaseReceiptLineInput,
  type PurchaseReturn,
  type PurchaseReturnSummary,
  type PurchaseRow,
  type PurchaseStatus,
} from "../../lib/tauri/canonical-purchasing";
import { useBranch } from "../../app/BranchContext";
import "./canonical-purchases.css";

const size = 20;
const errorText = (e: unknown) =>
  e instanceof Error
    ? e.message
    : typeof e === "object" && e && "message" in e
      ? String(e.message)
      : "The purchasing request failed.";
const money = (minor: number) => `${(minor / 100).toFixed(2)} EGP`;
const major = (minor: number) => (minor / 100).toFixed(2);
function minor(value: string) {
  const text = value.trim() || "0";
  if (!/^\d+(\.\d{0,2})?$/.test(text))
    throw new Error("Enter a non-negative amount with at most two decimals.");
  const [whole, fraction = ""] = text.split(".");
  const result = Number(whole) * 100 + Number(fraction.padEnd(2, "0"));
  if (!Number.isSafeInteger(result)) throw new Error("Amount is too large.");
  return result;
}
const stamp = (value: string | null) =>
  value ? value.slice(0, 16).replace("T", " ") : "—";

export function CanonicalPurchases() {
  const { pathname } = useLocation();
  const parts = pathname.split("/").filter(Boolean);
  if (parts[1] === "returns" && parts[2])
    return <ReturnDetails id={parts[2]} />;
  if (parts[1] === "returns") return <ReturnHistory />;
  if (parts[1] === "receipts" && parts[2] && parts[3] === "return")
    return <ReturnEntry receiptId={parts[2]} />;
  if (parts[1] === "receipts" && parts[2])
    return <ReceiptDetails id={parts[2]} />;
  if (pathname === "/purchases/new") return <Editor />;
  if (parts[1] && parts[2] === "edit") return <Editor id={parts[1]} />;
  if (parts[1] && parts[2] === "receive")
    return <Receiving id={parts[1]} />;
  if (parts[1]) return <Details id={parts[1]} />;
  return <PurchaseList />;
}

function PurchaseList() {
  const [search, setSearch] = useState("");
  const [status, setStatus] = useState("");
  const [page, setPage] = useState(0);
  const [attempt, setAttempt] = useState(0);
  const [data, setData] = useState<{ key: string; items: PurchaseRow[]; total: number } | null>(null);
  const [error, setError] = useState("");
  const key = JSON.stringify([search, status, page, attempt]);
  useEffect(() => {
    let active = true;
    const timer = setTimeout(() => {
      api
        .list({
          search: search || null,
          status: (status || null) as PurchaseStatus | null,
          limit: size,
          offset: page * size,
        })
        .then((result) => {
          if (active) {
            setData({ key, items: result.items, total: result.total });
            setError("");
          }
        })
        .catch((e) => {
          if (active) {
            setData(null);
            setError(errorText(e));
          }
        });
    }, search ? 180 : 0);
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [attempt, key, page, search, status]);
  const current = data?.key === key ? data : null;
  return (
    <div className="feature-page canonical-purchases">
      <PageHeader
        title="Purchases"
        description="Create, receive and trace purchase orders and supplier returns."
        actions={
          <><Button onClick={() => (window.location.hash = "#/purchases/returns")}>Return history</Button><Button variant="primary" onClick={() => (window.location.hash = "#/purchases/new")}><Plus size={16} /> Create purchase</Button></>
        }
      />
      {error && (
        <Alert title="Purchases could not be loaded" tone="danger">
          {error} <Button size="sm" onClick={() => setAttempt((v) => v + 1)}>Retry</Button>
        </Alert>
      )}
      <TableToolbar>
        <SearchInput
          label="Search purchases"
          placeholder="Purchase number, supplier or reference"
          value={search}
          onChange={(e) => { setSearch(e.target.value); setPage(0); }}
        />
        <Select
          label="Purchase status"
          showLabel
          value={status}
          onChange={(e) => { setStatus(e.target.value); setPage(0); }}
        >
          <option value="">All statuses</option>
          <option value="DRAFT">Draft</option>
          <option value="ORDERED">Ordered</option>
          <option value="CANCELLED">Cancelled</option>
        </Select>
      </TableToolbar>
      {!current ? (
        <Card><p role="status">Loading purchases…</p></Card>
      ) : current.items.length === 0 ? (
        <EmptyState
          title={search || status ? "No purchases match these filters" : "No purchase orders yet"}
          description={search || status ? "Change the search or status filter." : "Create a draft purchase order to begin."}
        />
      ) : (
        <Card>
          <Table
            label="Purchase orders"
            rows={current.items}
            rowKey={(row) => row.id}
            columns={[
              { key: "number", header: "Purchase", render: (row) => <Link className="text-link" to={`/purchases/${row.id}`}>{row.purchaseNumber}<span className="cell-secondary">{stamp(row.createdAt)}</span></Link> },
              { key: "supplier", header: "Supplier", render: (row) => <>{row.supplierName}<span className="cell-secondary">{row.supplierCode}</span></> },
              { key: "lines", header: "Lines", render: (row) => row.lineCount },
              { key: "total", header: "Total", render: (row) => money(row.totalMinor) },
              { key: "status", header: "Status", render: (row) => <Status value={row.status} /> },
            ]}
          />
          <div className="grid-footer">
            <Button disabled={page === 0} onClick={() => setPage((v) => v - 1)}>Previous</Button>
            <span>{current.total} total · Page {page + 1}</span>
            <Button disabled={(page + 1) * size >= current.total} onClick={() => setPage((v) => v + 1)}>Next</Button>
          </div>
        </Card>
      )}
    </div>
  );
}

type EditableLine = {
  productPackageId: string;
  productName: string;
  packageLabel: string;
  quantity: string;
  cost: string;
};

function Editor({ id }: { id?: string }) {
  const navigate = useNavigate();
  const [loaded, setLoaded] = useState(!id);
  const [revision, setRevision] = useState(0);
  const [supplierId, setSupplierId] = useState("");
  const [supplierReference, setSupplierReference] = useState("");
  const [expectedDate, setExpectedDate] = useState("");
  const [notes, setNotes] = useState("");
  const [discount, setDiscount] = useState("0.00");
  const [tax, setTax] = useState("0.00");
  const [lines, setLines] = useState<EditableLine[]>([]);
  const [supplierSearch, setSupplierSearch] = useState("");
  const [suppliers, setSuppliers] = useState<Supplier[]>([]);
  const [packageSearch, setPackageSearch] = useState("");
  const [packages, setPackages] = useState<PurchasePackage[]>([]);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const lock = useRef(false);
  const draftId = useRef(id ?? crypto.randomUUID());
  const requestId = useRef(crypto.randomUUID());

  useEffect(() => {
    if (!id) return;
    let active = true;
    api.get(id).then((order) => {
      if (!active) return;
      if (order.status !== "DRAFT") throw new Error("Only draft purchase orders can be edited.");
      setRevision(order.revision);
      setSupplierId(order.supplierId);
      setSupplierReference(order.supplierReference ?? "");
      setExpectedDate(order.expectedDate ?? "");
      setNotes(order.notes ?? "");
      setDiscount(major(order.discountMinor));
      setTax(major(order.taxMinor));
      setLines(order.lines.map((line) => ({
        productPackageId: line.productPackageId,
        productName: line.productNameSnapshot,
        packageLabel: line.packageLabelSnapshot,
        quantity: String(line.orderedQuantity),
        cost: major(line.unitCostMinor),
      })));
      setLoaded(true);
    }).catch((e) => setError(errorText(e)));
    return () => { active = false; };
  }, [id]);
  useEffect(() => {
    let active = true;
    const timer = setTimeout(() => canonicalPartners.listSuppliers({ search: supplierSearch || null, isActive: true, limit: 20, offset: 0 })
      .then((result) => { if (active) setSuppliers(result.items); })
      .catch((e) => { if (active) setError(errorText(e)); }), supplierSearch ? 180 : 0);
    return () => { active = false; clearTimeout(timer); };
  }, [supplierSearch]);
  useEffect(() => {
    let active = true;
    const timer = setTimeout(() => api.searchPackages({ search: packageSearch || null, limit: 20, offset: 0 })
      .then((result) => { if (active) setPackages(result.items); })
      .catch((e) => { if (active) setError(errorText(e)); }), packageSearch ? 180 : 0);
    return () => { active = false; clearTimeout(timer); };
  }, [packageSearch]);

  const calculated = useMemo(() => {
    try {
      const subtotal = lines.reduce((sum, line) => sum + Number(line.quantity || 0) * minor(line.cost), 0);
      const discountMinor = minor(discount);
      const taxMinor = minor(tax);
      return { subtotal, discountMinor, taxMinor, total: subtotal - discountMinor + taxMinor };
    } catch { return null; }
  }, [discount, lines, tax]);
  const add = (item: PurchasePackage) => {
    if (lines.some((line) => line.productPackageId === item.productPackageId)) {
      setError("This package is already on the purchase order.");
      return;
    }
    setLines((current) => [...current, {
      productPackageId: item.productPackageId,
      productName: item.productName,
      packageLabel: item.packageLabel,
      quantity: "1",
      cost: major(item.currentCostMinor ?? 0),
    }]);
    setError("");
  };
  const save = async () => {
    if (lock.current) return;
    lock.current = true;
    setBusy(true);
    setError("");
    try {
      if (!supplierId) throw new Error("Select an active supplier.");
      const fields: PurchaseDraftFields = {
        supplierId,
        supplierReference: supplierReference.trim() || null,
        expectedDate: expectedDate || null,
        notes: notes.trim() || null,
        discountMinor: minor(discount),
        taxMinor: minor(tax),
        lines: lines.map((line) => {
          const quantity = Number(line.quantity);
          if (!Number.isSafeInteger(quantity) || quantity <= 0) throw new Error("Every quantity must be a positive whole number.");
          return { productPackageId: line.productPackageId, orderedQuantity: quantity, unitCostMinor: minor(line.cost) };
        }),
      };
      const saved = id
        ? await api.updateDraft({ id, expectedRevision: revision, ...fields })
        : await api.createDraft({ id: draftId.current, requestId: requestId.current, ...fields });
      navigate(`/purchases/${saved.id}`);
    } catch (e) {
      setError(errorText(e));
    } finally {
      lock.current = false;
      setBusy(false);
    }
  };
  if (!loaded) return <Card><p role="status">Loading purchase draft…</p></Card>;
  return (
    <div className="feature-page canonical-purchases">
      <BackLink to={id ? `/purchases/${id}` : "/purchases"} label={id ? "Purchase detail" : "Purchases"} />
      <PageHeader title={id ? "Edit purchase draft" : "Create purchase draft"} description="Saving a draft does not change stock." actions={<Button variant="primary" loading={busy} disabled={busy} onClick={save}>Save draft</Button>} />
      {error && <Alert title="Draft could not be saved" tone="danger">{error}</Alert>}
      <div className="purchase-editor-grid">
        <Card>
          <h2>Supplier and terms</h2>
          <SearchInput label="Search active suppliers" placeholder="Supplier code or name" value={supplierSearch} onChange={(e) => setSupplierSearch(e.target.value)} />
          <Select label="Supplier" showLabel required value={supplierId} onChange={(e) => setSupplierId(e.target.value)}>
            <option value="">Select an active supplier</option>
            {suppliers.map((supplier) => <option key={supplier.id} value={supplier.id}>{supplier.code} · {supplier.displayName}</option>)}
          </Select>
          <Input label="Supplier reference" value={supplierReference} onChange={(e) => setSupplierReference(e.target.value)} />
          <Input label="Expected date" type="date" value={expectedDate} onChange={(e) => setExpectedDate(e.target.value)} />
          <Textarea label="Notes" value={notes} onChange={(e) => setNotes(e.target.value)} />
        </Card>
        <Card>
          <h2>Find Catalog packages</h2>
          <SearchInput label="Search packages" placeholder="Product, package, manufacturer or barcode" value={packageSearch} onChange={(e) => setPackageSearch(e.target.value)} />
          <div className="purchase-package-results" aria-label="Catalog package results">
            {packages.map((item) => <button type="button" key={item.productPackageId} onClick={() => add(item)} disabled={lines.some((line) => line.productPackageId === item.productPackageId)}>
              <span><strong>{item.productName}</strong><small>{item.packageLabel} · {item.manufacturerName ?? "Manufacturer unavailable"}{item.barcode ? ` · ${item.barcode}` : ""}</small></span>
              <span><PackagePlus size={16} /> {money(item.currentCostMinor ?? 0)}</span>
            </button>)}
          </div>
        </Card>
      </div>
      <Card>
        <h2>Purchase lines</h2>
        {lines.length === 0 ? <EmptyState title="No packages added" description="Search the Catalog and add the stockable package identity." /> : (
          <div className="purchase-lines-scroll">
            <table className="purchase-lines"><thead><tr><th>Package</th><th>Quantity</th><th>Unit cost</th><th>Line total</th><th><span className="sr-only">Remove</span></th></tr></thead>
              <tbody>{lines.map((line, index) => <tr key={line.productPackageId}>
                <td><strong>{line.productName}</strong><span>{line.packageLabel}</span></td>
                <td><Input label={`Quantity for ${line.productName}`} hideLabel type="number" min={1} step={1} value={line.quantity} onChange={(e) => setLines((all) => all.map((item, i) => i === index ? { ...item, quantity: e.target.value } : item))} /></td>
                <td><Input label={`Unit cost for ${line.productName}`} hideLabel inputMode="decimal" value={line.cost} onChange={(e) => setLines((all) => all.map((item, i) => i === index ? { ...item, cost: e.target.value } : item))} /></td>
                <td>{(() => { try { return money(Number(line.quantity || 0) * minor(line.cost)); } catch { return "Invalid"; } })()}</td>
                <td><Button size="sm" aria-label={`Remove ${line.productName}`} onClick={() => setLines((all) => all.filter((_, i) => i !== index))}><Trash2 size={15} /></Button></td>
              </tr>)}</tbody></table>
          </div>
        )}
        <div className="purchase-totals-edit">
          <Input label="Discount (EGP)" inputMode="decimal" value={discount} onChange={(e) => setDiscount(e.target.value)} />
          <Input label="Tax (EGP)" inputMode="decimal" value={tax} onChange={(e) => setTax(e.target.value)} />
          <dl><div><dt>Subtotal</dt><dd>{calculated ? money(calculated.subtotal) : "Invalid"}</dd></div><div><dt>Total</dt><dd>{calculated && calculated.total >= 0 ? money(calculated.total) : "Invalid"}</dd></div></dl>
        </div>
      </Card>
    </div>
  );
}

function Details({ id }: { id: string }) {
  const [order, setOrder] = useState<PurchaseOrder | null>(null);
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);
  const [orderOpen, setOrderOpen] = useState(false);
  const [cancelOpen, setCancelOpen] = useState(false);
  const [reason, setReason] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const lock = useRef(false);
  const orderRequest = useRef<string | null>(null);
  const cancelRequest = useRef<string | null>(null);
  useEffect(() => {
    let active = true;
    api.get(id).then((result) => { if (active) { setOrder(result); setError(""); } }).catch((e) => { if (active) setError(errorText(e)); });
    return () => { active = false; };
  }, [attempt, id]);
  const action = async (kind: "order" | "cancel") => {
    if (!order || lock.current) return;
    lock.current = true; setBusy(true); setError("");
    try {
      let result: PurchaseOrder;
      if (kind === "order") {
        orderRequest.current ??= crypto.randomUUID();
        result = await api.order({ id, expectedRevision: order.revision, requestId: orderRequest.current });
        orderRequest.current = null; setOrderOpen(false); setMessage("Purchase order finalized. No stock was received.");
      } else {
        if (!reason.trim()) throw new Error("Enter a cancellation reason.");
        cancelRequest.current ??= crypto.randomUUID();
        result = await api.cancel({ id, expectedRevision: order.revision, requestId: cancelRequest.current, reason });
        cancelRequest.current = null; setCancelOpen(false); setMessage("Purchase order cancelled.");
      }
      setOrder(result);
    } catch (e) { setError(errorText(e)); }
    finally { lock.current = false; setBusy(false); }
  };
  if (error && !order) return <><BackLink to="/purchases" label="Purchases" /><Alert title="Purchase could not be loaded" tone="danger">{error} <Button size="sm" onClick={() => setAttempt((v) => v + 1)}>Retry</Button></Alert></>;
  if (!order) return <Card><p role="status">Loading purchase…</p></Card>;
  const draft = order.status === "DRAFT";
  const receivable = ["ORDERED", "PARTIALLY_RECEIVED"].includes(order.status);
  return (
    <div className="feature-page canonical-purchases">
      <BackLink to="/purchases" label="Purchases" />
      <PageHeader title={order.purchaseNumber} description={`${order.supplierName} · Revision ${order.revision}`} actions={<>
        {draft && <Button onClick={() => (window.location.hash = `#/purchases/${id}/edit`)}><Pencil size={15} /> Edit draft</Button>}
        {draft && <Button variant="primary" onClick={() => setOrderOpen(true)}>Order purchase</Button>}
        {(draft || order.status === "ORDERED") && <Button variant="danger" onClick={() => setCancelOpen(true)}>Cancel purchase</Button>}
        {receivable && <Button variant="primary" onClick={() => (window.location.hash = `#/purchases/${id}/receive`)}>Receive stock</Button>}
      </>} />
      {message && <Alert title="Purchase updated" tone="success">{message}</Alert>}
      {error && <Alert title="Purchase action failed" tone="danger">{error}</Alert>}
      {receivable && <Alert title="Canonical receiving available" tone="info">Receipts post atomically to the selected branch through the existing Inventory ledger.</Alert>}
      <Card>
        <DetailList items={[
          { label: "Status", value: <Status value={order.status} /> },
          { label: "Supplier", value: `${order.supplierCode} · ${order.supplierName}` },
          { label: "Supplier reference", value: order.supplierReference || "—" },
          { label: "Expected date", value: order.expectedDate || "—" },
          { label: "Created", value: stamp(order.createdAt) },
          { label: "Ordered", value: stamp(order.orderedAt) },
          { label: "Cancelled", value: stamp(order.cancelledAt) },
          { label: "Cancellation reason", value: order.cancellationReason || "—" },
          { label: "Notes", value: order.notes || "—" },
        ]} />
      </Card>
      <Card>
        <Table label="Purchase order lines" rows={order.lines} rowKey={(line) => line.id} columns={[
          { key: "package", header: "Package", render: (line) => <>{line.productNameSnapshot}<span className="cell-secondary">{line.packageLabelSnapshot}</span></> },
          { key: "quantity", header: "Ordered", render: (line) => line.orderedQuantity },
          { key: "received", header: "Received", render: (line) => line.receivedQuantity },
          { key: "remaining", header: "Remaining", render: (line) => line.remainingQuantity },
          { key: "returned", header: "Returned", render: (line) => line.returnedQuantity },
          { key: "returnable", header: "Returnable", render: (line) => line.remainingReturnableQuantity },
          { key: "cost", header: "Unit cost", render: (line) => money(line.unitCostMinor) },
          { key: "total", header: "Line total", render: (line) => money(line.lineTotalMinor) },
        ]} />
        <div className="purchase-totals"><span>Subtotal <strong>{money(order.subtotalMinor)}</strong></span><span>Discount <strong>{money(order.discountMinor)}</strong></span><span>Tax <strong>{money(order.taxMinor)}</strong></span><span>Total <strong>{money(order.totalMinor)}</strong></span></div>
      </Card>
      <Card>
        <h2>Receipt history</h2>
        {order.receipts.length === 0 ? <EmptyState title="No receipts posted" description="Stock has not been received against this purchase order." /> : <Table label="Purchase receipt history" rows={order.receipts} rowKey={(receipt) => receipt.id} columns={[
          { key: "number", header: "Receipt", render: (receipt) => <Link className="text-link" to={`/purchases/receipts/${receipt.id}`}>{receipt.receiptNumber}</Link> },
          { key: "branch", header: "Branch", render: (receipt) => <>{receipt.branchName}<span className="cell-secondary">{receipt.branchCode}</span></> },
          { key: "quantity", header: "Quantity", render: (receipt) => receipt.totalQuantity },
          { key: "posted", header: "Posted", render: (receipt) => stamp(receipt.postedAt) },
        ]} />}
      </Card>
      <Card>
        <h2>Purchase return history</h2>
        {order.returns.length === 0 ? <EmptyState title="No supplier returns posted" description="Posted returns against received stock appear here." /> : <Table label="Purchase return history" rows={order.returns} rowKey={(item) => item.id} columns={[
          {key:"number",header:"Return",render:(item)=><Link className="text-link" to={`/purchases/returns/${item.id}`}>{item.returnNumber}</Link>},
          {key:"receipt",header:"Original receipt",render:(item)=><Link className="text-link" to={`/purchases/receipts/${item.receiptId}`}>{item.receiptNumber}</Link>},
          {key:"quantity",header:"Quantity",render:(item)=>item.totalQuantity},
          {key:"reason",header:"Reason",render:(item)=>item.reason},
          {key:"posted",header:"Posted",render:(item)=>stamp(item.postedAt)},
        ]}/>} 
      </Card>
      <Modal open={orderOpen} onOpenChange={(value) => { if (!busy) setOrderOpen(value); }} title="Order this purchase?" description="The supplier and line commitment becomes read-only. This action does not receive stock.">
        <div className="row end"><Button disabled={busy} onClick={() => setOrderOpen(false)}>Keep draft</Button><Button variant="primary" loading={busy} onClick={() => action("order")}>Confirm order</Button></div>
      </Modal>
      <Modal open={cancelOpen} onOpenChange={(value) => { if (!busy) setCancelOpen(value); }} title="Cancel purchase order" description="The cancelled order remains in purchasing history.">
        <div className="stack"><Textarea label="Cancellation reason" required value={reason} onChange={(e) => { setReason(e.target.value); cancelRequest.current = null; }} /><div className="row end"><Button disabled={busy} onClick={() => setCancelOpen(false)}>Keep purchase</Button><Button variant="danger" loading={busy} disabled={!reason.trim()} onClick={() => action("cancel")}>Confirm cancellation</Button></div></div>
      </Modal>
    </div>
  );
}

type ReceiptEdit = { quantity: string; lot: string; expiry: string; cost: string };
function Receiving({ id }: { id: string }) {
  const navigate = useNavigate();
  const { branchId, branchName, loading: branchLoading } = useBranch();
  const [order,setOrder]=useState<PurchaseOrder|null>(null);
  const [edits,setEdits]=useState<Record<string,ReceiptEdit>>({});
  const [error,setError]=useState("");
  const [review,setReview]=useState(false);
  const [busy,setBusy]=useState(false);
  const lock=useRef(false);
  const receiptId=useRef(crypto.randomUUID());
  const requestId=useRef(crypto.randomUUID());
  useEffect(()=>{ let active=true; api.get(id).then((value)=>{ if(!active)return; if(!["ORDERED","PARTIALLY_RECEIVED"].includes(value.status)) throw new Error("This purchase order is not eligible for receiving."); setOrder(value); setEdits(Object.fromEntries(value.lines.filter((line)=>line.remainingQuantity>0).map((line)=>[line.id,{quantity:"0",lot:"",expiry:"",cost:major(line.unitCostMinor)}]))); }).catch((e)=>setError(errorText(e))); return()=>{active=false}; },[id]);
  const selected=order?.lines.filter((line)=>Number(edits[line.id]?.quantity||0)>0)??[];
  const total=selected.reduce((sum,line)=>sum+Number(edits[line.id].quantity),0);
  const prepare=():PurchaseReceiptLineInput[]=>{
    if(!order) throw new Error("Purchase order is unavailable.");
    const lines=selected.map((line)=>{
      const edit=edits[line.id], quantity=Number(edit.quantity);
      if(!Number.isSafeInteger(quantity)||quantity<=0) throw new Error("Receive quantities must be positive whole numbers.");
      if(quantity>line.remainingQuantity) throw new Error(`Receive quantity for ${line.productNameSnapshot} exceeds the remaining quantity.`);
      return {purchaseOrderLineId:line.id,quantity,lotNumber:edit.lot.trim()||null,expiryDate:edit.expiry||null,acquisitionCostMinor:minor(edit.cost)};
    });
    if(!lines.length) throw new Error("Enter a receive quantity for at least one line.");
    return lines;
  };
  const openReview=()=>{ try{prepare();setError("");setReview(true);}catch(e){setError(errorText(e));} };
  const post=async()=>{
    if(!order||!branchId||lock.current)return; lock.current=true;setBusy(true);setError("");
    try{const receipt=await api.receive({id:receiptId.current,purchaseOrderId:order.id,expectedRevision:order.revision,requestId:requestId.current,branchId,lines:prepare()}); navigate(`/purchases/receipts/${receipt.id}`);}
    catch(e){setError(errorText(e));setReview(false);}finally{lock.current=false;setBusy(false);}
  };
  if(branchLoading||!order) return <div className="feature-page canonical-purchases"><BackLink to={`/purchases/${id}`} label="Purchase detail" />{error?<Alert title="Receiving could not be opened" tone="danger">{error}</Alert>:<Card><p role="status">Loading receiving workflow…</p></Card>}</div>;
  return <div className="feature-page canonical-purchases">
    <BackLink to={`/purchases/${id}`} label="Purchase detail" />
    <PageHeader title={`Receive ${order.purchaseNumber}`} description={`Post stock to ${branchName}. Purchase revision ${order.revision}.`} actions={<Button variant="primary" disabled={!selected.length||busy} onClick={openReview}>Review receipt</Button>} />
    {error&&<Alert title="Receipt could not be posted" tone="danger">{error}</Alert>}
    <Alert title="Inventory posting boundary" tone="info">Posting creates immutable canonical batches and PURCHASE_RECEIPT movements for branch {branchName}. It does not record supplier payment.</Alert>
    <Card><div className="purchase-lines-scroll"><table className="purchase-lines receipt-entry"><thead><tr><th>Package</th><th>Ordered</th><th>Received</th><th>Remaining</th><th>Receive now</th><th>Lot / batch</th><th>Expiry</th><th>Actual cost</th></tr></thead><tbody>{order.lines.map((line)=>{
      const edit=edits[line.id]; return <tr key={line.id}><td><strong>{line.productNameSnapshot}</strong><span>{line.packageLabelSnapshot}</span></td><td>{line.orderedQuantity}</td><td>{line.receivedQuantity}</td><td>{line.remainingQuantity}</td>{edit?<><td><Input hideLabel label={`Receive now ${line.productNameSnapshot}`} type="number" min={0} max={line.remainingQuantity} step={1} value={edit.quantity} onChange={(e)=>setEdits((all)=>({...all,[line.id]:{...edit,quantity:e.target.value}}))}/></td><td><Input hideLabel label={`Lot ${line.productNameSnapshot}`} value={edit.lot} onChange={(e)=>setEdits((all)=>({...all,[line.id]:{...edit,lot:e.target.value}}))}/></td><td><Input hideLabel label={`Expiry ${line.productNameSnapshot}`} type="date" value={edit.expiry} onChange={(e)=>setEdits((all)=>({...all,[line.id]:{...edit,expiry:e.target.value}}))}/></td><td><Input hideLabel label={`Acquisition cost ${line.productNameSnapshot}`} inputMode="decimal" value={edit.cost} onChange={(e)=>setEdits((all)=>({...all,[line.id]:{...edit,cost:e.target.value}}))}/></td></>:<td colSpan={4}>Fully received</td>}</tr>})}</tbody></table></div>
      <div className="purchase-totals"><span>Receiving branch <strong>{branchName}</strong></span><span>Selected allocations <strong>{selected.length}</strong></span><span>Total quantity <strong>{total}</strong></span></div>
    </Card>
    <Modal open={review} onOpenChange={(value)=>{if(!busy)setReview(value)}} title="Post purchase receipt?" description="This atomically creates or reuses Inventory batches, posts PURCHASE_RECEIPT movements, and advances the purchase status.">
      <div className="stack"><DetailList items={[{label:"Purchase",value:order.purchaseNumber},{label:"Branch",value:branchName},{label:"Allocations",value:selected.length},{label:"Total quantity",value:total}]} />
      <div className="row end"><Button disabled={busy} onClick={()=>setReview(false)}>Continue editing</Button><Button variant="primary" loading={busy} onClick={post}>Post receipt once</Button></div></div>
    </Modal>
  </div>;
}

function ReceiptDetails({id}:{id:string}) {
  const navigate=useNavigate();
  const [receipt,setReceipt]=useState<PurchaseReceipt|null>(null),[error,setError]=useState("");
  useEffect(()=>{let active=true;api.getReceipt(id).then((value)=>{if(active)setReceipt(value)}).catch((e)=>{if(active)setError(errorText(e))});return()=>{active=false}},[id]);
  if(!receipt)return <div className="feature-page canonical-purchases">{error?<Alert title="Receipt could not be loaded" tone="danger">{error}</Alert>:<Card><p role="status">Loading purchase receipt…</p></Card>}</div>;
  const canReturn=receipt.lines.some((line)=>line.remainingReturnableQuantity>0&&line.currentBatchQuantity>0);
  return <div className="feature-page canonical-purchases"><BackLink to={`/purchases/${receipt.purchaseOrderId}`} label="Purchase detail" />
    <PageHeader title={receipt.receiptNumber} description={`${receipt.purchaseNumber} · Posted ${stamp(receipt.postedAt)}`} actions={canReturn?<Button variant="primary" onClick={()=>navigate(`/purchases/receipts/${receipt.id}/return`)}>Return to supplier</Button>:undefined} />
    <Alert title="Receipt posted" tone="success">Inventory increased by {receipt.totalQuantity} package units through canonical PURCHASE_RECEIPT movements.</Alert>
    <Card><DetailList items={[{label:"Purchase",value:receipt.purchaseNumber},{label:"Supplier",value:`${receipt.supplierCode} · ${receipt.supplierName}`},{label:"Branch",value:`${receipt.branchCode} · ${receipt.branchName}`},{label:"Total quantity",value:receipt.totalQuantity},{label:"Posted by",value:receipt.createdBy||"—"},{label:"Posted",value:stamp(receipt.postedAt)}]} /></Card>
    <Card><Table label="Purchase receipt lines" rows={receipt.lines} rowKey={(line)=>line.id} columns={[
      {key:"package",header:"Package",render:(line)=><>{line.productNameSnapshot}<span className="cell-secondary">{line.packageLabelSnapshot}</span></>},
      {key:"quantity",header:"Quantity",render:(line)=>line.quantity},
      {key:"returned",header:"Returned",render:(line)=>line.returnedQuantity},
      {key:"returnable",header:"Returnable",render:(line)=>line.remainingReturnableQuantity},
      {key:"available",header:"Batch available",render:(line)=>line.currentBatchQuantity},
      {key:"lot",header:"Lot",render:(line)=>line.lotNumber||"No lot"},
      {key:"expiry",header:"Expiry",render:(line)=>line.expiryDate||"No expiry"},
      {key:"cost",header:"Acquisition cost",render:(line)=>money(line.acquisitionCostMinor)},
      {key:"trace",header:"Inventory trace",render:(line)=><><span>{line.batchId}</span><span className="cell-secondary">Movement {line.movementId}</span></>},
    ]}/></Card>
    <Card><h2>Related supplier returns</h2>{receipt.returns.length===0?<EmptyState title="No returns posted" description="This receipt has no supplier returns."/>:<Table label="Receipt return history" rows={receipt.returns} rowKey={(item)=>item.id} columns={[
      {key:"number",header:"Return",render:(item)=><Link className="text-link" to={`/purchases/returns/${item.id}`}>{item.returnNumber}</Link>},
      {key:"quantity",header:"Quantity",render:(item)=>item.totalQuantity},
      {key:"reason",header:"Reason",render:(item)=>item.reason},
      {key:"posted",header:"Posted",render:(item)=>stamp(item.postedAt)},
    ]}/>}</Card>
  </div>;
}

function ReturnEntry({receiptId}:{receiptId:string}) {
  const navigate=useNavigate();
  const [receipt,setReceipt]=useState<PurchaseReceipt|null>(null),[order,setOrder]=useState<PurchaseOrder|null>(null);
  const [quantities,setQuantities]=useState<Record<string,string>>({}),[reason,setReason]=useState(""),[error,setError]=useState(""),[review,setReview]=useState(false),[busy,setBusy]=useState(false);
  const lock=useRef(false),returnId=useRef(crypto.randomUUID()),requestId=useRef(crypto.randomUUID());
  useEffect(()=>{let active=true;api.getReceipt(receiptId).then(async(value)=>{const purchase=await api.get(value.purchaseOrderId);if(!active)return;setReceipt(value);setOrder(purchase);setQuantities(Object.fromEntries(value.lines.map((line)=>[line.id,"0"])))}).catch((e)=>{if(active)setError(errorText(e))});return()=>{active=false}},[receiptId]);
  const selected=receipt?.lines.filter((line)=>Number(quantities[line.id]||0)>0)??[];
  const prepare=()=>{
    if(!receipt)throw new Error("Purchase receipt is unavailable.");
    if(!reason.trim())throw new Error("Enter a supplier return reason.");
    const lines=selected.map((line)=>{const quantity=Number(quantities[line.id]);if(!Number.isSafeInteger(quantity)||quantity<=0)throw new Error("Return quantities must be positive whole numbers.");if(quantity>line.remainingReturnableQuantity)throw new Error(`Return quantity for ${line.productNameSnapshot} exceeds the historically returnable quantity.`);if(quantity>line.currentBatchQuantity)throw new Error(`Original batch stock is insufficient for ${line.productNameSnapshot}.`);return{receiptLineId:line.id,quantity}});
    if(!lines.length)throw new Error("Enter a return quantity for at least one eligible receipt line.");return lines;
  };
  const openReview=()=>{try{prepare();setError("");setReview(true)}catch(e){setError(errorText(e))}};
  const post=async()=>{if(!receipt||!order||lock.current)return;lock.current=true;setBusy(true);setError("");try{const result=await api.createReturn({id:returnId.current,purchaseOrderId:order.id,receiptId:receipt.id,expectedRevision:order.revision,requestId:requestId.current,reason:reason.trim(),lines:prepare()});navigate(`/purchases/returns/${result.id}`)}catch(e){setError(errorText(e));setReview(false)}finally{lock.current=false;setBusy(false)}};
  if(!receipt||!order)return <div className="feature-page canonical-purchases"><BackLink to={`/purchases/receipts/${receiptId}`} label="Receipt detail"/>{error?<Alert title="Purchase return could not be opened" tone="danger">{error}</Alert>:<Card><p role="status">Loading returnable receipt stock…</p></Card>}</div>;
  return <div className="feature-page canonical-purchases"><BackLink to={`/purchases/receipts/${receipt.id}`} label="Receipt detail"/>
    <PageHeader title={`Return from ${receipt.receiptNumber}`} description={`${receipt.supplierName} · ${receipt.branchName} · Purchase revision ${order.revision}`} actions={<Button variant="primary" disabled={!selected.length||!reason.trim()||busy} onClick={openReview}>Review return</Button>}/>
    {error&&<Alert title="Purchase return could not be posted" tone="danger">{error}</Alert>}
    <Alert title="Exact batch return" tone="info">Stock is removed only from each receipt line's original batch. No Catalog or alternate batch selection is available.</Alert>
    <Card><div className="purchase-lines-scroll"><table className="purchase-lines return-entry"><thead><tr><th>Package</th><th>Received</th><th>Returned</th><th>Returnable</th><th>Batch available</th><th>Return now</th><th>Lot / expiry</th><th>Acquisition cost</th></tr></thead><tbody>{receipt.lines.map((line)=>{const eligible=line.remainingReturnableQuantity>0&&line.currentBatchQuantity>0;return <tr key={line.id}><td><strong>{line.productNameSnapshot}</strong><span>{line.packageLabelSnapshot}</span></td><td>{line.quantity}</td><td>{line.returnedQuantity}</td><td>{line.remainingReturnableQuantity}</td><td>{line.currentBatchQuantity}</td><td>{eligible?<Input hideLabel label={`Return now ${line.productNameSnapshot}`} type="number" min={0} max={Math.min(line.remainingReturnableQuantity,line.currentBatchQuantity)} step={1} value={quantities[line.id]||"0"} onChange={(e)=>setQuantities((all)=>({...all,[line.id]:e.target.value}))}/>:"Unavailable"}</td><td>{line.lotNumber||"No lot"}<span>{line.expiryDate||"No expiry"}</span></td><td>{money(line.acquisitionCostMinor)}</td></tr>})}</tbody></table></div>
      <Textarea label="Return reason" required value={reason} onChange={(e)=>{setReason(e.target.value);requestId.current=crypto.randomUUID()}}/>
      <div className="purchase-totals"><span>Supplier <strong>{receipt.supplierName}</strong></span><span>Branch <strong>{receipt.branchName}</strong></span><span>Selected lines <strong>{selected.length}</strong></span><span>Total return <strong>{selected.reduce((sum,line)=>sum+Number(quantities[line.id]),0)}</strong></span></div>
    </Card>
    <Modal open={review} onOpenChange={(value)=>{if(!busy)setReview(value)}} title="Post purchase return?" description="This immutable return subtracts stock from the exact original batches through canonical PURCHASE_RETURN movements."><div className="stack"><DetailList items={[{label:"Purchase",value:receipt.purchaseNumber},{label:"Receipt",value:receipt.receiptNumber},{label:"Supplier",value:receipt.supplierName},{label:"Branch",value:receipt.branchName},{label:"Reason",value:reason.trim()},{label:"Total quantity",value:selected.reduce((sum,line)=>sum+Number(quantities[line.id]),0)}]}/><div className="row end"><Button disabled={busy} onClick={()=>setReview(false)}>Continue editing</Button><Button variant="danger" loading={busy} onClick={post}>Post return once</Button></div></div></Modal>
  </div>;
}

function ReturnDetails({id}:{id:string}) {
  const [value,setValue]=useState<PurchaseReturn|null>(null),[error,setError]=useState("");
  useEffect(()=>{let active=true;api.getReturn(id).then((result)=>{if(active)setValue(result)}).catch((e)=>{if(active)setError(errorText(e))});return()=>{active=false}},[id]);
  if(!value)return <div className="feature-page canonical-purchases">{error?<Alert title="Purchase return could not be loaded" tone="danger">{error}</Alert>:<Card><p role="status">Loading purchase return…</p></Card>}</div>;
  return <div className="feature-page canonical-purchases"><BackLink to={`/purchases/receipts/${value.receiptId}`} label="Original receipt"/><PageHeader title={value.returnNumber} description={`${value.purchaseNumber} · Posted ${stamp(value.postedAt)}`}/><Alert title="Supplier return posted" tone="success">Inventory decreased by {value.totalQuantity} package units through canonical PURCHASE_RETURN movements.</Alert>
    <Card><DetailList items={[{label:"Purchase",value:value.purchaseNumber},{label:"Original receipt",value:value.receiptNumber},{label:"Supplier",value:`${value.supplierCode} · ${value.supplierName}`},{label:"Branch",value:`${value.branchCode} · ${value.branchName}`},{label:"Reason",value:value.reason},{label:"Total quantity",value:value.totalQuantity},{label:"Source revision",value:value.sourcePurchaseRevision},{label:"Posted by",value:value.createdBy||"—"},{label:"Posted",value:stamp(value.postedAt)}]}/></Card>
    <Card><Table label="Purchase return lines" rows={value.lines} rowKey={(line)=>line.id} columns={[
      {key:"package",header:"Package",render:(line)=><>{line.productNameSnapshot}<span className="cell-secondary">{line.packageLabelSnapshot}</span></>},
      {key:"quantity",header:"Returned",render:(line)=>line.quantity},{key:"source",header:"Original received",render:(line)=>line.receivedQuantity},{key:"prior",header:"Previously returned",render:(line)=>line.previouslyReturnedQuantity},{key:"remaining",header:"Remaining returnable",render:(line)=>line.remainingReturnableQuantity},
      {key:"lot",header:"Lot / expiry",render:(line)=><>{line.lotNumber||"No lot"}<span className="cell-secondary">{line.expiryDate||"No expiry"}</span></>},{key:"cost",header:"Acquisition cost",render:(line)=>money(line.acquisitionCostMinor)},{key:"trace",header:"Inventory trace",render:(line)=><>{line.batchId}<span className="cell-secondary">Movement {line.movementId}</span></>},
    ]}/></Card></div>;
}

function ReturnHistory() {
  const [search,setSearch]=useState(""),[page,setPage]=useState(0),[attempt,setAttempt]=useState(0),[data,setData]=useState<{key:string;items:PurchaseReturnSummary[];total:number}|null>(null),[error,setError]=useState("");
  const key=JSON.stringify([search,page,attempt]);
  useEffect(()=>{let active=true;const timer=setTimeout(()=>api.listReturns({search:search||null,limit:size,offset:page*size}).then((result)=>{if(active){setData({key,items:result.items,total:result.total});setError("")}}).catch((e)=>{if(active){setData(null);setError(errorText(e))}}),search?180:0);return()=>{active=false;clearTimeout(timer)}},[attempt,key,page,search]);
  const current=data?.key===key?data:null;
  return <div className="feature-page canonical-purchases"><BackLink to="/purchases" label="Purchases"/><PageHeader title="Purchase return history" description="Immutable supplier returns posted against canonical receipt batches."/>{error&&<Alert title="Returns could not be loaded" tone="danger">{error} <Button size="sm" onClick={()=>setAttempt((v)=>v+1)}>Retry</Button></Alert>}<TableToolbar><SearchInput label="Search purchase returns" placeholder="Return, purchase, receipt or reason" value={search} onChange={(e)=>{setSearch(e.target.value);setPage(0)}}/></TableToolbar>
    {!current?<Card><p role="status">Loading purchase returns…</p></Card>:current.items.length===0?<EmptyState title={search?"No returns match this search":"No purchase returns posted"} description={search?"Change the return search.":"Eligible returns begin from posted purchase receipts."}/>:<Card><Table label="Purchase returns" rows={current.items} rowKey={(item)=>item.id} columns={[
      {key:"number",header:"Return",render:(item)=><Link className="text-link" to={`/purchases/returns/${item.id}`}>{item.returnNumber}<span className="cell-secondary">{stamp(item.postedAt)}</span></Link>},{key:"purchase",header:"Purchase / receipt",render:(item)=><>{item.purchaseNumber}<span className="cell-secondary">{item.receiptNumber}</span></>},{key:"supplier",header:"Supplier",render:(item)=><>{item.supplierName}<span className="cell-secondary">{item.supplierCode}</span></>},{key:"branch",header:"Branch",render:(item)=>item.branchName},{key:"quantity",header:"Quantity",render:(item)=>item.totalQuantity},{key:"reason",header:"Reason",render:(item)=>item.reason},
    ]}/><div className="grid-footer"><Button disabled={page===0} onClick={()=>setPage((v)=>v-1)}>Previous</Button><span>{current.total} total · Page {page+1}</span><Button disabled={(page+1)*size>=current.total} onClick={()=>setPage((v)=>v+1)}>Next</Button></div></Card>}
  </div>;
}
