import { useEffect, useState } from "react";
import {
  PageHeader,
  RouteTabs,
  Status,
  Button,
  Modal,
  Select,
  Input,
  Table,
  Pagination,
  Alert,
  EmptyState,
  Card,
  TableToolbar,
  SearchInput,
  ConfirmationModal,
  type Column,
} from "../../design-system";
import { isDesktopRuntime } from "../../lib/tauri/client";
import { useBranch } from "../../app/BranchContext";
import { useDemo } from "../../app/DemoContext";
import { listCategories } from "../../lib/tauri/catalog";
import {
  listStockCounts,
  getStockCount,
  createStockCount,
  saveCountItem,
  completeStockCount,
} from "../../lib/tauri/inventory";
import type {
  StockCountRow,
  StockCountDetail,
  StockCountItemView,
} from "../../lib/tauri/inventory";
import { inventoryTabs } from "./Inventory";
const PAGE_SIZE = 8;
const countStatusLabel = (status: string) =>
  ({
    draft: "Draft",
    in_progress: "In progress",
    completed: "Completed",
  })[status] ?? status;
export function StockCounts() {
  const desktop = isDesktopRuntime();
  const { notify } = useDemo();
  const { branchId } = useBranch();
  const [search, setSearch] = useState("");
  const [status, setStatus] = useState("");
  const [page, setPage] = useState(1);
  const [data, setData] = useState<{
    items: StockCountRow[];
    total: number;
    limit: number;
    offset: number;
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);
  const [detail, setDetail] = useState<StockCountDetail | null>(null);
  const [detailRevision, setDetailRevision] = useState(0);
  const [detailLoading, setDetailLoading] = useState(false);
  const [newOpen, setNewOpen] = useState(false);
  const [completeTarget, setCompleteTarget] = useState<string | null>(null);
  const [completing, setCompleting] = useState(false);
  useEffect(() => {
    if (!desktop) return;
    let active = true;
    const timer = window.setTimeout(
      () => {
        if (!active) return;
        setLoading(true);
        setError("");
        listStockCounts({
          branchId,
          status: status || null,
          search: search || null,
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
                : "Count sessions could not be loaded.",
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
  }, [desktop, branchId, search, status, page, attempt]);
  const openDetail = (id: string) => {
    if (!desktop) return;
    setDetail(null);
    setDetailLoading(true);
    setError("");
    void getStockCount(id)
      .then((d) => {
        setDetail(d);
        setDetailRevision((v) => v + 1);
      })
      .catch((err: unknown) =>
        setError(
          err instanceof Error
            ? err.message
            : "The count session could not be loaded.",
        ),
      )
      .finally(() => setDetailLoading(false));
  };
  const pages = data ? Math.max(1, Math.ceil(data.total / data.limit)) : 1;
  const columns: Column<StockCountRow>[] = [
    {
      key: "id",
      header: "Session",
      render: (r) => (
        <Button variant="ghost" size="sm" onClick={() => openDetail(r.id)}>
          {r.id}
        </Button>
      ),
    },
    { key: "branch", header: "Branch", render: (r) => r.branchName },
    { key: "scope", header: "Scope", render: (r) => r.scope || "—" },
    {
      key: "category",
      header: "Category",
      render: (r) => r.categoryName ?? "All categories",
    },
    {
      key: "started",
      header: "Started",
      render: (r) => r.startedAt.slice(0, 16).replace("T", " "),
    },
    { key: "items", header: "Items", render: (r) => r.itemCount },
    {
      key: "discrepancies",
      header: "Discrepancies",
      render: (r) => r.discrepancyCount,
    },
    {
      key: "status",
      header: "Status",
      render: (r) => <Status value={countStatusLabel(r.status)} />,
    },
  ];
  return (
    <div className="feature-page">
      <PageHeader
        title="Stock counts"
        description="Count, compare, and review differences before a stock correction."
        actions={
          <Button
            variant="primary"
            onClick={() => setNewOpen(true)}
            disabled={!desktop}
          >
            New count session
          </Button>
        }
      />
      <RouteTabs items={inventoryTabs} />
      {!desktop && (
        <Alert title="Desktop app required" tone="warning">
          Stock counts run against the local database. Open M² Health as the
          desktop app to count inventory.
        </Alert>
      )}
      {desktop && error && (
        <Alert title="Stock counts could not be loaded" tone="danger">
          {error}
          <Button size="sm" onClick={() => setAttempt((v) => v + 1)}>
            Retry
          </Button>
        </Alert>
      )}
      {!desktop ? (
        <Card>
          <EmptyState
            title="Stock counts require the desktop app"
            description="Count sessions and corrections are stored in the local SQLite database."
          />
        </Card>
      ) : (
        <Card>
          <TableToolbar>
            <SearchInput
              label="Search count sessions"
              placeholder="Search sessions…"
              value={search}
              onChange={(e) => {
                setSearch(e.target.value);
                setPage(1);
              }}
            />
            <Select
              label="Status"
              value={status}
              onChange={(e) => {
                setStatus(e.target.value);
                setPage(1);
              }}
            >
              <option value="">All sessions</option>
              {["draft", "in_progress", "completed"].map((s) => (
                <option key={s} value={s}>
                  {countStatusLabel(s)}
                </option>
              ))}
            </Select>
          </TableToolbar>
          {loading && !data ? (
            <div className="stack" role="status">
              <Alert title="Loading count sessions…" tone="info" />
            </div>
          ) : (
            <Table
              label="Count sessions"
              rows={data?.items ?? []}
              columns={columns}
              rowKey={(r) => r.id}
            />
          )}
          <div className="grid-footer">
            <span className="muted">
              {data ? `${data.total} sessions · branch-scoped` : "—"}
            </span>
            <Pagination page={page} pages={pages} onChange={setPage} />
          </div>
        </Card>
      )}
      <NewCountDialog
        key={newOpen ? "open" : "closed"}
        open={newOpen}
        onOpenChange={(open) => {
          setNewOpen(open);
          if (!open) setAttempt((v) => v + 1);
        }}
        branchId={branchId}
        notify={notify}
        onCreated={(id) => {
          setNewOpen(false);
          setAttempt((v) => v + 1);
          openDetail(id);
        }}
        onError={(message) => setError(message)}
      />
      <CountDetailModal
        detail={detail}
        revision={detailRevision}
        loading={detailLoading}
        onOpenChange={() => setDetail(null)}
        notify={notify}
        onChanged={() => openDetail(detail?.count.id ?? "")}
        onComplete={(id) => setCompleteTarget(id)}
        onError={(message) => setError(message)}
      />
      <ConfirmationModal
        open={!!completeTarget}
        onOpenChange={() => setCompleteTarget(null)}
        title="Complete count session?"
        description="Each counted difference posts a signed count correction movement. The session becomes read-only."
        onConfirm={() => {
          if (!completeTarget) return;
          setCompleting(true);
          void completeStockCount({ stockCountId: completeTarget })
            .then((d) => {
              setCompleting(false);
              setDetail(d);
              setDetailRevision((v) => v + 1);
              setAttempt((v) => v + 1);
              notify("Count completed. Corrections were posted to the ledger.");
            })
            .catch((err: unknown) => {
              setCompleting(false);
              setError(
                err instanceof Error
                  ? err.message
                  : "The count could not be completed.",
              );
            });
        }}
      />
      {completing && (
        <div className="stack" role="status">
          <Alert title="Completing count…" tone="info" />
        </div>
      )}
    </div>
  );
}
function NewCountDialog({
  open,
  onOpenChange,
  branchId,
  notify,
  onCreated,
  onError,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  branchId: string;
  notify: (message: string) => void;
  onCreated: (id: string) => void;
  onError: (message: string) => void;
}) {
  const { branchName } = useBranch();
  const [scope, setScope] = useState("");
  const [categoryId, setCategoryId] = useState("");
  const [categories, setCategories] = useState<
    { id: string; name: string }[]
  >([]);
  const [submitting, setSubmitting] = useState(false);
  useEffect(() => {
    let active = true;
    listCategories()
      .then((c) => {
        if (active) setCategories(c);
      })
      .catch(() => {
        // Categories stay optional; an empty list just hides the scope select.
      });
    return () => {
      active = false;
    };
  }, []);
  return (
    <Modal
      open={open}
      onOpenChange={onOpenChange}
      title="New stock count"
      description="Snapshot every positive stock batch on the selected branch. Discrepancies post corrections on completion."
    >
      <div className="stack">
        <p className="muted">Branch: {branchName}</p>
        <Input
          label="Scope / note"
          value={scope}
          onChange={(e) => setScope(e.target.value)}
          placeholder="End-of-month count, cold storage section…"
        />
        {categories.length > 0 && (
          <Select
            label="Category scope"
            value={categoryId}
            onChange={(e) => setCategoryId(e.target.value)}
          >
            <option value="">All categories</option>
            {categories.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </Select>
        )}
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
            onClick={() => {
              setSubmitting(true);
              void createStockCount({
                branchId,
                scope: scope.trim() || undefined,
                categoryId: categoryId || null,
              })
                .then((c) => {
                  notify(`Count session ${c.id} created.`);
                  onCreated(c.id);
                })
                .catch((err: unknown) => {
                  setSubmitting(false);
                  onError(
                    err instanceof Error
                      ? err.message
                      : "The count session could not be created.",
                  );
                });
            }}
          >
            Create session
          </Button>
        </div>
      </div>
    </Modal>
  );
}
function CountDetailModal({
  detail,
  revision,
  loading,
  onOpenChange,
  notify,
  onChanged,
  onComplete,
  onError,
}: {
  detail: StockCountDetail | null;
  revision: number;
  loading: boolean;
  onOpenChange: (open: boolean) => void;
  notify: (message: string) => void;
  onChanged: () => void;
  onComplete: (id: string) => void;
  onError: (message: string) => void;
}) {
  return (
    <Modal
      size="lg"
      open={!!detail || loading}
      onOpenChange={onOpenChange}
      title={detail ? `Count ${detail.count.id}` : "Loading session…"}
      description={
        detail
          ? `${detail.branchName} · ${countStatusLabel(detail.count.status)} · ${detail.items.length} counted items`
          : "Fetching count session…"
      }
    >
      <div className="stack">
        {!detail ? (
          <Alert
            title={loading ? "Loading session…" : "No session selected"}
            tone="info"
          />
        ) : (
          <CountTable
            key={`${detail.count.id}-${revision}`}
            detail={detail}
            notify={notify}
            onChanged={onChanged}
            onComplete={onComplete}
            onError={onError}
          />
        )}
      </div>
    </Modal>
  );
}
function CountTable({
  detail,
  notify,
  onChanged,
  onComplete,
  onError,
}: {
  detail: StockCountDetail;
  notify: (message: string) => void;
  onChanged: () => void;
  onComplete: (id: string) => void;
  onError: (message: string) => void;
}) {
  const [counts, setCounts] = useState<Record<string, number | "">>(() =>
    Object.fromEntries(
      detail.items.map((it) => [it.batchId, it.countedQuantity]),
    ),
  );
  const [review, setReview] = useState(false);
  const [saving, setSaving] = useState(false);
  const isCompleted = detail.count.status === "completed";
  const effective = (it: StockCountItemView) =>
    counts[it.batchId] ?? it.systemQuantity;
  const changed = detail.items.filter(
    (it) => effective(it) !== it.countedQuantity,
  );
  const save = () => {
    if (!changed.length) return;
    setSaving(true);
    const jobs = changed.map((it) =>
      saveCountItem({
        stockCountId: detail.count.id,
        batchId: it.batchId,
        countedQuantity: Number(effective(it)),
      }),
    );
    void Promise.all(jobs)
      .then(() => {
        setSaving(false);
        notify("Counted quantities saved.");
        onChanged();
      })
      .catch((err: unknown) => {
        setSaving(false);
        onError(
          err instanceof Error
            ? err.message
            : "Counted quantities could not be saved.",
        );
      });
  };
  const items = detail.items;
  const shown = review
    ? items.filter((it) => effective(it) !== it.systemQuantity)
    : items;
  return (
    <>
      {isCompleted ? (
        <Alert title="Count completed" tone="success">
          All differences were posted as count corrections. This session is
          read-only.
        </Alert>
      ) : (
        <Alert title="Editing session" tone="info">
          Enter counted quantities, then save. Completing posts a correction
          movement for every difference.
        </Alert>
      )}
      <Table
        label="Count quantities"
        rows={shown}
        rowKey={(it) => it.batchId}
        columns={[
          {
            key: "product",
            header: "Product / package",
            render: (it) => (
              <>
                {it.product}
                <span className="cell-secondary">
                  {[it.pack, it.batchNumber].filter(Boolean).join(" · ")}
                </span>
              </>
            ),
          },
          {
            key: "system",
            header: "Expected",
            render: (it) => it.systemQuantity,
          },
          {
            key: "counted",
            header: "Counted",
            render: (it) =>
              isCompleted ? (
                it.countedQuantity
              ) : (
                <Input
                  hideLabel
                  label={`Count ${it.product}`}
                  type="number"
                  min={0}
                  value={counts[it.batchId] ?? it.systemQuantity}
                  onChange={(e) =>
                    setCounts((prev) => ({
                      ...prev,
                      [it.batchId]:
                        e.target.value === ""
                          ? ""
                          : Math.max(0, Number(e.target.value)),
                    }))
                  }
                />
              ),
          },
          {
            key: "variance",
            header: "Variance",
            render: (it) =>
              isCompleted
                ? it.variance
                : effective(it) === ""
                  ? -it.systemQuantity
                  : Number(effective(it)) - it.systemQuantity,
          },
        ]}
      />
      {!isCompleted && (
        <div className="row">
          <Button variant="secondary" onClick={() => setReview((v) => !v)}>
            {review ? "Show all items" : "Review differences"}
          </Button>
          <Button
            variant="primary"
            loading={saving}
            disabled={!changed.length}
            onClick={save}
          >
            Save counts
          </Button>
          <Button
            variant="danger"
            disabled={!!changed.length}
            onClick={() => onComplete(detail.count.id)}
          >
            Complete count
          </Button>
        </div>
      )}
    </>
  );
}