import { useState, type ReactNode } from "react";
import { NavLink } from "react-router-dom";
import { Plus, Trash2, Download, Printer, ArrowLeft } from "lucide-react";
import {
  Alert,
  Badge,
  Button,
  Card,
  EmptyState,
  Input,
  Modal,
  PageHeader,
  Pagination,
  SearchInput,
  SectionHeader,
  Select,
  Skeleton,
  Table,
  TableToolbar,
  Textarea,
  type Column,
  type Tone,
} from "../components/core";
export type PreviewState = "ready" | "loading" | "empty" | "error";
export function FeaturePage({
  title,
  description,
  actions,
  children,
}: {
  title: string;
  description: string;
  actions?: ReactNode;
  children: ReactNode;
}) {
  const [state, setState] = useState<PreviewState>("ready");
  return (
    <div className="feature-page">
      <PageHeader title={title} description={description} actions={actions} />
      <div className="feature-meta">
        <Badge tone="warning">UI demo · changes reset on reload</Badge>
        <Select
          label="Preview state"
          value={state}
          onChange={(e) => setState(e.target.value as PreviewState)}
        >
          <option value="ready">Populated view</option>
          <option value="loading">Preview loading</option>
          <option value="empty">Preview empty</option>
          <option value="error">Preview error</option>
        </Select>
      </div>
      {state === "ready" ? (
        children
      ) : state === "loading" ? (
        <Card>
          <SectionHeader title={`Loading ${title.toLowerCase()}…`} />
          <div className="stack" role="status">
            <Skeleton />
            <Skeleton />
            <Skeleton />
          </div>
        </Card>
      ) : state === "empty" ? (
        <Card>
          <EmptyState
            title={`No ${title.toLowerCase()} yet`}
            description="This is the empty-state preview. Return to the sample workspace to explore this module."
            action={
              <Button onClick={() => setState("ready")}>
                Show sample data
              </Button>
            }
          />
        </Card>
      ) : (
        <Alert title="This workspace could not be loaded" tone="danger">
          <span>
            This is a simulated error. Your demo changes are unaffected.{" "}
          </span>
          <Button size="sm" onClick={() => setState("ready")}>
            Retry preview
          </Button>
        </Alert>
      )}
    </div>
  );
}
export function RouteTabs({
  items,
}: {
  items: { label: string; to: string }[];
}) {
  return (
    <nav className="route-tabs" aria-label="Module navigation">
      {items.map((item) => (
        <NavLink key={item.to} end to={item.to}>
          {item.label}
        </NavLink>
      ))}
    </nav>
  );
}
export function LinkButton({
  to,
  children,
  primary = false,
}: {
  to: string;
  children: ReactNode;
  primary?: boolean;
}) {
  return (
    <NavLink
      className={`button ${primary ? "primary" : "secondary"} md`}
      to={to}
    >
      {children}
    </NavLink>
  );
}
export function BackLink({ to, label }: { to: string; label: string }) {
  return (
    <NavLink className="back-link" to={to}>
      <ArrowLeft size={14} />
      {label}
    </NavLink>
  );
}
export function Status({ value }: { value: string }) {
  const tone: Tone = /Critical|Expired|Failed|Denied|Cancelled|Conflict/i.test(
    value,
  )
    ? "danger"
    : /Low|Pending|Draft|Partial|hold|Progress|Ordered|Invited|Setup/i.test(
          value,
        )
      ? "warning"
      : /Active|Completed|Received|Success|stock|Posted|Verified/i.test(value)
        ? "success"
        : "neutral";
  return (
    <Badge tone={tone} dot>
      {value}
    </Badge>
  );
}
export function SummaryStrip({
  items,
}: {
  items: { label: string; value: ReactNode; hint?: string }[];
}) {
  return (
    <div className="summary-strip">
      {items.map((item) => (
        <Card key={item.label}>
          <span className="muted">{item.label}</span>
          <strong>{item.value}</strong>
          {item.hint && <small className="muted">{item.hint}</small>}
        </Card>
      ))}
    </div>
  );
}
export function DetailList({
  items,
}: {
  items: { label: string; value: ReactNode }[];
}) {
  return (
    <dl className="detail-list">
      {items.map((item) => (
        <div key={item.label}>
          <dt>{item.label}</dt>
          <dd>{item.value}</dd>
        </div>
      ))}
    </dl>
  );
}
export function FormSection({
  title,
  description,
  children,
  action,
}: {
  title: string;
  description?: string;
  children: ReactNode;
  action?: ReactNode;
}) {
  return (
    <Card>
      <SectionHeader title={title} subtitle={description} action={action} />
      <div className="form-grid">{children}</div>
    </Card>
  );
}
export type GridFilter<T> = {
  label: string;
  options: string[];
  value: (row: T) => string;
};
export function DataGrid<T>({
  label,
  rows,
  columns,
  rowKey,
  searchText,
  filters = [],
  actions,
  pageSize = 8,
}: {
  label: string;
  rows: T[];
  columns: Column<T>[];
  rowKey: (row: T) => string;
  searchText: (row: T) => string;
  filters?: GridFilter<T>[];
  actions?: ReactNode;
  pageSize?: number;
}) {
  const [query, setQuery] = useState(""),
    [selected, setSelected] = useState<Record<string, string>>({}),
    [page, setPage] = useState(1);
  const filtered = rows.filter(
    (row) =>
      searchText(row).toLowerCase().includes(query.toLowerCase()) &&
      filters.every(
        (f) => !selected[f.label] || f.value(row) === selected[f.label],
      ),
  );
  const pages = Math.max(1, Math.ceil(filtered.length / pageSize));
  const current = Math.min(page, pages);
  return (
    <Card>
      <TableToolbar>
        <SearchInput
          label={`Search ${label}`}
          placeholder={`Search ${label.toLowerCase()}…`}
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setPage(1);
          }}
        />
        <div className="row">
          {filters.map((f) => (
            <Select
              key={f.label}
              label={f.label}
              value={selected[f.label] ?? ""}
              onChange={(e) => {
                setSelected({ ...selected, [f.label]: e.target.value });
                setPage(1);
              }}
            >
              <option value="">All {f.label.toLowerCase()}</option>
              {f.options.map((o) => (
                <option key={o}>{o}</option>
              ))}
            </Select>
          ))}
          {actions}
        </div>
      </TableToolbar>
      <Table
        label={label}
        rows={filtered.slice((current - 1) * pageSize, current * pageSize)}
        columns={columns}
        rowKey={rowKey}
      />
      <div className="grid-footer">
        <span className="muted">{filtered.length} results · demo records</span>
        <Pagination page={current} pages={pages} onChange={setPage} />
      </div>
    </Card>
  );
}
export function RepeatableRows({
  label,
  values,
  onChange,
  placeholder,
}: {
  label: string;
  values: string[];
  onChange: (values: string[]) => void;
  placeholder?: string;
}) {
  return (
    <div className="repeatable">
      <div className="row spread">
        <strong>{label}</strong>
        <Button size="sm" onClick={() => onChange([...values, ""])}>
          <Plus size={14} />
          Add {label.toLowerCase().replace(/s$/, "")}
        </Button>
      </div>
      {values.map((value, i) => (
        <div className="row" key={i}>
          <Input
            label={`${label} ${i + 1}`}
            value={value}
            placeholder={placeholder}
            required
            onChange={(e) =>
              onChange(values.map((v, n) => (n === i ? e.target.value : v)))
            }
          />
          <Button
            size="sm"
            disabled={values.length === 1}
            aria-label={`Remove ${label.toLowerCase()} ${i + 1}`}
            onClick={() => onChange(values.filter((_, n) => n !== i))}
          >
            <Trash2 size={14} />
          </Button>
        </div>
      ))}
    </div>
  );
}
export type WorkflowField = {
  name: string;
  label: string;
  type?: string;
  options?: string[];
  value?: string;
  required?: boolean;
  min?: number;
  pattern?: string;
};
export function WorkflowDialog({
  title,
  description,
  open,
  onClose,
  fields,
  onSubmit,
  submitLabel = "Apply demo change",
  danger = false,
}: {
  title: string;
  description: string;
  open: boolean;
  onClose: () => void;
  fields: WorkflowField[];
  onSubmit: (values: Record<string, string>) => void;
  submitLabel?: string;
  danger?: boolean;
}) {
  return (
    <Modal
      open={open}
      onOpenChange={(o) => !o && onClose()}
      title={title}
      description={description}
    >
      <form
        onSubmit={(e) => {
          e.preventDefault();
          const values = Object.fromEntries(
            new FormData(e.currentTarget).entries(),
          ) as Record<string, string>;
          onSubmit(values);
          onClose();
        }}
      >
        <div className="stack">
          {danger && (
            <Alert title="Review this action carefully" tone="danger">
              This confirmation models a destructive workflow. Only the current
              UI preview can change.
            </Alert>
          )}
          {fields.map((f) =>
            f.options ? (
              <Select
                key={f.name}
                showLabel
                name={f.name}
                label={f.label}
                defaultValue={f.value}
                required={f.required}
              >
                {f.options.map((o) => (
                  <option key={o}>{o}</option>
                ))}
              </Select>
            ) : f.type === "textarea" ? (
              <Textarea
                key={f.name}
                name={f.name}
                label={f.label}
                defaultValue={f.value}
                required={f.required}
              />
            ) : (
              <Input
                key={f.name}
                name={f.name}
                label={f.label}
                type={f.type ?? "text"}
                defaultValue={f.value}
                required={f.required}
                pattern={f.pattern}
                min={f.min}
                step={f.type === "number" ? "any" : undefined}
              />
            ),
          )}
          <div className="row end">
            <Button onClick={onClose}>Cancel</Button>
            <Button type="submit" variant={danger ? "danger" : "primary"}>
              {submitLabel}
            </Button>
          </div>
        </div>
      </form>
    </Modal>
  );
}
export function OutputPreview({
  title,
  open,
  onClose,
  children,
}: {
  title: string;
  open: boolean;
  onClose: () => void;
  children: ReactNode;
}) {
  return (
    <Modal
      title={title}
      description="Print / export preview only. No printer or filesystem integration."
      open={open}
      onOpenChange={(o) => !o && onClose()}
    >
      <div className="stack">
        {children}
        <div className="row">
          <Button disabled>
            <Printer size={15} />
            Print (not connected)
          </Button>
          <Button disabled>
            <Download size={15} />
            Export (not connected)
          </Button>
        </div>
      </div>
    </Modal>
  );
}
