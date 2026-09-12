import { useState } from "react";
import {
  FeaturePage,
  RouteTabs,
  DataGrid,
  ExpiryBadge,
  PriceDisplay,
  Button,
  SummaryStrip,
  WorkflowDialog,
  Drawer,
  DetailList,
} from "../../design-system";
import { useDemo } from "../../app/DemoContext";
import { inventoryTabs } from "./Inventory";
import type { Product } from "../../mock/types";
export function Expiry() {
  const { products, notify } = useDemo();
  const [window, setWindow] = useState(30),
    [action, setAction] = useState(""),
    [selected, setSelected] = useState<Product | null>(null);
  return (
    <FeaturePage
      title="Expiry management"
      description="Prioritize batches by expiry date and value at risk."
    >
      <RouteTabs items={inventoryTabs} />
      <SummaryStrip
        items={[
          {
            label: "Expired",
            value: products.filter((p) => p.days < 0).length,
          },
          {
            label: "Within 30 days",
            value: products.filter((p) => p.days >= 0 && p.days <= 30).length,
          },
          {
            label: "Within 60 days",
            value: products.filter((p) => p.days > 30 && p.days <= 60).length,
          },
          {
            label: "Within 90 days",
            value: products.filter((p) => p.days > 60 && p.days <= 90).length,
          },
        ]}
      />
      <div className="section-menu">
        {[-1, 30, 60, 90].map((days) => (
          <Button
            key={days}
            variant={window === days ? "primary" : "secondary"}
            onClick={() => setWindow(days)}
          >
            {days === -1 ? "Expired" : `Within ${days} days`}
          </Button>
        ))}
      </div>
      <DataGrid
        label="Expiry batches"
        rows={products.filter((p) =>
          window === -1 ? p.days < 0 : p.days >= 0 && p.days <= window,
        )}
        rowKey={(p) => p.id}
        searchText={(p) => `${p.name} ${p.batch}`}
        columns={[
          {
            key: "name",
            header: "Product / package",
            render: (p) => (
              <>
                {p.name}
                <span className="cell-secondary">{p.pack}</span>
              </>
            ),
          },
          { key: "batch", header: "Batch", render: (p) => p.batch },
          { key: "quantity", header: "Quantity", render: (p) => p.stock },
          { key: "date", header: "Expiry date", render: (p) => p.expiry },
          {
            key: "days",
            header: "Time left",
            render: (p) => <ExpiryBadge days={p.days} />,
          },
          {
            key: "value",
            header: "Value at risk",
            render: (p) => <PriceDisplay amount={p.stock * p.cost} />,
          },
          {
            key: "suggestion",
            header: "Suggested action",
            render: (p) =>
              p.days < 0 ? "Quarantine / write-off" : "Review supplier return",
          },
          {
            key: "actions",
            header: "Actions",
            render: (p) => (
              <Button
                size="sm"
                onClick={() => {
                  setSelected(p);
                  setAction("Review");
                }}
              >
                Review
              </Button>
            ),
          },
        ]}
      />
      <Drawer
        open={action === "Review"}
        onOpenChange={() => setAction("")}
        title={selected?.name ?? "Batch review"}
        description="Expiry workflow preview · no physical or ledger action"
      >
        <div className="stack">
          <DetailList
            items={[
              { label: "Batch", value: selected?.batch },
              { label: "Expiry", value: selected?.expiry },
              { label: "Quantity", value: selected?.stock },
            ]}
          />
          <Button onClick={() => setAction("Return to supplier")}>
            Return to supplier
          </Button>
          <Button variant="danger" onClick={() => setAction("Write off batch")}>
            Write-off
          </Button>
        </div>
      </Drawer>
      <WorkflowDialog
        key={action}
        open={!!action && action !== "Review"}
        onClose={() => setAction("")}
        title={action}
        description={`${selected?.name} · ${selected?.batch}. This models an approval request only.`}
        danger={action === "Write off batch"}
        fields={[
          {
            name: "quantity",
            label: "Quantity to review",
            type: "number",
            min: 1,
            required: true,
            value: String(selected?.stock ?? 1),
          },
          { name: "reason", label: "Reason", type: "textarea", required: true },
        ]}
        onSubmit={() => notify(`${action} reviewed. No stock was changed.`)}
      />
    </FeaturePage>
  );
}
