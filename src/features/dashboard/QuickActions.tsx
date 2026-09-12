import {
  Zap,
  ShoppingCart,
  PackagePlus,
  Truck,
  ClipboardList,
  Barcode,
  ChartNoAxesCombined,
} from "lucide-react";
import { Card, SectionHeader, QuickActionCard } from "../../design-system";
export function QuickActions({
  onAction,
}: {
  onAction: (label: string) => void;
}) {
  return (
    <Card>
      <SectionHeader title="Quick actions" icon={Zap} />
      <div className="quick-grid">
        {[
          { label: "New sale", icon: ShoppingCart },
          { label: "Add product", icon: PackagePlus },
          { label: "Receive purchase", icon: Truck },
          { label: "Stock count", icon: ClipboardList },
          { label: "Print barcode", icon: Barcode },
          { label: "View reports", icon: ChartNoAxesCombined },
        ].map((a) => (
          <QuickActionCard
            key={a.label}
            {...a}
            onClick={() => onAction(a.label)}
          />
        ))}
      </div>
    </Card>
  );
}
