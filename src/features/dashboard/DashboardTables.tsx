import {
  ReceiptText,
  PackageMinus,
  Trophy,
  ArrowRight,
  Clock3,
} from "lucide-react";
import {
  Card,
  SectionHeader,
  Button,
  Table,
  Badge,
  StockBadge,
  ExpiryBadge,
  PriceDisplay,
} from "../../design-system";
import {
  demoSales,
  demoStock,
  demoProducts,
  demoExpiry,
} from "../../data/demo";
const View = ({ onClick }: { onClick: () => void }) => (
  <Button size="sm" variant="ghost" onClick={onClick}>
    View all <ArrowRight size={13} />
  </Button>
);
export function RecentSales({ onView }: { onView: () => void }) {
  return (
    <Card>
      <SectionHeader
        title="Recent sales"
        icon={ReceiptText}
        action={<View onClick={onView} />}
      />
      <Table
        label="Recent demo sales"
        rows={demoSales}
        rowKey={(r) => r.invoice}
        columns={[
          {
            key: "invoice",
            header: "Invoice",
            render: (r) => <span className="invoice">{r.invoice}</span>,
          },
          { key: "time", header: "Time", render: (r) => r.time },
          { key: "customer", header: "Customer", render: (r) => r.customer },
          { key: "items", header: "Items", render: (r) => r.items },
          {
            key: "total",
            header: "Total",
            render: (r) => <PriceDisplay amount={r.total} />,
          },
          { key: "payment", header: "Payment", render: (r) => r.payment },
          {
            key: "status",
            header: "Status",
            render: () => <Badge tone="success">Completed</Badge>,
          },
        ]}
      />
    </Card>
  );
}
export function StockAlerts({ onView }: { onView: () => void }) {
  return (
    <Card>
      <SectionHeader
        title="Low stock alerts"
        icon={PackageMinus}
        action={<View onClick={onView} />}
      />
      <Table
        label="Demo low stock alerts"
        rows={demoStock}
        rowKey={(r) => r.name}
        columns={[
          { key: "name", header: "Product", render: (r) => r.name },
          {
            key: "stock",
            header: "Stock",
            render: (r) => (
              <strong className={r.stock <= 3 ? "text-danger" : ""}>
                {r.stock}
              </strong>
            ),
          },
          { key: "min", header: "Min.", render: (r) => r.min },
          {
            key: "status",
            header: "Status",
            render: (r) => (
              <StockBadge status={r.stock <= 3 ? "Critical" : "Low"} />
            ),
          },
        ]}
      />
    </Card>
  );
}
export function TopProducts({ onView }: { onView: () => void }) {
  return (
    <Card>
      <SectionHeader
        title="Top selling products"
        icon={Trophy}
        action={<View onClick={onView} />}
      />
      <Table
        label="Demo top selling products"
        rows={demoProducts}
        rowKey={(r) => r.name}
        columns={[
          {
            key: "name",
            header: "Product",
            render: (r) => (
              <div>
                {r.name}
                <div className="muted product-category">{r.category}</div>
              </div>
            ),
          },
          { key: "units", header: "Units sold", render: (r) => r.units },
          {
            key: "revenue",
            header: "Revenue",
            render: (r) => <PriceDisplay amount={r.revenue} />,
          },
        ]}
      />
    </Card>
  );
}
export function ExpiryAlerts({ onView }: { onView: () => void }) {
  return (
    <Card>
      <SectionHeader
        title="Expiry watch"
        icon={Clock3}
        action={<View onClick={onView} />}
      />
      <Table
        label="Demo expiry alerts"
        rows={demoExpiry}
        rowKey={(r) => r.name}
        columns={[
          { key: "name", header: "Product", render: (r) => r.name },
          { key: "date", header: "Expiry date", render: (r) => r.date },
          {
            key: "days",
            header: "Time left",
            render: (r) => <ExpiryBadge days={r.days} />,
          },
        ]}
      />
      <div className="expiry-note">
        <span className="status-dot" /> 1 product batch needs a review this week
      </div>
    </Card>
  );
}
