import {
  CalendarDays,
  ChevronDown,
  Plus,
  Database,
  HardDrive,
  Monitor,
  ScanBarcode,
  Activity,
  ArrowUpRight,
} from "lucide-react";
import {
  PageHeader,
  Button,
  Card,
  SectionHeader,
  Badge,
} from "../../design-system";
import { DashboardStats } from "./DashboardStats";
import { SalesOverview } from "./SalesOverview";
import { InventoryStatus } from "./InventoryStatus";
import {
  RecentSales,
  StockAlerts,
  TopProducts,
  ExpiryAlerts,
} from "./DashboardTables";
import { QuickActions } from "./QuickActions";
export function Dashboard({ onAction }: { onAction: (label: string) => void }) {
  return (
    <>
      <PageHeader
        title="Good morning, Mohannad"
        description="A clear view of your pharmacy, all in one place."
        actions={
          <>
            <Button onClick={() => onAction("Demo date")}>
              <CalendarDays size={15} /> 11 Sep 2026 <ChevronDown size={13} />
            </Button>
            <Button variant="primary" onClick={() => onAction("New sale")}>
              <Plus size={16} /> New sale
            </Button>
          </>
        }
      />
      <div className="workspace-caption">
        <span>
          <span className="status-dot" /> Main branch{" "}
          <span className="caption-separator">/</span> Daily overview
        </span>
        <Badge tone="warning">UI demo · sample data</Badge>
      </div>
      <DashboardStats />
      <div className="dashboard-row overview-row">
        <SalesOverview />
        <InventoryStatus />
        <QuickActions onAction={onAction} />
      </div>
      <div className="dashboard-row tables-row">
        <RecentSales onView={() => onAction("Sales history")} />
        <StockAlerts onView={() => onAction("Low stock inventory")} />
      </div>
      <div className="dashboard-row bottom-row">
        <TopProducts onView={() => onAction("Product performance")} />
        <ExpiryAlerts onView={() => onAction("Expiry report")} />
        <Card>
          <SectionHeader
            title="Workspace status"
            icon={Activity}
            action={<Badge>Preview</Badge>}
          />
          <div className="system-list">
            {[
              { label: "Database", icon: Database },
              { label: "Backup", icon: HardDrive },
              { label: "POS terminals", icon: Monitor },
              { label: "Barcode scanner", icon: ScanBarcode },
            ].map(({ label, icon: Icon }) => (
              <div key={label}>
                <Icon size={16} />
                <span>{label}</span>
                <Badge>Not connected</Badge>
              </div>
            ))}
          </div>
          <div className="card-note">
            UI foundation <ArrowUpRight size={13} />
          </div>
        </Card>
      </div>
      <footer className="main-footer">
        <span>
          M² Health <span className="caption-separator">/</span> Your pharmacy.
          In focus.
        </span>
        <span>Better pharmacies. Healthier communities.</span>
      </footer>
    </>
  );
}
