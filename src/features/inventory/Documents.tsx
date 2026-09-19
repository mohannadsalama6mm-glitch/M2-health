import { InventoryNavigation } from "./InventoryWorkspace";
import { useEffect, useRef, useState } from "react";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";
import {
  Alert,
  Button,
  Card,
  Input,
  Select,
  Textarea,
  PageHeader,
  Table,
  Modal,
  LinkButton,
} from "../../design-system";
import { useBranch } from "../../app/BranchContext";
import { isDesktopRuntime } from "../../lib/tauri/client";
import {
  listProducts,
  getProductDetail,
  toMinor,
} from "../../lib/tauri/catalog";
import type {
  ProductListItem,
  PackageDetail,
} from "../../lib/tauri/catalog.types";
import {
  saveDocument,
  getDocument,
  listDocuments,
  readyDocument,
  postDocument,
  cancelDocument,
  reverseDocument,
  workflowBatches,
  documentRequestKey,
  type InventoryDocument,
  type DocumentHeader,
  type SaveDocument,
  type DocumentKind,
  type DocumentStatus,
  type DocumentLineInput,
  type LineAction,
  type BatchChoice,
} from "../../lib/tauri/inventory-documents";

const kinds: [DocumentKind, string][] = [
  ["RECEIPT", "Receive Stock"],
  ["ADJUSTMENT", "Adjust Stock"],
  ["WRITE_OFF", "Damage / Expiry Write-Off"],
  ["TRANSFER", "Transfer Stock"],
];
const message = (e: unknown) =>
  e instanceof Error ? e.message : "The operation could not be completed.";
const blank = (kind: DocumentKind, branchId: string): SaveDocument => ({
  documentId: null,
  expectedRevision: null,
  kind,
  branchId,
  destinationBranchId: null,
  referenceText: null,
  supplierReference: null,
  notes: null,
  createdBy: null,
  lines: [],
});
function lineInput(l: DocumentLineInput): DocumentLineInput {
  return {
    productPackageId: l.productPackageId,
    batchId: l.batchId,
    action: l.action,
    quantity: l.quantity,
    reason: l.reason,
    lotNumber: l.lotNumber,
    expiryDate: l.expiryDate,
    costPriceMinor: l.costPriceMinor,
    supplierReference: l.supplierReference,
  };
}
function draft(d: InventoryDocument): SaveDocument {
  return {
    ...blank(d.kind, d.branchId),
    documentId: d.id,
    expectedRevision: d.revision,
    destinationBranchId: d.destinationBranchId,
    referenceText: d.referenceText,
    supplierReference: d.supplierReference,
    notes: d.notes,
    createdBy: d.createdBy,
    lines: d.lines.map(lineInput),
  };
}

export function InventoryDocuments() {
  const [context] = useSearchParams();
  return <InventoryDocumentsScope key={context.toString()} />;
}
function InventoryDocumentsScope() {
  const [context] = useSearchParams();
  const { id } = useParams();
  const navigate = useNavigate();
  const { branchId, loading } = useBranch();
  const [kind, setKind] = useState<DocumentKind | "">("");
  const [status, setStatus] = useState<DocumentStatus | "">("");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [page, setPage] = useState(0);
  const [rows, setRows] = useState<DocumentHeader[]>([]);
  const [total, setTotal] = useState(0);
  const [document, setDocument] = useState<InventoryDocument | null>(null);
  const [newKind, setNewKind] = useState<DocumentKind | null>(() =>
    kinds.some(([k]) => k === context.get("new"))
      ? (context.get("new") as DocumentKind)
      : null,
  );
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);
  const requestKey = JSON.stringify([
    id,
    branchId,
    kind,
    status,
    from,
    to,
    page,
    attempt,
  ]);
  const [loadedKey, setLoadedKey] = useState("");
  useEffect(() => {
    if (!isDesktopRuntime() || loading || !branchId) return;
    let active = true;
    const request = id
      ? getDocument(id).then((d) => {
          if (active) setDocument(d);
        })
      : listDocuments({
          branchId,
          kind: kind || null,
          status: status || null,
          dateFrom: from || null,
          dateTo: to || null,
          limit: 20,
          offset: page * 20,
        }).then((p) => {
          if (active) {
            setRows(p.items);
            setTotal(p.total);
          }
        });
    request
      .then(() => {
        if (active) {
          setError("");
          setLoadedKey(requestKey);
        }
      })
      .catch((e) => {
        if (active) setError(message(e));
      });
    return () => {
      active = false;
    };
  }, [
    id,
    branchId,
    loading,
    kind,
    status,
    from,
    to,
    page,
    attempt,
    requestKey,
  ]);
  if (!isDesktopRuntime())
    return (
      <Alert title="Inventory documents require the desktop app" tone="info">
        Open M² Health desktop to manage local stock documents.
      </Alert>
    );
  const current = !error && id && document?.id === id ? document : null;
  const back = () => {
    setNewKind(null);
    setDocument(null);
    navigate("/inventory/documents");
    setAttempt((a) => a + 1);
  };
  return (
    <div className="stack">
      <PageHeader
        title="Inventory Documents"
        description="Receive, adjust, write off and transfer stock with a permanent document history."
        actions={<LinkButton to="/inventory">Inventory balances</LinkButton>}
      />
      <InventoryNavigation />
      {error && (
        <Alert title="Unable to load documents" tone="danger">
          {error}
          <Button onClick={() => setAttempt((a) => a + 1)}>Retry</Button>
        </Alert>
      )}
      {newKind || current ? (
        <DocumentEditor
          key={current ? `${current.id}:${current.revision}` : `new:${newKind}`}
          document={current}
          initial={current ? draft(current) : blank(newKind!, branchId)}
          onBack={back}
          onSaved={(d) => {
            setDocument(d);
            setNewKind(null);
            navigate(`/inventory/documents/${d.id}`);
          }}
        />
      ) : id ? (
        <p role="status">Loading document…</p>
      ) : (
        <>
          <div className="row wrap">
            {kinds.map(([k, label]) => (
              <Button
                key={k}
                disabled={loading || !branchId}
                onClick={() => setNewKind(k)}
              >
                {label}
              </Button>
            ))}
          </div>
          <Card>
            <div className="row wrap">
              <Select
                label="Document type"
                value={kind}
                onChange={(e) => {
                  setKind(e.target.value as DocumentKind | "");
                  setPage(0);
                }}
              >
                <option value="">All types</option>
                {[...kinds, ["REVERSAL", "Reversals"]].map(([k, label]) => (
                  <option key={k} value={k}>
                    {label}
                  </option>
                ))}
              </Select>
              <Select
                label="Document status"
                value={status}
                onChange={(e) => {
                  setStatus(e.target.value as DocumentStatus | "");
                  setPage(0);
                }}
              >
                <option value="">All statuses</option>
                {["DRAFT", "READY", "POSTED", "CANCELLED"].map((s) => (
                  <option key={s}>{s}</option>
                ))}
              </Select>
              <Input
                label="From date"
                type="date"
                value={from}
                onChange={(e) => {
                  setFrom(e.target.value);
                  setPage(0);
                }}
              />
              <Input
                label="To date"
                type="date"
                value={to}
                onChange={(e) => {
                  setTo(e.target.value);
                  setPage(0);
                }}
              />
            </div>
            {loadedKey !== requestKey && !error && (
              <p role="status">Loading documents…</p>
            )}
            <Table
              label="Inventory documents"
              rows={!error && loadedKey === requestKey ? rows : []}
              rowKey={(r) => r.id}
              columns={[
                {
                  key: "number",
                  header: "Document",
                  render: (r) => (
                    <Button
                      variant="ghost"
                      onClick={() => navigate(`/inventory/documents/${r.id}`)}
                    >
                      {r.documentNumber}
                    </Button>
                  ),
                },
                { key: "kind", header: "Type", render: (r) => r.kind },
                { key: "status", header: "Status", render: (r) => r.status },
                { key: "date", header: "Created", render: (r) => r.createdAt },
              ]}
            />
            <div className="grid-footer">
              <span>{total} documents · local SQLite</span>
              <Button disabled={page === 0} onClick={() => setPage(page - 1)}>
                Previous documents
              </Button>
              <span>Page {page + 1}</span>
              <Button
                disabled={(page + 1) * 20 >= total}
                onClick={() => setPage(page + 1)}
              >
                Next documents
              </Button>
            </div>
          </Card>
        </>
      )}
    </div>
  );
}

function DocumentEditor({
  document,
  initial,
  onBack,
  onSaved,
}: {
  document: InventoryDocument | null;
  initial: SaveDocument;
  onBack: () => void;
  onSaved: (d: InventoryDocument) => void;
}) {
  const { branches } = useBranch();
  const [form, setForm] = useState(initial);
  const [dirty, setDirty] = useState(!document);
  const [labels, setLabels] = useState<Record<string, string>>(() =>
    Object.fromEntries(
      document?.lines.map((l) => [
        l.productPackageId,
        `${l.productLabel} · ${l.packageLabel}`,
      ]) ?? [],
    ),
  );
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const locked = useRef(false);
  const [confirm, setConfirm] = useState<"post" | "cancel" | "reverse" | null>(
    null,
  );
  const [reason, setReason] = useState("");
  const editable = !document || document.status === "DRAFT";
  const change = (patch: Partial<SaveDocument>) => {
    setForm((f) => ({ ...f, ...patch }));
    setDirty(true);
  };
  const run = async (work: () => Promise<InventoryDocument>) => {
    if (locked.current) return;
    locked.current = true;
    setBusy(true);
    setError("");
    try {
      onSaved(await work());
    } catch (e) {
      setError(message(e));
    } finally {
      locked.current = false;
      setBusy(false);
    }
  };
  const perform = () => {
    if (!document) return;
    const action = confirm;
    setConfirm(null);
    void run(() =>
      action === "post"
        ? postDocument(
            document.id,
            document.revision,
            documentRequestKey("post", document.id, document.revision),
          )
        : action === "cancel"
          ? cancelDocument(document.id, document.revision)
          : reverseDocument(
              document.id,
              documentRequestKey(
                "reverse",
                document.id,
                document.revision,
                reason,
              ),
              reason,
            ),
    );
  };
  return (
    <Card>
      <div className="stack">
        <div className="row between">
          <h2>
            {document?.documentNumber ??
              kinds.find(([k]) => k === form.kind)?.[1]}
          </h2>
          <strong>{document?.status ?? "Unsaved draft"}</strong>
          <Button disabled={busy} onClick={onBack}>
            Back to documents
          </Button>
        </div>
        {error && (
          <Alert title="Document action failed" tone="danger">
            {error}
          </Alert>
        )}
        {document?.status === "POSTED" && (
          <Alert title="Posted document is read-only" tone="success">
            Stock changes and movement links are saved. Corrections require a
            reversal.
          </Alert>
        )}
        {document?.status === "CANCELLED" && (
          <Alert title="Cancelled document" tone="info">
            This document cannot be posted or edited. No stock was changed.
          </Alert>
        )}
        <div className="row wrap">
          <Select
            label="Source branch"
            disabled={!!document || busy}
            value={form.branchId}
            onChange={(e) => change({ branchId: e.target.value, lines: [] })}
          >
            {branches
              .filter((b) => b.isActive)
              .map((b) => (
                <option key={b.id} value={b.id}>
                  {b.name}
                </option>
              ))}
          </Select>
          {form.kind === "TRANSFER" && (
            <Select
              label="Destination branch"
              disabled={!editable || busy}
              value={form.destinationBranchId ?? ""}
              onChange={(e) =>
                change({ destinationBranchId: e.target.value || null })
              }
            >
              <option value="">Select destination</option>
              {branches
                .filter((b) => b.isActive && b.id !== form.branchId)
                .map((b) => (
                  <option key={b.id} value={b.id}>
                    {b.name}
                  </option>
                ))}
            </Select>
          )}
          <Input
            label="Reference"
            disabled={!editable || busy}
            value={form.referenceText ?? ""}
            onChange={(e) => change({ referenceText: e.target.value || null })}
          />
          {form.kind === "RECEIPT" && (
            <Input
              label="Supplier reference"
              disabled={!editable || busy}
              value={form.supplierReference ?? ""}
              onChange={(e) =>
                change({ supplierReference: e.target.value || null })
              }
            />
          )}
        </div>
        <Textarea
          label="Notes"
          disabled={!editable || busy}
          value={form.notes ?? ""}
          onChange={(e) => change({ notes: e.target.value || null })}
        />
        {editable && (
          <LineEntry
            key={`${form.kind}:${form.branchId}`}
            kind={form.kind}
            branchId={form.branchId}
            disabled={busy}
            onAdd={(line, label) => {
              change({ lines: [...form.lines, line] });
              setLabels((l) => ({ ...l, [line.productPackageId]: label }));
            }}
          />
        )}
        <Table
          label="Document lines"
          rows={form.lines.map((l, position) => ({ ...l, position }))}
          rowKey={(r) => String(r.position)}
          columns={[
            {
              key: "product",
              header: "Package",
              render: (r) => labels[r.productPackageId] ?? "Selected package",
            },
            { key: "action", header: "Action", render: (r) => r.action },
            {
              key: "quantity",
              header: "Quantity",
              render: (r) =>
                editable ? (
                  <Input
                    label={`Line ${r.position + 1} quantity`}
                    type="number"
                    min="1"
                    step="1"
                    disabled={busy}
                    value={r.quantity}
                    onChange={(e) =>
                      change({
                        lines: form.lines.map((l, n) =>
                          n === r.position
                            ? { ...l, quantity: Number(e.target.value) }
                            : l,
                        ),
                      })
                    }
                  />
                ) : (
                  r.quantity
                ),
            },
            {
              key: "lot",
              header: "Lot / expiry",
              render: (r) =>
                `${r.lotNumber ?? "Unspecified"} · ${r.expiryDate ?? document?.preview[r.position]?.expiryDate ?? "Unknown expiry"}`,
            },
            {
              key: "reason",
              header: "Reason",
              render: (r) =>
                editable ? (
                  <Input
                    label={`Line ${r.position + 1} reason`}
                    disabled={busy}
                    value={r.reason}
                    onChange={(e) =>
                      change({
                        lines: form.lines.map((l, n) =>
                          n === r.position
                            ? { ...l, reason: e.target.value }
                            : l,
                        ),
                      })
                    }
                  />
                ) : (
                  r.reason
                ),
            },
            {
              key: "preview",
              header: "Current → projected",
              render: (r) =>
                !dirty &&
                document &&
                document.status !== "POSTED" &&
                document.status !== "CANCELLED"
                  ? `${document.preview[r.position]?.currentStock ?? "—"} → ${document.preview[r.position]?.projectedStock ?? "—"}`
                  : "—",
            },
            ...(editable
              ? [
                  {
                    key: "remove",
                    header: "Actions",
                    render: (r: DocumentLineInput & { position: number }) => (
                      <Button
                        disabled={busy}
                        onClick={() =>
                          change({
                            lines: form.lines.filter(
                              (_, n) => n !== r.position,
                            ),
                          })
                        }
                      >
                        Remove line {r.position + 1}
                      </Button>
                    ),
                  },
                ]
              : []),
          ]}
        />
        {dirty && (
          <p className="muted">
            Save the draft to refresh the backend stock preview before marking
            ready.
          </p>
        )}
        <div className="row wrap">
          {editable && (
            <Button
              variant="primary"
              loading={busy}
              onClick={() => void run(() => saveDocument(form))}
            >
              Save draft
            </Button>
          )}
          {document?.status === "DRAFT" && (
            <Button
              disabled={dirty || busy}
              onClick={() =>
                void run(() => readyDocument(document.id, document.revision))
              }
            >
              Mark ready
            </Button>
          )}
          {document?.status === "READY" && (
            <Button
              variant="primary"
              disabled={busy}
              onClick={() => setConfirm("post")}
            >
              Post document
            </Button>
          )}
          {document && ["DRAFT", "READY"].includes(document.status) && (
            <Button disabled={busy} onClick={() => setConfirm("cancel")}>
              Cancel document
            </Button>
          )}
        </div>
        {!!document?.movements.length && (
          <Table
            label="Document movement links"
            rows={document.movements}
            rowKey={(r) => r.movementId}
            columns={[
              { key: "id", header: "Movement ID", render: (r) => r.movementId },
              {
                key: "type",
                header: "Movement",
                render: (r) => r.movementType,
              },
              { key: "qty", header: "Change", render: (r) => r.quantityDelta },
              {
                key: "branch",
                header: "Branch",
                render: (r) =>
                  branches.find((b) => b.id === r.branchId)?.name ?? r.branchId,
              },
              {
                key: "original",
                header: "Reverses movement",
                render: (r) => r.reversesMovementId ?? "—",
              },
            ]}
          />
        )}
        {document?.reversalOf && (
          <p>
            Reversal of{" "}
            <LinkButton to={`/inventory/documents/${document.reversalOf}`}>
              Original document
            </LinkButton>{" "}
            · {document.reversalReason}
          </p>
        )}
        {document?.status === "POSTED" && document.kind !== "REVERSAL" && (
          <div className="row wrap">
            <Input
              label="Reversal reason"
              value={reason}
              onChange={(e) => setReason(e.target.value)}
            />
            <Button
              disabled={busy || !reason.trim()}
              onClick={() => setConfirm("reverse")}
            >
              Reverse document
            </Button>
          </div>
        )}
        <Modal
          open={confirm !== null}
          onOpenChange={(open) => {
            if (!open) setConfirm(null);
          }}
          title={
            confirm === "post"
              ? "Post inventory document?"
              : confirm === "cancel"
                ? "Cancel inventory document?"
                : "Reverse inventory document?"
          }
          description={
            confirm === "cancel"
              ? "The document will become read-only without changing stock."
              : "This creates permanent stock movements. The backend will recheck stock and eligibility."
          }
        >
          <div className="row end">
            <Button onClick={() => setConfirm(null)}>Back</Button>
            <Button variant="primary" disabled={busy} onClick={perform}>
              Confirm
            </Button>
          </div>
        </Modal>
      </div>
    </Card>
  );
}

function LineEntry({
  kind,
  branchId,
  disabled,
  onAdd,
}: {
  kind: DocumentKind;
  branchId: string;
  disabled: boolean;
  onAdd: (line: DocumentLineInput, label: string) => void;
}) {
  const [context] = useSearchParams();
  const [search, setSearch] = useState("");
  const [products, setProducts] = useState<ProductListItem[]>([]);
  const [product, setProduct] = useState(context.get("product") || "");
  const [packages, setPackages] = useState<PackageDetail[]>([]);
  const [packageId, setPackageId] = useState(context.get("package") || "");
  const [batches, setBatches] = useState<BatchChoice[]>([]);
  const [batchId, setBatchId] = useState(context.get("batch") || "");
  const [batchOffset, setBatchOffset] = useState(0);
  const [more, setMore] = useState(false);
  const [action, setAction] = useState<LineAction>(
    kind === "RECEIPT"
      ? "RECEIVE"
      : kind === "TRANSFER"
        ? "TRANSFER"
        : kind === "WRITE_OFF"
          ? context.get("action") === "EXPIRY"
            ? "EXPIRY"
            : "DAMAGE"
          : "OUT",
  );
  const [quantity, setQuantity] = useState("1");
  const [reason, setReason] = useState("");
  const [lot, setLot] = useState("");
  const [expiry, setExpiry] = useState("");
  const [cost, setCost] = useState("");
  const [supplier, setSupplier] = useState("");
  const [error, setError] = useState("");
  useEffect(() => {
    if (!search.trim()) return;
    let active = true;
    const timer = setTimeout(() => {
      listProducts({ search, includeInactive: false, limit: 10, offset: 0 })
        .then((p) => {
          if (active) setProducts(p.items);
        })
        .catch((e) => {
          if (active) setError(message(e));
        });
    }, 200);
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [search]);
  useEffect(() => {
    if (!product) return;
    let active = true;
    getProductDetail(product)
      .then((d) => {
        if (active) setPackages(d.packages.filter((p) => p.package.isActive));
      })
      .catch((e) => {
        if (active) setError(message(e));
      });
    return () => {
      active = false;
    };
  }, [product]);
  useEffect(() => {
    if (!packageId || kind === "RECEIPT") return;
    let active = true;
    workflowBatches(branchId, packageId, batchOffset)
      .then((rows) => {
        if (active) {
          setBatches((old) => (batchOffset ? [...old, ...rows] : rows));
          setMore(rows.length === 50);
        }
      })
      .catch((e) => {
        if (active) setError(message(e));
      });
    return () => {
      active = false;
    };
  }, [branchId, packageId, kind, batchOffset]);
  const selected = batches.find((b) => b.id === batchId);
  const add = () => {
    const n = Number(quantity);
    const minor = cost.trim() ? toMinor(cost) : null;
    if (
      !packageId ||
      !Number.isSafeInteger(n) ||
      n <= 0 ||
      !reason.trim() ||
      (kind !== "RECEIPT" && !batchId) ||
      (cost.trim() && minor === null)
    ) {
      setError(
        "Select a package and batch where required, enter a positive whole quantity, a reason and a valid cost.",
      );
      return;
    }
    onAdd(
      {
        productPackageId: packageId,
        batchId: kind === "RECEIPT" ? null : batchId,
        action,
        quantity: n,
        reason,
        lotNumber: lot || null,
        expiryDate: expiry || null,
        costPriceMinor: minor,
        supplierReference: supplier || null,
      },
      `${products.find((p) => p.id === product)?.nameEn ?? products.find((p) => p.id === product)?.nameAr ?? "Product"} · ${packages.find((p) => p.package.id === packageId)?.package.packageLabel ?? "Package"}`,
    );
    setError("");
  };
  return (
    <div className="stack">
      <h3>Add document line</h3>
      {error && (
        <Alert title="Line could not be added" tone="danger">
          {error}
        </Alert>
      )}
      <div className="row wrap">
        <Input
          label="Search packages"
          value={search}
          disabled={disabled}
          onChange={(e) => setSearch(e.target.value)}
        />
        <Select
          label="Product"
          value={product}
          disabled={disabled}
          onChange={(e) => {
            setProduct(e.target.value);
            setPackages([]);
            setPackageId("");
            setBatches([]);
            setBatchId("");
          }}
        >
          <option value="">Select product</option>
          {product && !products.some((p) => p.id === product) && (
            <option value={product}>Selected Catalog product</option>
          )}
          {products.map((p) => (
            <option key={p.id} value={p.id}>
              {p.nameEn ?? p.nameAr ?? p.scientificName}
            </option>
          ))}
        </Select>
        <Select
          label="Package"
          value={packageId}
          disabled={disabled}
          onChange={(e) => {
            setPackageId(e.target.value);
            setBatchId("");
            setBatches([]);
            setBatchOffset(0);
          }}
        >
          <option value="">Select package</option>
          {packages.map((p) => (
            <option key={p.package.id} value={p.package.id}>
              {p.package.packageLabel}
            </option>
          ))}
        </Select>
        {kind !== "RECEIPT" && (
          <Select
            label="Batch"
            value={batchId}
            disabled={disabled}
            onChange={(e) => setBatchId(e.target.value)}
          >
            <option value="">Select batch</option>
            {batches.map((b) => (
              <option key={b.id} value={b.id}>
                {b.lotNumber ?? "Unspecified lot"} ·{" "}
                {b.expiryDate ?? "Unknown expiry"} · {b.quantity} available
              </option>
            ))}
          </Select>
        )}
        {kind !== "RECEIPT" && more && (
          <Button onClick={() => setBatchOffset((v) => v + 50)}>
            More batches
          </Button>
        )}
        {kind === "ADJUSTMENT" && (
          <Select
            label="Adjustment direction"
            value={action}
            onChange={(e) => setAction(e.target.value as LineAction)}
          >
            <option value="IN">IN</option>
            <option value="OUT">OUT</option>
          </Select>
        )}
        {kind === "WRITE_OFF" && (
          <Select
            label="Write-off type"
            value={action}
            onChange={(e) => setAction(e.target.value as LineAction)}
          >
            <option value="DAMAGE">Damage</option>
            <option value="EXPIRY">Expiry</option>
          </Select>
        )}
        <Input
          label="Quantity"
          type="number"
          min="1"
          step="1"
          value={quantity}
          disabled={disabled}
          onChange={(e) => setQuantity(e.target.value)}
        />
        <Input
          label="Line reason"
          value={reason}
          disabled={disabled}
          onChange={(e) => setReason(e.target.value)}
        />
      </div>
      {selected && (
        <p>
          Available stock: {selected.quantity} · Expiry date:{" "}
          {selected.expiryDate ?? "Unknown"}. Eligibility is checked by the
          backend.
        </p>
      )}
      {kind === "RECEIPT" && (
        <div className="row wrap">
          <Input
            label="Lot number"
            value={lot}
            onChange={(e) => setLot(e.target.value)}
          />
          <Input
            label="Expiry date"
            type="date"
            value={expiry}
            onChange={(e) => setExpiry(e.target.value)}
          />
          <Input
            label="Unit cost (EGP)"
            inputMode="decimal"
            value={cost}
            onChange={(e) => setCost(e.target.value)}
          />
          <Input
            label="Line supplier reference"
            value={supplier}
            onChange={(e) => setSupplier(e.target.value)}
          />
        </div>
      )}
      <Button disabled={disabled} onClick={add}>
        Add line
      </Button>
    </div>
  );
}
