import { Minus, Plus, Package, type LucideIcon } from "lucide-react";
import { Badge, Card, IconButton, SearchInput, StatCard } from "./core";
export const PharmacyStatCard = StatCard;
export const ProductSearch = SearchInput;
export function StockBadge({
  status,
}: {
  status: "In stock" | "Low" | "Critical" | "Out of stock";
}) {
  return (
    <Badge
      dot
      tone={
        status === "In stock"
          ? "success"
          : status === "Low"
            ? "warning"
            : "danger"
      }
    >
      {status}
    </Badge>
  );
}
export function ExpiryBadge({ days }: { days: number }) {
  return (
    <Badge tone={days <= 7 ? "danger" : days <= 30 ? "warning" : "success"}>
      {days <= 0 ? "Expired" : `${days} days`}
    </Badge>
  );
}
export function PriceDisplay({ amount }: { amount: number }) {
  return (
    <span className="price">
      {amount.toLocaleString("en-US", { minimumFractionDigits: 2 })}{" "}
      <small>EGP</small>
    </span>
  );
}
export function QuantityStepper({
  value,
  onChange,
  min = 0,
  max = 999,
}: {
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
}) {
  return (
    <div className="stepper">
      <IconButton
        label="Decrease quantity"
        disabled={value <= min}
        onClick={() => onChange(value - 1)}
      >
        <Minus size={14} />
      </IconButton>
      <input
        type="number"
        aria-label="Quantity"
        min={min}
        max={max}
        value={value}
        onChange={(e) =>
          onChange(Math.min(max, Math.max(min, Number(e.target.value))))
        }
      />
      <IconButton
        label="Increase quantity"
        disabled={value >= max}
        onClick={() => onChange(value + 1)}
      >
        <Plus size={14} />
      </IconButton>
    </div>
  );
}
export function BarcodeDisplay({ value }: { value: string }) {
  return (
    <div className="barcode">
      <div aria-hidden="true" />
      <span>{value}</span>
      <small>Illustrative barcode · not for scanning</small>
    </div>
  );
}
export function ProductCard({
  name,
  subtitle,
  price,
}: {
  name: string;
  subtitle: string;
  price: number;
}) {
  return (
    <Card className="product-card">
      <span className="product-art">
        <Package size={32} />
      </span>
      <div>
        <h3>{name}</h3>
        <p className="muted">{subtitle}</p>
        <PriceDisplay amount={price} />
      </div>
      <StockBadge status="In stock" />
    </Card>
  );
}
export function QuickActionCard({
  label,
  icon: Icon,
  onClick,
}: {
  label: string;
  icon: LucideIcon;
  onClick: () => void;
}) {
  return (
    <button className="quick-action" onClick={onClick}>
      <Icon size={21} />
      <span>{label}</span>
    </button>
  );
}
