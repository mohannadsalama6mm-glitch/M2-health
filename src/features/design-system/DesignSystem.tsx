import { useState } from "react";
import { WorkspaceShowcase } from "./WorkspaceShowcase";
import {
  Plus,
  Search,
  Package,
  ShoppingCart,
  Settings,
  Bell,
  Users,
  HeartPulse,
  Check,
  X,
  Clock,
  Truck,
  Barcode,
  LayoutDashboard,
} from "lucide-react";
import * as DS from "../../design-system";
import { demoSales } from "../../data/demo";
import "./showcase.css";
const colors = [
  "brand-deep",
  "primary",
  "primary-hover",
  "emerald",
  "mint",
  "primary-soft",
  "primary-subtle",
  "bg",
  "surface",
  "border",
  "text",
  "text-secondary",
  "text-muted",
  "success",
  "warning",
  "danger",
  "info",
];
export function DesignSystem({
  notify,
}: {
  notify: (message: string) => void;
}) {
  const [enabled, setEnabled] = useState(true);
  const [page, setPage] = useState(1);
  const [quantity, setQuantity] = useState(1);
  const [modal, setModal] = useState("");
  const [rtl, setRtl] = useState(false);
  const [search, setSearch] = useState("");
  return (
    <div className="showcase" dir={rtl ? "rtl" : "ltr"}>
      <DS.Breadcrumbs items={["Developer workspace", "Design system"]} />
      <DS.PageHeader
        title="The M² Health design system"
        description="One visual language. Every pharmacy workflow."
        actions={
          <>
            <DS.Badge tone="warning">Internal · development only</DS.Badge>
            <DS.Switch label="RTL preview" checked={rtl} onChange={setRtl} />
          </>
        }
      />
      <DS.Alert title="UI foundation / v0.1">
        Interactive component reference. All examples use sample data. This page
        is excluded from production routing and pharmacy navigation.
      </DS.Alert>
      <div className="showcase-grid">
        <DS.Card className="wide">
          <DS.SectionHeader title="01 / Brand & identity" />
          <div className="brand-examples">
            <DS.Logo />
            <DS.Logo variant="compact" />
            <DS.Logo variant="icon" />
            <DS.Logo dark />
          </div>
          <p className="muted">
            Emerald M · superscript ² · integrated medical cross. Better
            pharmacies. Healthier communities.
          </p>
        </DS.Card>
        <DS.Card>
          <DS.SectionHeader title="02 / Semantic colors" />
          <div className="swatches">
            {colors.map((c) => (
              <div key={c}>
                <span style={{ background: `var(--color-${c})` }} />
                <code>{c}</code>
              </div>
            ))}
          </div>
        </DS.Card>
        <DS.Card>
          <DS.SectionHeader title="03 / Typography" />
          {[
            ["display", "Display · M² Health"],
            ["h1", "H1 · Pharmacy workspace"],
            ["h2", "H2 · Sales overview"],
            ["h3", "H3 · Inventory alerts"],
            ["body-lg", "Body large · A clearer working day"],
            ["body", "Body · Smart Pharmacy Management"],
            ["small", "Small · Product information"],
            ["caption", "Caption · Sample data only"],
          ].map(([token, label]) => (
            <div className="type-row" key={token}>
              <span style={{ fontSize: `var(--text-${token})` }}>{label}</span>
            </div>
          ))}
          <div className="type-row" style={{ fontWeight: 500 }}>
            Body medium · Product details
          </div>
          <p lang="ar" dir="rtl" className="arabic-sample">
            صيدليات أكثر صحة. مجتمعات أكثر عافية.
          </p>
          <p className="muted">
            IBM Plex Sans Arabic · locally bundled · 400 / 500 / 600
          </p>
        </DS.Card>
        <DS.Card>
          <DS.SectionHeader title="04 / Spacing, radius & elevation" />
          <div className="spacing-demo">
            {[1, 2, 3, 4, 5, 6, 8, 10, 12, 16].map((n) => (
              <div key={n}>
                <span style={{ height: `var(--space-${n})` }} />
                <small>{n * 4}</small>
              </div>
            ))}
          </div>
          <div className="row radius-demo">
            {["sm", "md", "lg", "xl", "full"].map((r) => (
              <div key={r} style={{ borderRadius: `var(--radius-${r})` }}>
                {r}
              </div>
            ))}
          </div>
          <div className="row shadows-demo">
            {["sm", "md", "lg"].map((s) => (
              <div key={s} style={{ boxShadow: `var(--shadow-${s})` }}>
                {s}
              </div>
            ))}
          </div>
        </DS.Card>
        <DS.Card>
          <DS.SectionHeader title="05 / Lucide iconography" />
          <div className="icon-demo">
            {[
              Plus,
              Search,
              Package,
              ShoppingCart,
              Settings,
              Bell,
              Users,
              HeartPulse,
              Check,
              X,
              Clock,
              Truck,
              Barcode,
              LayoutDashboard,
            ].map((Icon, i) => (
              <DS.Tooltip key={i} label={`Icon sample ${i + 1}`}>
                <DS.IconButton
                  label={`Icon sample ${i + 1}`}
                  onClick={() => notify("Icon button activated")}
                >
                  <Icon size={20} />
                </DS.IconButton>
              </DS.Tooltip>
            ))}
          </div>
          <p className="muted">
            16 / 18 / 20 / 24 px. Consistent strokes and accessible labels.
          </p>
        </DS.Card>
        <DS.Card className="wide">
          <DS.SectionHeader title="06 / Buttons & interaction states" />
          <div className="button-matrix">
            {(
              ["primary", "secondary", "outline", "ghost", "danger"] as const
            ).map((variant) => (
              <div className="row" key={variant}>
                <code>{variant}</code>
                <DS.Button
                  variant={variant}
                  onClick={() => notify(`${variant} button activated`)}
                >
                  Default
                </DS.Button>
                <DS.Button variant={variant} className="demo-hover">
                  Hover
                </DS.Button>
                <DS.Button variant={variant} className="demo-focus">
                  Focus
                </DS.Button>
                <DS.Button variant={variant} className="demo-active">
                  Active
                </DS.Button>
                <DS.Button variant={variant} disabled>
                  Disabled
                </DS.Button>
                <DS.Button variant={variant} loading>
                  Loading
                </DS.Button>
              </div>
            ))}
          </div>
          <DS.Divider />
          <div className="row">
            {(["sm", "md", "lg"] as const).map((size) => (
              <DS.Button
                key={size}
                size={size}
                variant="primary"
                onClick={() => notify(`${size} button activated`)}
              >
                <Plus size={16} />
                {size} button
              </DS.Button>
            ))}
            <DS.Tooltip label="Create a sample product">
              <DS.IconButton
                label="Create sample product"
                onClick={() => notify("Sample action only")}
              >
                <Plus size={20} />
              </DS.IconButton>
            </DS.Tooltip>
          </div>
        </DS.Card>
        <DS.Card>
          <DS.SectionHeader title="07 / Fields & selection" />
          <div className="field-grid">
            <DS.Input label="Default" placeholder="Product name" />
            <DS.Input
              label="Focus example"
              className="demo-focus"
              placeholder="Use Tab to focus"
            />
            <DS.Input label="Filled" defaultValue="Panadol 500 mg" />
            <DS.Input label="Disabled" disabled placeholder="Not available" />
            <DS.Input
              label="Error"
              error="Enter a valid product code."
              defaultValue="??"
            />
            <DS.Input
              label="Success"
              success="Product code is available."
              defaultValue="PRD-1042"
            />
          </div>
          <DS.Divider />
          <DS.ProductSearch placeholder="Search medicines…" />
          <div className="field-grid controls-demo">
            <DS.Select label="Product category">
              <option>Medicines</option>
              <option>Vitamins</option>
            </DS.Select>
            <DS.Checkbox label="Track stock" defaultChecked />
            <DS.Radio name="unit" label="Box" defaultChecked />
            <DS.Radio name="unit" label="Strip" />
            <DS.Switch
              label="Expiry alerts"
              checked={enabled}
              onChange={setEnabled}
            />
            <DS.Checkbox label="Disabled option" disabled />
          </div>
          <DS.Textarea label="Notes" placeholder="Additional product details" />
        </DS.Card>
        <DS.Card>
          <DS.SectionHeader title="08 / Cards, status & badges" />
          <DS.StatCard
            label="Total sales"
            value="18,450"
            unit="EGP"
            detail="Sample daily sales"
            trend="12%"
            icon={ShoppingCart}
          />
          <DS.Divider />
          <div className="row">
            {(["success", "warning", "danger", "info", "neutral"] as const).map(
              (tone) => (
                <DS.Badge key={tone} tone={tone} dot>
                  {tone}
                </DS.Badge>
              ),
            )}
          </div>
          <DS.Divider />
          <DS.Card>
            <DS.SectionHeader title="Reusable content card" icon={Package} />
            <p className="muted">Consistent surfaces, borders, and spacing.</p>
          </DS.Card>
          <DS.Divider />
          <div className="row">
            <DS.Avatar name="Mohannad" />
            <DS.StatusBadge tone="success" dot>
              Available
            </DS.StatusBadge>
          </div>
        </DS.Card>
        <DS.Card className="wide">
          <DS.SectionHeader title="09 / Tables & pagination" />
          <DS.TableToolbar>
            <DS.SearchInput
              label="Filter demo customers"
              placeholder="Filter demo customers…"
              value={search}
              onChange={(e) => {
                setSearch(e.target.value);
                setPage(1);
              }}
            />
            <DS.Badge>5 demo transactions</DS.Badge>
          </DS.TableToolbar>
          <DS.Table
            label="Component demo transactions"
            rows={demoSales
              .filter((r) =>
                r.customer.toLowerCase().includes(search.toLowerCase()),
              )
              .slice((page - 1) * 3, page * 3)}
            rowKey={(r) => r.invoice}
            columns={[
              { key: "id", header: "Invoice", render: (r) => r.invoice },
              {
                key: "customer",
                header: "Customer",
                render: (r) => r.customer,
              },
              {
                key: "total",
                header: "Total",
                render: (r) => <DS.PriceDisplay amount={r.total} />,
              },
              {
                key: "status",
                header: "Status",
                render: () => <DS.Badge tone="success">Completed</DS.Badge>,
              },
            ]}
          />
          <DS.Pagination
            page={page}
            pages={Math.max(
              1,
              Math.ceil(
                demoSales.filter((r) =>
                  r.customer.toLowerCase().includes(search.toLowerCase()),
                ).length / 3,
              ),
            )}
            onChange={setPage}
          />
        </DS.Card>
        <DS.Card>
          <DS.SectionHeader title="10 / Navigation & overlays" />
          <DS.Tabs
            items={[
              { label: "Product", content: "Product details appear here." },
              { label: "Pricing", content: <DS.PriceDisplay amount={18.5} /> },
              { label: "Stock", content: <DS.StockBadge status="In stock" /> },
            ]}
          />
          <div className="row">
            <DS.Button onClick={() => setModal("modal")}>Open modal</DS.Button>
            <DS.Button onClick={() => setModal("confirm")}>
              Confirmation
            </DS.Button>
            <DS.Button onClick={() => setModal("drawer")}>
              Open drawer
            </DS.Button>
            <DS.Dropdown
              label={
                <>
                  Actions <Plus size={13} />
                </>
              }
              items={[
                {
                  label: "Preview action",
                  onSelect: () => notify("Preview action selected"),
                },
                {
                  label: "Inspect component",
                  onSelect: () => setModal("modal"),
                },
              ]}
            />
          </div>
        </DS.Card>
        <DS.Card>
          <DS.SectionHeader title="11 / Feedback & loading" />
          <div className="stack">
            {(["success", "warning", "danger", "info"] as const).map((tone) => (
              <DS.Alert
                key={tone}
                tone={tone}
                title={`${tone.charAt(0).toUpperCase() + tone.slice(1)} message`}
              >
                An example of contextual feedback.
              </DS.Alert>
            ))}
            <DS.Progress value={65} label="Sample progress" />
            <DS.Skeleton />
            <div className="row">
              <DS.Spinner />
              <DS.Button
                onClick={() =>
                  notify(
                    "Your sample changes were acknowledged. No data was saved.",
                  )
                }
              >
                Show toast
              </DS.Button>
            </div>
          </div>
        </DS.Card>
        <DS.Card>
          <DS.SectionHeader title="12 / Pharmacy components" />
          <div className="stack">
            <DS.ProductCard
              name="Panadol 500 mg"
              subtitle="Paracetamol · 24 tablets"
              price={18.5}
            />
            <div className="row">
              <DS.StockBadge status="In stock" />
              <DS.StockBadge status="Low" />
              <DS.StockBadge status="Out of stock" />
              <DS.ExpiryBadge days={7} />
              <DS.ExpiryBadge days={22} />
            </div>
            <div className="row">
              <DS.QuantityStepper value={quantity} onChange={setQuantity} />
              <DS.PriceDisplay amount={18.5 * quantity} />
            </div>
            <DS.BarcodeDisplay value="1234567890123" />
            <DS.QuickActionCard
              label="New sale"
              icon={ShoppingCart}
              onClick={() => notify("Sales is a future-phase workflow.")}
            />
          </div>
        </DS.Card>
        <DS.Card>
          <DS.SectionHeader title="13 / Empty state & layout anatomy" />
          <DS.EmptyState
            title="Your catalog starts here"
            description="Products will appear here after the catalog phase."
            action={
              <DS.Button
                onClick={() => notify("Catalog is planned for a later phase.")}
                variant="outline"
              >
                <Plus size={15} /> Add product
              </DS.Button>
            }
          />
          <div className="anatomy">
            <aside>Sidebar</aside>
            <header>Topbar · 64 px</header>
            <section>Page header / actions</section>
            <article>
              Primary content area
              <br />
              <small>Reusable cards, tables, and patterns</small>
            </article>
          </div>
        </DS.Card>
      </div>
      <DS.Modal
        open={modal === "modal"}
        onOpenChange={() => setModal("")}
        title="Product preview"
        description="Accessible modal with focus trapping, Escape to close, and focus restoration."
      >
        <DS.ProductCard
          name="Panadol 500 mg"
          subtitle="UI demo product"
          price={18.5}
        />
      </DS.Modal>
      <DS.ConfirmationModal
        open={modal === "confirm"}
        onOpenChange={() => setModal("")}
        title="Remove demo item?"
        description="This demonstrates a confirmation. No actual product will be deleted."
        onConfirm={() => notify("Demo confirmation completed.")}
      />
      <WorkspaceShowcase />
      <DS.Drawer
        open={modal === "drawer"}
        onOpenChange={() => setModal("")}
        title="Product details"
        description="UI sample · reusable side drawer"
      >
        <div className="stack">
          <DS.ProductCard
            name="Panadol 500 mg"
            subtitle="Paracetamol · 24 tablets"
            price={18.5}
          />
          <DS.BarcodeDisplay value="1234567890123" />
          <DS.Input label="Product name" defaultValue="Panadol 500 mg" />
          <DS.Button
            variant="primary"
            onClick={() => notify("Preview only. No product data was saved.")}
          >
            Preview save
          </DS.Button>
        </div>
      </DS.Drawer>
    </div>
  );
}
