import {
  useId,
  useState,
  useRef,
  type ReactNode,
  type ButtonHTMLAttributes,
  type InputHTMLAttributes,
  type SelectHTMLAttributes,
  type TextareaHTMLAttributes,
} from "react";
import * as Dialog from "@radix-ui/react-dialog";
import * as Menu from "@radix-ui/react-dropdown-menu";
import * as Tip from "@radix-ui/react-tooltip";
import {
  Search,
  X,
  LoaderCircle,
  ChevronLeft,
  ChevronRight,
  Check,
  Info,
  PackageOpen,
  ArrowUpRight,
  type LucideIcon,
} from "lucide-react";
export type Tone = "success" | "warning" | "danger" | "info" | "neutral";
export function Spinner() {
  return (
    <LoaderCircle
      size={16}
      className="spinner"
      aria-label="Loading"
      role="status"
    />
  );
}
export function Button({
  variant = "secondary",
  size = "md",
  loading = false,
  children,
  className = "",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: "primary" | "secondary" | "outline" | "ghost" | "danger";
  size?: "sm" | "md" | "lg";
  loading?: boolean;
}) {
  return (
    <button
      type="button"
      {...props}
      disabled={props.disabled || loading}
      aria-busy={loading}
      className={`button ${variant} ${size} ${className}`}
    >
      {loading && <Spinner />}
      {children}
    </button>
  );
}
export function IconButton({
  label,
  children,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { label: string }) {
  return (
    <button
      type="button"
      {...props}
      aria-label={label}
      title={label}
      className={`icon-button ${props.className ?? ""}`}
    >
      {children}
    </button>
  );
}
export function Input({
  label,
  hideLabel = false,
  error,
  success,
  ...props
}: InputHTMLAttributes<HTMLInputElement> & {
  label: string;
  hideLabel?: boolean;
  error?: string;
  success?: string;
}) {
  const id = useId();
  return (
    <label className="field" htmlFor={id}>
      <span className={hideLabel ? "sr-only" : undefined}>{label}</span>
      <input
        {...props}
        id={id}
        aria-invalid={!!error}
        aria-describedby={error || success ? `${id}-hint` : undefined}
        className={
          (props.className ?? "") +
          " " +
          (error ? "invalid" : success ? "valid" : "")
        }
      />
      {(error || success) && (
        <small
          id={`${id}-hint`}
          className={error ? "text-danger" : "text-success"}
        >
          {error || success}
        </small>
      )}
    </label>
  );
}
export function SearchInput({
  label = "Search",
  ...props
}: InputHTMLAttributes<HTMLInputElement> & { label?: string }) {
  return (
    <div className="search-field">
      <Search size={16} />
      <input {...props} type="search" aria-label={label} />
    </div>
  );
}
export function Textarea({
  label,
  ...props
}: TextareaHTMLAttributes<HTMLTextAreaElement> & { label: string }) {
  return (
    <label className="field">
      <span>{label}</span>
      <textarea {...props} />
    </label>
  );
}
export function Select({
  label,
  showLabel = false,
  children,
  ...props
}: SelectHTMLAttributes<HTMLSelectElement> & {
  label: string;
  showLabel?: boolean;
}) {
  return (
    <label className={`select-field ${showLabel ? "field" : ""}`}>
      <span className={showLabel ? undefined : "sr-only"}>{label}</span>
      <select {...props} aria-label={label}>
        {children}
      </select>
    </label>
  );
}
export function Checkbox({
  label,
  ...props
}: InputHTMLAttributes<HTMLInputElement> & { label: string }) {
  return (
    <label className="check-field">
      <input {...props} type="checkbox" />
      {label}
    </label>
  );
}
export function Radio({
  label,
  ...props
}: InputHTMLAttributes<HTMLInputElement> & { label: string }) {
  return (
    <label className="check-field">
      <input {...props} type="radio" />
      {label}
    </label>
  );
}
export function Switch({
  label,
  checked,
  onChange,
  disabled = false,
}: {
  label: string;
  checked: boolean;
  onChange: (value: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <label className="check-field">
      <button
        type="button"
        className="switch"
        role="switch"
        aria-label={label}
        aria-checked={checked}
        disabled={disabled}
        onClick={() => onChange(!checked)}
      >
        <span />
      </button>
      {label}
    </label>
  );
}
export function Card({
  children,
  className = "",
}: {
  children: ReactNode;
  className?: string;
}) {
  return <section className={`card ${className}`}>{children}</section>;
}
export function Badge({
  children,
  tone = "neutral",
  dot = false,
}: {
  children: ReactNode;
  tone?: Tone;
  dot?: boolean;
}) {
  return (
    <span className={`badge ${tone}`}>
      {dot && <span className="status-dot" />}
      {children}
    </span>
  );
}
export const StatusBadge = Badge;
export function StatCard({
  label,
  value,
  unit,
  detail,
  trend,
  icon: Icon,
  tone = "success",
}: {
  label: string;
  value: string;
  unit?: string;
  detail: string;
  trend?: string;
  icon: LucideIcon;
  tone?: Tone;
}) {
  return (
    <Card className="stat-card">
      <div className="stat-top">
        <span>{label}</span>
        <span className={`stat-icon ${tone}`}>
          <Icon size={19} />
        </span>
      </div>
      <div className="stat-value">
        {value}
        <small>{unit}</small>
      </div>
      <div className="stat-foot">
        <span>{detail}</span>
        {trend && (
          <Badge tone={tone}>
            <ArrowUpRight size={12} />
            {trend}
          </Badge>
        )}
      </div>
    </Card>
  );
}
export function SectionHeader({
  title,
  icon: Icon,
  action,
  subtitle,
}: {
  title: string;
  icon?: LucideIcon;
  action?: ReactNode;
  subtitle?: string;
}) {
  return (
    <header className="section-header">
      <div>
        {Icon && <Icon size={19} />}
        <div>
          <h3>{title}</h3>
          {subtitle && <p className="muted">{subtitle}</p>}
        </div>
      </div>
      {action}
    </header>
  );
}
export function PageHeader({
  title,
  description,
  actions,
}: {
  title: string;
  description?: string;
  actions?: ReactNode;
}) {
  return (
    <header className="page-header">
      <div>
        <h1>{title}</h1>
        {description && <p>{description}</p>}
      </div>
      <div className="row">{actions}</div>
    </header>
  );
}
export type Column<T> = {
  key: string;
  header: ReactNode;
  render: (row: T) => ReactNode;
};
export function Table<T>({
  columns,
  rows,
  rowKey,
  label,
}: {
  columns: Column<T>[];
  rows: T[];
  rowKey: (row: T) => string;
  label: string;
}) {
  return (
    <div
      className="table-scroll"
      tabIndex={0}
      aria-label={`${label} scroll area`}
    >
      <table>
        <caption className="sr-only">{label}</caption>
        <thead>
          <tr>
            {columns.map((c) => (
              <th key={c.key} scope="col">
                {c.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={rowKey(row)}>
              {columns.map((c) => (
                <td key={c.key}>{c.render(row)}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      {rows.length === 0 && (
        <EmptyState
          title="No matching results"
          description="Try another search term."
        />
      )}
    </div>
  );
}
export function TableToolbar({ children }: { children: ReactNode }) {
  return <div className="table-toolbar">{children}</div>;
}
export function Pagination({
  page,
  pages,
  onChange,
}: {
  page: number;
  pages: number;
  onChange: (page: number) => void;
}) {
  const visible = Array.from(
    new Set([1, pages, ...Array.from({ length: 5 }, (_, i) => page + i - 2)]),
  )
    .filter((n) => n >= 1 && n <= pages)
    .sort((a, b) => a - b);
  const steps: (number | string)[] = [];
  visible.forEach((n, i) => {
    if (i > 0 && n - visible[i - 1] > 1) steps.push(`gap-${n}`);
    steps.push(n);
  });
  return (
    <nav className="pagination" aria-label="Pagination">
      <IconButton
        label="Previous page"
        disabled={page <= 1}
        onClick={() => onChange(page - 1)}
      >
        <ChevronLeft size={16} />
      </IconButton>
      {steps.map((n) =>
        typeof n === "string" ? (
          <span key={n} aria-hidden="true">
            …
          </span>
        ) : (
          <Button
            key={n}
            size="sm"
            variant={page === n ? "primary" : "ghost"}
            aria-current={page === n ? "page" : undefined}
            onClick={() => onChange(n)}
          >
            {n}
          </Button>
        ),
      )}
      <IconButton
        label="Next page"
        disabled={page >= pages}
        onClick={() => onChange(page + 1)}
      >
        <ChevronRight size={16} />
      </IconButton>
    </nav>
  );
}
export function Tabs({
  items,
}: {
  items: { label: string; content: ReactNode }[];
}) {
  const [active, setActive] = useState(0);
  const id = useId();
  return (
    <div>
      <div role="tablist" className="tabs" aria-label="Detail tabs">
        {items.map((item, i) => (
          <button
            key={item.label}
            id={`${id}-tab-${i}`}
            aria-controls={`${id}-panel-${i}`}
            role="tab"
            tabIndex={i === active ? 0 : -1}
            aria-selected={i === active}
            onClick={() => setActive(i)}
            onKeyDown={(e) => {
              const next =
                e.key === "ArrowRight"
                  ? (i + 1) % items.length
                  : e.key === "ArrowLeft"
                    ? (i - 1 + items.length) % items.length
                    : e.key === "Home"
                      ? 0
                      : e.key === "End"
                        ? items.length - 1
                        : null;
              if (next !== null) {
                e.preventDefault();
                setActive(next);
                document.getElementById(`${id}-tab-${next}`)?.focus();
              }
            }}
          >
            {item.label}
          </button>
        ))}
      </div>
      <div
        role="tabpanel"
        id={`${id}-panel-${active}`}
        aria-labelledby={`${id}-tab-${active}`}
        className="tab-panel"
      >
        {items[active].content}
      </div>
    </div>
  );
}
export function Modal({
  open,
  onOpenChange,
  title,
  description,
  children,
  drawer = false,
  size = "md",
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description: string;
  children: ReactNode;
  drawer?: boolean;
  size?: "md" | "lg";
}) {
  const returnFocus = useRef<HTMLElement | null>(null);
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="overlay" />
        <Dialog.Content
          onOpenAutoFocus={() => {
            returnFocus.current = document.activeElement as HTMLElement;
          }}
          onCloseAutoFocus={(event) => {
            event.preventDefault();
            returnFocus.current?.focus();
          }}
          className={`modal ${drawer ? "drawer" : ""} ${size === "lg" ? "modal-wide" : ""}`}
        >
          <Dialog.Title>{title}</Dialog.Title>
          <Dialog.Description>{description}</Dialog.Description>
          <Dialog.Close asChild>
            <IconButton label="Close dialog" className="modal-close">
              <X size={18} />
            </IconButton>
          </Dialog.Close>
          <div className="modal-body">{children}</div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
export function Drawer(props: Omit<Parameters<typeof Modal>[0], "drawer">) {
  return <Modal {...props} drawer />;
}
export function ConfirmationModal({
  onConfirm,
  ...props
}: Omit<Parameters<typeof Modal>[0], "children"> & { onConfirm: () => void }) {
  return (
    <Modal {...props}>
      <div className="row end">
        <Button onClick={() => props.onOpenChange(false)}>Cancel</Button>
        <Button
          variant="danger"
          onClick={() => {
            onConfirm();
            props.onOpenChange(false);
          }}
        >
          Confirm
        </Button>
      </div>
    </Modal>
  );
}
export function Dropdown({
  label,
  items,
}: {
  label: ReactNode;
  items: { label: string; onSelect: () => void }[];
}) {
  return (
    <Menu.Root>
      <Menu.Trigger asChild>
        <button className="dropdown-trigger">{label}</button>
      </Menu.Trigger>
      <Menu.Portal>
        <Menu.Content className="dropdown-content" sideOffset={8} align="end">
          {items.map((item) => (
            <Menu.Item key={item.label} onSelect={item.onSelect}>
              {item.label}
            </Menu.Item>
          ))}
        </Menu.Content>
      </Menu.Portal>
    </Menu.Root>
  );
}
export function Tooltip({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <Tip.Provider delayDuration={200}>
      <Tip.Root>
        <Tip.Trigger asChild>{children}</Tip.Trigger>
        <Tip.Portal>
          <Tip.Content className="tooltip" sideOffset={8}>
            {label}
            <Tip.Arrow />
          </Tip.Content>
        </Tip.Portal>
      </Tip.Root>
    </Tip.Provider>
  );
}
export function Alert({
  title,
  children,
  tone = "info",
}: {
  title: string;
  children?: ReactNode;
  tone?: Tone;
}) {
  return (
    <div
      className={`alert ${tone}`}
      role={tone === "danger" ? "alert" : "status"}
    >
      <Info size={18} />
      <div>
        <strong>{title}</strong>
        {children && <p>{children}</p>}
      </div>
    </div>
  );
}
export function Toast({
  message,
  onClose,
}: {
  message: string;
  onClose: () => void;
}) {
  return (
    <div className="toast" role="status">
      <Check size={18} />
      <span>{message}</span>
      <IconButton label="Dismiss notification" onClick={onClose}>
        <X size={16} />
      </IconButton>
    </div>
  );
}
export function Skeleton() {
  return (
    <div className="skeleton" aria-label="Loading content">
      <span />
      <span />
      <span />
    </div>
  );
}
export function Progress({ value, label }: { value: number; label: string }) {
  return (
    <div className="progress-row">
      <progress aria-label={label} max={100} value={value} />
      <span>{value}%</span>
    </div>
  );
}
export function Divider() {
  return <hr className="divider" />;
}
export function Avatar({ name }: { name: string }) {
  return (
    <span className="avatar" aria-label={name}>
      {name.charAt(0)}
    </span>
  );
}
export function EmptyState({
  title,
  description,
  action,
}: {
  title: string;
  description: string;
  action?: ReactNode;
}) {
  return (
    <div className="empty-state">
      <PackageOpen size={32} />
      <h3>{title}</h3>
      <p>{description}</p>
      {action}
    </div>
  );
}
export function Breadcrumbs({ items }: { items: string[] }) {
  return (
    <nav aria-label="Breadcrumb" className="breadcrumbs">
      {items.map((item, i) => (
        <span key={item}>
          {i > 0 && <ChevronRight size={12} />}
          <span aria-current={i === items.length - 1 ? "page" : undefined}>
            {item}
          </span>
        </span>
      ))}
    </nav>
  );
}
