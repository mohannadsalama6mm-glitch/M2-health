import {
  ShoppingCart,
  ReceiptText,
  Package,
  PackageMinus,
  Clock3,
} from "lucide-react";
import { PharmacyStatCard } from "../../design-system";
export function DashboardStats() {
  return (
    <div className="stats-grid">
      <PharmacyStatCard
        label="Today's sales"
        value="18,450"
        unit="EGP"
        detail="vs. 16,470 yesterday"
        trend="12%"
        icon={ShoppingCart}
      />
      <PharmacyStatCard
        label="Transactions"
        value="142"
        detail="11 more than yesterday"
        trend="8.4%"
        icon={ReceiptText}
      />
      <PharmacyStatCard
        label="Total products"
        value="436"
        detail="Across all categories"
        icon={Package}
        tone="info"
      />
      <PharmacyStatCard
        label="Low stock items"
        value="18"
        detail="3 need immediate attention"
        icon={PackageMinus}
        tone="warning"
      />
      <PharmacyStatCard
        label="Expiring soon"
        value="12"
        detail="Within the next 30 days"
        icon={Clock3}
        tone="danger"
      />
    </div>
  );
}
