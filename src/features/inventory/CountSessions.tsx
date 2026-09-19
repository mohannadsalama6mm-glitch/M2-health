import { InventoryNavigation } from "./InventoryWorkspace";
import { useEffect, useRef, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
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
import { listProducts, getProductDetail } from "../../lib/tauri/catalog";
import type {
  ProductListItem,
  PackageDetail,
} from "../../lib/tauri/catalog.types";
import {
  workflowBatches,
  type BatchChoice,
} from "../../lib/tauri/inventory-documents";
import {
  createCount,
  getCount,
  listCounts,
  startCount,
  recordCount,
  discoverCount,
  reviewCount,
  resumeCount,
  cancelCount,
  completeCount,
  countTransition,
  type CountStatus,
  type CountDetail,
  type CountSession,
  type CountSummary,
  type CountLine,
  type CountEntry,
  type CountDiscovery,
} from "../../lib/tauri/stock-counts";
const message = (e: unknown) =>
  e instanceof Error
    ? e.message
    : "The count operation could not be completed.";
export function CountSessions() {
  const { id } = useParams();
  const navigate = useNavigate();
  const { branchId, branches, loading } = useBranch();
  const [status, setStatus] = useState<CountStatus | "">("");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [page, setPage] = useState(0);
  const [attempt, setAttempt] = useState(0);
  const [rows, setRows] = useState<
    (CountSession & { summary: CountSummary })[]
  >([]);
  const [total, setTotal] = useState(0);
  const [data, setData] = useState<CountDetail | null>(null);
  const [error, setError] = useState("");
  const [creating, setCreating] = useState(false);
  const [notes, setNotes] = useState("");
  const [busy, setBusy] = useState(false);
  const lock = useRef(false);
  const queryKey = JSON.stringify([
    id,
    branchId,
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
    const p = id
      ? getCount(id).then((d) => {
          if (active) setData(d);
        })
      : listCounts({
          branchId,
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
    p.then(() => {
      if (active) {
        setError("");
        setLoadedKey(queryKey);
      }
    }).catch((e) => {
      if (active) setError(message(e));
    });
    return () => {
      active = false;
    };
  }, [id, branchId, loading, status, from, to, page, attempt, queryKey]);
  const create = async () => {
    if (lock.current) return;
    lock.current = true;
    setBusy(true);
    try {
      const d = await createCount(branchId, notes || null);
      setData(d);
      setCreating(false);
      navigate(`/inventory/count-sessions/${d.session.id}`);
    } catch (e) {
      setError(message(e));
    } finally {
      lock.current = false;
      setBusy(false);
    }
  };
  if (!isDesktopRuntime())
    return (
      <Alert title="Stock counts require the desktop app" tone="info">
        Open the local desktop workspace to count stock.
      </Alert>
    );
  return (
    <div className="stack">
      <InventoryNavigation />
      <PageHeader
        title="Stock Counts"
        description="Capture a fixed stock snapshot, enter physical package counts and review live corrections."
        actions={<LinkButton to="/inventory">Inventory balances</LinkButton>}
      />
      {error && (
        <Alert title="Stock count could not be loaded" tone="danger">
          {error}
          <Button onClick={() => setAttempt((a) => a + 1)}>Retry</Button>
        </Alert>
      )}
      {id ? (
        !error && data?.session.id === id ? (
          <CountWorkspace key={id} initial={data} />
        ) : (
          <p role="status">Loading count…</p>
        )
      ) : (
        <>
          <Button
            onClick={() => setCreating(true)}
            disabled={loading || !branchId}
          >
            New count
          </Button>
          <Modal
            open={creating}
            onOpenChange={setCreating}
            title="New stock count"
            description="ALL_INVENTORY captures active batches, including zero-stock batches, when you start. No stock changes occur until completion."
          >
            <p>Branch: {branches.find((b) => b.id === branchId)?.name}</p>
            <Textarea
              label="Count notes"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
            />
            <Button loading={busy} onClick={() => void create()}>
              Create draft count
            </Button>
          </Modal>
          <Card>
            <div className="row wrap">
              <Select
                label="Count status"
                value={status}
                onChange={(e) => {
                  setStatus(e.target.value as CountStatus | "");
                  setPage(0);
                }}
              >
                <option value="">All statuses</option>
                {[
                  "DRAFT",
                  "IN_PROGRESS",
                  "REVIEW",
                  "COMPLETED",
                  "CANCELLED",
                ].map((s) => (
                  <option key={s}>{s}</option>
                ))}
              </Select>
              <Input
                label="Count from date"
                type="date"
                value={from}
                onChange={(e) => {
                  setFrom(e.target.value);
                  setPage(0);
                }}
              />
              <Input
                label="Count to date"
                type="date"
                value={to}
                onChange={(e) => {
                  setTo(e.target.value);
                  setPage(0);
                }}
              />
            </div>
            <Table
              label="Stock count sessions"
              rows={!error && loadedKey === queryKey ? rows : []}
              rowKey={(r) => r.id}
              columns={[
                {
                  key: "number",
                  header: "Session",
                  render: (r) => (
                    <LinkButton to={`/inventory/count-sessions/${r.id}`}>
                      {r.sessionNumber}
                    </LinkButton>
                  ),
                },
                {
                  key: "branch",
                  header: "Branch",
                  render: (r) =>
                    branches.find((b) => b.id === r.branchId)?.name ??
                    r.branchId,
                },
                { key: "status", header: "Status", render: (r) => r.status },
                { key: "scope", header: "Scope", render: (r) => r.scopeType },
                {
                  key: "dates",
                  header: "Created / started / completed",
                  render: (r) => (
                    <>
                      {r.createdAt}
                      <br />
                      {r.startedAt ?? "Not started"}
                      <br />
                      {r.completedAt ?? "Not completed"}
                    </>
                  ),
                },
                {
                  key: "variance",
                  header: "Variances",
                  render: (r) =>
                    `${r.summary.positive} positive · ${r.summary.negative} negative · ${r.summary.uncounted} uncounted`,
                },
              ]}
            />
            <div className="grid-footer">
              <span>{total} count sessions · local SQLite</span>
              <Button disabled={!page} onClick={() => setPage((p) => p - 1)}>
                Previous counts
              </Button>
              <span>Page {page + 1}</span>
              <Button
                disabled={(page + 1) * 20 >= total}
                onClick={() => setPage((p) => p + 1)}
              >
                Next counts
              </Button>
            </div>
          </Card>
        </>
      )}
    </div>
  );
}
function CountWorkspace({ initial }: { initial: CountDetail }) {
  const { branches } = useBranch();
  const [d, setD] = useState(initial);
  const [page, setPage] = useState(0);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const lock = useRef(false);
  const [dirty, setDirty] = useState<string[]>([]);
  const [reloadVersion, setReloadVersion] = useState(0);
  const [confirm, setConfirm] = useState<"complete" | "cancel" | null>(null);
  const run = async (work: () => Promise<CountDetail>, offset = page * 20) => {
    if (lock.current) return false;
    lock.current = true;
    setBusy(true);
    setError("");
    try {
      const result = await work();
      setD(await getCount(result.session.id, offset));
      return true;
    } catch (e) {
      setError(message(e));
      return false;
    } finally {
      lock.current = false;
      setBusy(false);
    }
  };
  const state = d.session.status;
  const editing = state === "IN_PROGRESS";
  const closed = state === "COMPLETED" || state === "CANCELLED";
  const clean = dirty.length === 0;
  const changed = (id: string) =>
    setDirty((old) => (old.includes(id) ? old : [...old, id]));
  const save = async (
    line: CountLine,
    patch: Pick<CountEntry, "countedQuantity" | "notes" | "excludedReason">,
  ) => {
    const ok = await run(() =>
      recordCount({
        ...countTransition(d),
        lineId: line.id,
        lineRevision: line.revision,
        ...patch,
      }),
    );
    if (ok) setDirty((old) => old.filter((id) => id !== line.id));
    return ok;
  };
  return (
    <div className="stack">
      <Card>
        <div className="row between">
          <h2>{d.session.sessionNumber}</h2>
          <strong>{state}</strong>
          <LinkButton to="/inventory/count-sessions">Back to counts</LinkButton>
        </div>
        <p>
          {branches.find((b) => b.id === d.session.branchId)?.name} ·
          ALL_INVENTORY · {d.session.notes}
        </p>
        {error && (
          <Alert title="Count action failed" tone="danger">
            {error}
            {editing && (
              <Button
                disabled={busy}
                onClick={() => {
                  void run(() => getCount(d.session.id, page * 20)).then(
                    (ok) => {
                      if (ok) {
                        setDirty([]);
                        setReloadVersion((v) => v + 1);
                      }
                    },
                  );
                }}
              >
                Reload saved count and discard unsaved edits
              </Button>
            )}
          </Alert>
        )}
        {state === "DRAFT" && (
          <Alert title="Draft count — stock unchanged" tone="info">
            Starting captures the fixed expected snapshot. Every physical count
            starts blank.
          </Alert>
        )}
        {state === "COMPLETED" && (
          <Alert title="Completed count is read-only" tone="success">
            The recorded live baseline, physical counts and correction links are
            permanent.
          </Alert>
        )}
        {state === "CANCELLED" && (
          <Alert title="Count cancelled" tone="info">
            Cancellation created no stock corrections. This session cannot
            reopen.
          </Alert>
        )}
        {d.summary.intervening > 0 && (
          <Alert title="Intervening stock movements detected" tone="warning">
            {d.summary.intervening} lines changed after the snapshot.
            Corrections use current live stock, while the original expected
            snapshot is preserved.
          </Alert>
        )}
        <p aria-label="Count summary">
          {d.summary.total} total · {d.summary.counted} counted ·{" "}
          {d.summary.uncounted} uncounted · {d.summary.excluded} excluded ·{" "}
          {d.summary.matches} matches · {d.summary.positive} positive variances
          · {d.summary.negative} negative variances
        </p>
        <div className="row wrap">
          {state === "DRAFT" && (
            <Button
              disabled={busy}
              onClick={() => void run(() => startCount(countTransition(d)))}
            >
              Start count
            </Button>
          )}
          {editing && (
            <Button
              disabled={busy || !clean}
              onClick={() => void run(() => reviewCount(countTransition(d)))}
            >
              Review variances
            </Button>
          )}
          {state === "REVIEW" && (
            <>
              <Button
                disabled={busy}
                onClick={() => void run(() => resumeCount(countTransition(d)))}
              >
                Return to entry
              </Button>
              <Button
                disabled={busy || d.summary.uncounted > 0}
                onClick={() => setConfirm("complete")}
              >
                Complete count
              </Button>
            </>
          )}
          {!closed && (
            <Button
              disabled={busy || !clean}
              onClick={() => setConfirm("cancel")}
            >
              Cancel count
            </Button>
          )}
          <Button
            disabled={busy || !clean}
            onClick={() => void run(() => getCount(d.session.id, page * 20))}
          >
            Refresh review
          </Button>
        </div>
        {!clean && (
          <p role="status">
            Save each edited row before review, cancellation or changing pages.
          </p>
        )}
        {state !== "DRAFT" && (
          <div className="table-scroll">
            <table className="data-table" aria-label="Count entries">
              <thead>
                <tr>
                  {[
                    "Product / package",
                    "Batch / expiry",
                    "Snapshot expected",
                    "Physical count",
                    "Snapshot variance",
                    state === "COMPLETED"
                      ? "Final live baseline"
                      : "Current live",
                    "Final correction",
                    "Movement / actions",
                  ].map((t) => (
                    <th key={t}>{t}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {d.lines.map((l) => (
                  <CountEntryRow
                    key={`${l.id}:${l.revision}:${reloadVersion}`}
                    line={l}
                    editing={editing}
                    disabled={busy}
                    changed={() => changed(l.id)}
                    save={(patch) => save(l, patch)}
                  />
                ))}
              </tbody>
            </table>
          </div>
        )}
        <div className="grid-footer">
          <Button
            disabled={busy || !clean || page === 0}
            onClick={() => {
              setPage(page - 1);
              void run(
                () => getCount(d.session.id, (page - 1) * 20),
                (page - 1) * 20,
              );
            }}
          >
            Previous lines
          </Button>
          <span>Lines page {page + 1}</span>
          <Button
            disabled={busy || !clean || (page + 1) * 20 >= d.summary.total}
            onClick={() => {
              setPage(page + 1);
              void run(
                () => getCount(d.session.id, (page + 1) * 20),
                (page + 1) * 20,
              );
            }}
          >
            Next lines
          </Button>
        </div>
      </Card>
      {editing && (
        <Discovery
          disabled={busy || !clean}
          onAdd={(i) =>
            run(() => discoverCount({ ...countTransition(d), ...i }))
          }
          branchId={d.session.branchId}
        />
      )}
      <Modal
        open={confirm !== null}
        onOpenChange={(open) => {
          if (!open) setConfirm(null);
        }}
        title={
          confirm === "complete"
            ? "Complete stock count?"
            : "Cancel stock count?"
        }
        description={
          confirm === "complete"
            ? `This will create exactly ${d.summary.corrections} correction movements using the reviewed live balances. If stock changes again, completion will be rejected until review is refreshed.`
            : "Cancel this session without creating stock movements. It cannot reopen."
        }
      >
        <Button disabled={busy} onClick={() => setConfirm(null)}>
          Back
        </Button>
        <Button
          disabled={busy}
          onClick={() => {
            const action = confirm;
            setConfirm(null);
            void run(() =>
              action === "complete"
                ? completeCount(d)
                : cancelCount(countTransition(d)),
            );
          }}
        >
          Confirm count {confirm === "complete" ? "completion" : "cancellation"}
        </Button>
      </Modal>
    </div>
  );
}
function CountEntryRow({
  line: l,
  editing,
  disabled,
  changed,
  save,
}: {
  line: CountLine;
  editing: boolean;
  disabled: boolean;
  changed: () => void;
  save: (
    patch: Pick<CountEntry, "countedQuantity" | "notes" | "excludedReason">,
  ) => Promise<boolean>;
}) {
  const [value, setValue] = useState(
    l.countedQuantity === null ? "" : String(l.countedQuantity),
  );
  const [notes, setNotes] = useState(l.notes ?? "");
  const [exclusion, setExclusion] = useState(l.excludedReason ?? "");
  const [error, setError] = useState("");
  const submit = () => {
    const q = value === "" ? null : Number(value);
    if (q !== null && (!Number.isSafeInteger(q) || q < 0)) {
      setError("Enter zero or a positive whole package count.");
      return;
    }
    setError("");
    void save({
      countedQuantity: q,
      notes: notes || null,
      excludedReason: null,
    });
  };
  return (
    <tr>
      <td>
        {l.productLabel}
        <br />
        {l.packageLabel}
      </td>
      <td>
        {l.lotNumber ?? "Unspecified lot"}
        <br />
        {l.expiryDate ?? "Unknown expiry"}
      </td>
      <td>{l.expectedQuantity}</td>
      <td>
        {editing ? (
          <>
            <Input
              label="Physical count"
              aria-label={`Physical count ${l.id}`}
              type="number"
              min="0"
              step="1"
              value={value}
              disabled={disabled}
              onChange={(e) => {
                setValue(e.target.value);
                changed();
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  submit();
                }
              }}
            />
            <Input
              label="Notes"
              aria-label={`Count notes ${l.id}`}
              value={notes}
              disabled={disabled}
              onChange={(e) => {
                setNotes(e.target.value);
                changed();
              }}
            />
            {error && <span role="alert">{error}</span>}
          </>
        ) : (
          (l.countedQuantity ??
          (l.excludedReason ? `Excluded: ${l.excludedReason}` : "Uncounted"))
        )}
      </td>
      <td>{l.snapshotVariance ?? "—"}</td>
      <td>
        {l.currentLive}
        {l.interveningMovements > 0 && (
          <strong> · Changed ({l.interveningMovements})</strong>
        )}
      </td>
      <td>{l.correctionDelta ?? "—"}</td>
      <td>
        {editing ? (
          <>
            <Button
              aria-label={`Save count ${l.id}`}
              disabled={disabled}
              onClick={submit}
            >
              Save count
            </Button>
            <Input
              label="Exclusion reason"
              aria-label={`Exclusion reason ${l.id}`}
              value={exclusion}
              disabled={disabled}
              onChange={(e) => {
                setExclusion(e.target.value);
                changed();
              }}
            />
            <Button
              aria-label={`Exclude line ${l.id}`}
              disabled={disabled || !exclusion.trim()}
              onClick={() =>
                void save({
                  countedQuantity: null,
                  notes: notes || null,
                  excludedReason: exclusion,
                })
              }
            >
              Exclude line
            </Button>
          </>
        ) : (
          (l.movementId ?? (l.correctionDelta === 0 ? "No correction" : "—"))
        )}
      </td>
    </tr>
  );
}
function Discovery({
  branchId,
  disabled,
  onAdd,
}: {
  branchId: string;
  disabled: boolean;
  onAdd: (
    i: Omit<CountDiscovery, "sessionId" | "expectedRevision">,
  ) => Promise<boolean>;
}) {
  const [search, setSearch] = useState("");
  const [products, setProducts] = useState<ProductListItem[]>([]);
  const [product, setProduct] = useState("");
  const [packages, setPackages] = useState<PackageDetail[]>([]);
  const [packageId, setPackage] = useState("");
  const [batches, setBatches] = useState<BatchChoice[]>([]);
  const [batch, setBatch] = useState("");
  const [offset, setOffset] = useState(0);
  const [more, setMore] = useState(false);
  const [lot, setLot] = useState("");
  const [expiry, setExpiry] = useState("");
  const [notes, setNotes] = useState("");
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
      .then((p) => {
        if (active) setPackages(p.packages.filter((p) => p.package.isActive));
      })
      .catch((e) => {
        if (active) setError(message(e));
      });
    return () => {
      active = false;
    };
  }, [product]);
  useEffect(() => {
    if (!packageId) return;
    let active = true;
    workflowBatches(branchId, packageId, offset)
      .then((b) => {
        if (active) {
          setBatches((old) => (offset ? [...old, ...b] : b));
          setMore(b.length === 50);
        }
      })
      .catch((e) => {
        if (active) setError(message(e));
      });
    return () => {
      active = false;
    };
  }, [branchId, packageId, offset]);
  return (
    <Card>
      <h3>Add discovered stock</h3>
      <p>
        Select an existing batch or explicitly identify a previously unrecorded
        lot. Adding creates no stock quantity; enter its physical count in the
        session.
      </p>
      {error && (
        <Alert title="Discovery could not be added" tone="danger">
          {error}
        </Alert>
      )}
      <div className="row wrap">
        <Input
          label="Search discovered package"
          value={search}
          disabled={disabled}
          onChange={(e) => setSearch(e.target.value)}
        />
        <Select
          label="Discovered product"
          value={product}
          disabled={disabled}
          onChange={(e) => {
            setProduct(e.target.value);
            setPackage("");
            setPackages([]);
            setBatch("");
          }}
        >
          <option value="">Select product</option>
          {products.map((p) => (
            <option key={p.id} value={p.id}>
              {p.nameEn ?? p.nameAr}
            </option>
          ))}
        </Select>
        <Select
          label="Discovered package"
          value={packageId}
          disabled={disabled}
          onChange={(e) => {
            setPackage(e.target.value);
            setBatch("");
            setOffset(0);
            setBatches([]);
          }}
        >
          <option value="">Select package</option>
          {packages.map((p) => (
            <option key={p.package.id} value={p.package.id}>
              {p.package.packageLabel}
            </option>
          ))}
        </Select>
        <Select
          label="Discovered batch"
          value={batch}
          disabled={disabled}
          onChange={(e) => setBatch(e.target.value)}
        >
          <option value="">Previously unrecorded batch</option>
          {batches.map((b) => (
            <option key={b.id} value={b.id}>
              {b.lotNumber ?? "Unspecified lot"} · {b.quantity} available
            </option>
          ))}
        </Select>
        {more && (
          <Button disabled={disabled} onClick={() => setOffset((o) => o + 50)}>
            More discovered batches
          </Button>
        )}
        {!batch && (
          <>
            <Input
              label="Discovered lot"
              value={lot}
              disabled={disabled}
              onChange={(e) => setLot(e.target.value)}
            />
            <Input
              label="Discovered expiry"
              type="date"
              value={expiry}
              disabled={disabled}
              onChange={(e) => setExpiry(e.target.value)}
            />
          </>
        )}
        <Input
          label="Discovery reason"
          value={notes}
          disabled={disabled}
          onChange={(e) => setNotes(e.target.value)}
        />
        <Button
          disabled={disabled || !packageId || !notes.trim()}
          onClick={() =>
            void onAdd({
              productPackageId: packageId,
              batchId: batch || null,
              lotNumber: batch ? null : lot || null,
              expiryDate: batch ? null : expiry || null,
              notes,
            })
          }
        >
          Add discovered batch
        </Button>
      </div>
    </Card>
  );
}
