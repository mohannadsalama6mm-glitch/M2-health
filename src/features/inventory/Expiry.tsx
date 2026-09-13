import { useEffect, useState } from "react";
import {
  PageHeader,
  RouteTabs,
  ExpiryBadge,
  PriceDisplay,
  Button,
  SummaryStrip,
  Modal,
  Select,
  Input,
  Textarea,
  Card,
  TableToolbar,
  SearchInput,
  Table,
  Pagination,
  Alert,
  EmptyState,
  type Column,
} from "../../design-system";
import { isDesktopRuntime } from "../../lib/tauri/client";
import { useBranch } from "../../app/BranchContext";
import { useDemo } from "../../app/DemoContext";
import { listExpiry, writeOffStock, toMajor } from "../../lib/tauri/inventory";
import type { ExpiryRow, ExpiryPage } from "../../lib/tauri/inventory";
import { inventoryTabs } from "./Inventory";
const PAGE_SIZE = 10;
const WINDOWS = [
  { label: "Expired", days: -1 },
  { label: "Within 30 days", days: 30 },
  { label: "Within 60 days", days: 60 },
  { label: "Within 90 days", days: 90 },
];
const suggestion = (days: number) =>
  days < 0 ? "Quarantine / write-off" : "Review supplier return";
export function Expiry() {
  const desktop = isDesktopRuntime();
  const { notify } = useDemo();
  const { branchId } = useBranch();
  const [windowDays, setWindowDays] = useState(30);
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  const [data, setData] = useState<ExpiryPage | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);
  const [strip, setStrip] = useState<{ days: number; total: number }[]>([]);
  const [writeOffRow, setWriteOffRow] = useState<ExpiryRow | null>(null);
  useEffect(() => {
    if (!desktop) return;
    let active = true;
    Promise.all(
      WINDOWS.map((w) =>
        listExpiry({ branchId, windowDays: w.days, limit: 1 })
          .then((p) => ({ days: w.days, total: p.total }))
          .catch(() => ({ days: w.days, total: 0 })),
      ),
    ).then((totals) => {
      if (active) setStrip(totals);
    });
    return () => {
      active = false;
    };
  }, [desktop, branchId, attempt]);
  useEffect(() => {
    if (!desktop) return;
    let active = true;
    const timer = window.setTimeout(
      () => {
        if (!active) return;
        setLoading(true);
        setError("");
        listExpiry({
          branchId,
          windowDays: windowDays,
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
                : "Expiry batches could not be loaded.",
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
  }, [desktop, branchId, windowDays, search, page, attempt]);
  const pages = data ? Math.max(1, Math.ceil(data.total / data.limit)) : 1;
  const stripValue = (days: number) =>
    strip.find((s) => s.days === days)?.total ?? "—";
  const columns: Column<ExpiryRow>[] = [
    {
      key: "name",
      header: "Product / package",
      render: (r) => (
        <>
          {r.nameEn ?? r.nameAr ?? r.scientificName ?? "Unnamed product"}
          <span className="cell-secondary">{r.packageLabel}</span>
        </>
      ),
    },
    { key: "batch", header: "Batch", render: (r) => r.batchNumber },
    { key: "quantity", header: "Quantity", render: (r) => r.quantity },
    {
      key: "date",
      header: "Expiry date",
      render: (r) => (r.expiryDate ? r.expiryDate : "—"),
    },
    {
      key: "days",
      header: "Time left",
      render: (r) => <ExpiryBadge days={r.days} />,
    },
    {
      key: "value",
      header: "Value at risk",
      render: (r) => (
        <PriceDisplay amount={toMajor(r.valueMinor)} />
      ),
    },
    { key: "branch", header: "Branch", render: (r) => r.branchName },
    {
      key: "suggestion",
      header: "Suggested action",
      render: (r) => suggestion(r.days),
    },
    {
      key: "actions",
      header: "Actions",
      render: (r) => (
        <Button
          size="sm"
          variant="danger"
          onClick={() => setWriteOffRow(r)}
        >
          Write-off
        </Button>
      ),
    },
  ];
  return (
    <div className="feature-page">
      <PageHeader
        title="Expiry management"
        description="Prioritize batches by expiry date and value at risk."
      />
      <RouteTabs items={inventoryTabs} />
      {!desktop && (
        <Alert title="Desktop app required" tone="warning">
          Expiry batches are tracked in the local database. Open M² Health as
          the desktop app to manage expiring stock.
        </Alert>
      )}
      <SummaryStrip
        items={WINDOWS.map((w) => ({
          label: w.label,
          value: desktop ? stripValue(w.days) : "—",
        }))}
      />
      <div className="section-menu">
        {WINDOWS.map((w) => (
          <Button
            key={w.days}
            variant={windowDays === w.days ? "primary" : "secondary"}
            onClick={() => {
              setWindowDays(w.days);
              setPage(1);
            }}
          >
            {w.label}
          </Button>
        ))}
      </div>
      {desktop && error && (
        <Alert title="Expiry batches could not be loaded" tone="danger">
          {error}
          <Button size="sm" onClick={() => setAttempt((v) => v + 1)}>
            Retry
          </Button>
        </Alert>
      )}
      {!desktop ? (
        <Card>
          <EmptyState
            title="Expiry management requires the desktop app"
            description="Batch expiry dates are stored in the local SQLite database."
          />
        </Card>
      ) : (
        <Card>
          <TableToolbar>
            <SearchInput
              label="Search expiry batches"
              placeholder="Search product, batch…"
              value={search}
              onChange={(e) => {
                setSearch(e.target.value);
                setPage(1);
              }}
            />
          </TableToolbar>
          {loading && !data ? (
            <div className="stack" role="status">
              <Alert title="Loading expiry batches…" tone="info" />
            </div>
          ) : (
            <Table
              label="Expiry batches"
              rows={data?.items ?? []}
              columns={columns}
              rowKey={(r) => r.batchId}
            />
          )}
          <div className="grid-footer">
            <span className="muted">
              {data ? `${data.total} batches · branch-scoped` : "—"}
            </span>
            <Pagination page={page} pages={pages} onChange={setPage} />
          </div>
        </Card>
      )}
      <WriteOffDialog
        key={writeOffRow ? writeOffRow.batchId : "closed"}
        open={!!writeOffRow}
        onOpenChange={() => setWriteOffRow(null)}
        row={writeOffRow}
        notify={notify}
        onDone={() => {
          setWriteOffRow(null);
          setAttempt((v) => v + 1);
        }}
        onError={(message) => setError(message)}
      />
    </div>
  );
}
function WriteOffDialog({
  open,
  onOpenChange,
  row,
  notify,
  onDone,
  onError,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  row: ExpiryRow | null;
  notify: (message: string) => void;
  onDone: () => void;
  onError: (message: string) => void;
}) {
  const [quantity, setQuantity] = useState(row ? String(row.quantity) : "1");
  const [kind, setKind] = useState<"damage" | "expired" | "supplier_return">(
    "expired",
  );
  const [reason, setReason] = useState("");
  const [submitting, setSubmitting] = useState(false);
  return (
    <Modal
      open={open}
      onOpenChange={onOpenChange}
      title="Write off batch"
      description="Removes stock with a signed ledger movement. Unused stock should go through supplier return instead."
    >
      <div className="stack">
        {row && (
          <p className="muted">
            {row.nameEn ?? row.nameAr ?? row.scientificName ?? "Unnamed"} —{" "}
            {row.batchNumber} · {row.quantity} in stock on {row.branchName}
          </p>
        )}
        <Input
          label="Quantity to write off"
          type="number"
          min="1"
          value={quantity}
          onChange={(e) => setQuantity(e.target.value)}
        />
        <Select
          label="Write-off kind"
          value={kind}
          onChange={(e) =>
            setKind(e.target.value as "damage" | "expired" | "supplier_return")
          }
        >
          <option value="expired">Expired</option>
          <option value="damage">Damaged</option>
          <option value="supplier_return">Supplier return</option>
        </Select>
        <Textarea
          label="Reason"
          required
          value={reason}
          onChange={(e) => setReason(e.target.value)}
          placeholder="Expiry review, broken packaging…"
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
            variant="danger"
            loading={submitting}
            disabled={!row || !quantity || Number(quantity) <= 0 || !reason.trim()}
            onClick={() => {
              if (!row) return;
              setSubmitting(true);
              void writeOffStock({
                batchId: row.batchId,
                kind,
                quantity: Number(quantity),
                reason: reason.trim(),
                user: null,
              })
                .then(() => {
                  notify("Stock written off and posted to the ledger.");
                  onDone();
                })
                .catch((err: unknown) => {
                  setSubmitting(false);
                  onError(
                    err instanceof Error
                      ? err.message
                      : "The write-off could not be posted.",
                  );
                });
            }}
          >
            Write off
          </Button>
        </div>
      </div>
    </Modal>
  );
}