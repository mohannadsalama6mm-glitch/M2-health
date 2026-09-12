import { useState } from "react";
import { Link } from "react-router-dom";
import {
  FeaturePage,
  RouteTabs,
  DataGrid,
  PriceDisplay,
  Status,
  Button,
  LinkButton,
  WorkflowDialog,
  SummaryStrip,
  Drawer,
  DetailList,
} from "../../design-system";
import { useDemo } from "../../app/DemoContext";
import { movements, movementTypes } from "../../mock/fixtures";
import type { Product } from "../../mock/types";
export const inventoryTabs = [
  { label: "Stock overview", to: "/inventory" },
  { label: "Movements", to: "/inventory/movements" },
  { label: "Stock counts", to: "/inventory/counts" },
  { label: "Expiry management", to: "/inventory/expiry" },
];
export const stockStatus = (p: Product) =>
  p.status === "Inactive"
    ? "Inactive"
    : p.stock === 0
      ? "Out of stock"
      : p.stock < p.reorder
        ? "Low stock"
        : p.days <= 30
          ? "Expiring"
          : "In stock";
export function Inventory() {
  const { products, setProducts, branch, branches, notify } = useDemo();
  const [action, setAction] = useState("");
  return (
    <FeaturePage
      title="Inventory"
      description={`${branch} · package and batch-level stock preview`}
      actions={
        <>
          <LinkButton to="/inventory/counts">Stock count</LinkButton>
          <Button variant="primary" onClick={() => setAction("Adjust stock")}>
            Adjust stock
          </Button>
        </>
      }
    >
      <RouteTabs items={inventoryTabs} />
      <SummaryStrip
        items={[
          {
            label: "In stock",
            value: products.filter(
              (p) => p.stock >= p.reorder && p.status === "Active",
            ).length,
          },
          {
            label: "Low stock",
            value: products.filter((p) => p.stock > 0 && p.stock < p.reorder)
              .length,
          },
          {
            label: "Out of stock",
            value: products.filter((p) => !p.stock).length,
          },
          {
            label: "Inventory value · demo",
            value: (
              <PriceDisplay
                amount={products.reduce((s, p) => s + p.stock * p.cost, 0)}
              />
            ),
          },
        ]}
      />
      <DataGrid
        label="Inventory"
        rows={products}
        rowKey={(p) => p.id}
        searchText={(p) => `${p.name} ${p.batch} ${p.barcode}`}
        filters={[
          {
            label: "Statuses",
            options: [
              "In stock",
              "Low stock",
              "Out of stock",
              "Expiring",
              "Inactive",
            ],
            value: stockStatus,
          },
        ]}
        actions={
          <Button onClick={() => setAction("Transfer stock")}>Transfer</Button>
        }
        columns={[
          {
            key: "name",
            header: "Product / package",
            render: (p) => (
              <Link className="text-link" to={`/catalog/${p.id}`}>
                {p.name}
                <span className="cell-secondary">{p.pack}</span>
              </Link>
            ),
          },
          { key: "batch", header: "Batch", render: (p) => p.batch },
          { key: "stock", header: "Stock", render: (p) => p.stock },
          { key: "reorder", header: "Reorder", render: (p) => p.reorder },
          { key: "expiry", header: "Expiry", render: (p) => p.expiry },
          {
            key: "cost",
            header: "Cost",
            render: (p) => <PriceDisplay amount={p.cost} />,
          },
          {
            key: "price",
            header: "Selling price",
            render: (p) => <PriceDisplay amount={p.price} />,
          },
          {
            key: "status",
            header: "Status",
            render: (p) => <Status value={stockStatus(p)} />,
          },
        ]}
      />
      <WorkflowDialog
        key={action}
        open={!!action}
        onClose={() => setAction("")}
        title={action}
        description="Preview an inventory operation. No actual stock ledger is posted."
        fields={[
          {
            name: "product",
            label: "Product",
            options: products.map((p) => p.name),
          },
          {
            name: "quantity",
            label:
              action === "Adjust stock"
                ? "New stock quantity"
                : "Transfer quantity",
            type: "number",
            min: 0,
            required: true,
            value: "0",
          },
          ...(action === "Transfer stock"
            ? [
                {
                  name: "branch",
                  label: "Destination branch",
                  options: branches
                    .filter((b) => b.name !== branch)
                    .map((b) => b.name),
                },
              ]
            : []),
          {
            name: "reason",
            label: "Reason / reference",
            type: "textarea",
            required: true,
          },
        ]}
        onSubmit={(v) => {
          if (action === "Adjust stock")
            setProducts(
              products.map((p) =>
                p.name === v.product ? { ...p, stock: Number(v.quantity) } : p,
              ),
            );
          notify(
            action === "Adjust stock"
              ? "Stock quantity updated in this preview."
              : "Transfer reviewed. No inventory was moved.",
          );
        }}
      />
    </FeaturePage>
  );
}
export function StockMovements() {
  const [detail, setDetail] = useState<(typeof movements)[number] | null>(null);
  return (
    <FeaturePage
      title="Stock movements"
      description="A chronological ledger of sample inventory activity."
    >
      <RouteTabs items={inventoryTabs} />
      <DataGrid
        label="Stock movements"
        rows={movements}
        rowKey={(r) => r.id}
        searchText={(r) => `${r.product} ${r.id} ${r.reason}`}
        filters={[
          {
            label: "Movement types",
            options: movementTypes,
            value: (r) => r.type,
          },
          {
            label: "Branches",
            options: ["Main branch", "Maadi branch"],
            value: (r) => r.branch,
          },
        ]}
        columns={[
          {
            key: "date",
            header: "Date / reference",
            render: (r) => (
              <Button variant="ghost" size="sm" onClick={() => setDetail(r)}>
                {r.date}
                <span className="cell-secondary">{r.id}</span>
              </Button>
            ),
          },
          { key: "type", header: "Type", render: (r) => r.type },
          {
            key: "product",
            header: "Product / package",
            render: (r) => (
              <>
                {r.product}
                <span className="cell-secondary">{r.pack}</span>
              </>
            ),
          },
          { key: "batch", header: "Batch", render: (r) => r.batch },
          { key: "branch", header: "Branch", render: (r) => r.branch },
          { key: "in", header: "In", render: (r) => r.incoming || "—" },
          { key: "out", header: "Out", render: (r) => r.outgoing || "—" },
          { key: "user", header: "User", render: (r) => r.user },
          { key: "reason", header: "Reason", render: (r) => r.reason },
          {
            key: "status",
            header: "Status",
            render: (r) => <Status value={r.status} />,
          },
        ]}
      />
      <Drawer
        open={!!detail}
        onOpenChange={() => setDetail(null)}
        title={detail?.id ?? "Movement details"}
        description="Illustrative stock ledger entry"
      >
        <DetailList
          items={
            detail
              ? Object.entries(detail).map(([label, value]) => ({
                  label,
                  value,
                }))
              : []
          }
        />
      </Drawer>
    </FeaturePage>
  );
}
