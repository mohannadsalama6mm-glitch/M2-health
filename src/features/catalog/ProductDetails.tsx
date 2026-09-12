import { useParams, Link } from "react-router-dom";
import { Pencil } from "lucide-react";
import {
  FeaturePage,
  BackLink,
  LinkButton,
  Card,
  Tabs,
  DetailList,
  PriceDisplay,
  Status,
  Table,
  BarcodeDisplay,
  EmptyState,
  SectionHeader,
} from "../../design-system";
import { useDemo } from "../../app/DemoContext";
import { movements } from "../../mock/fixtures";
export function ProductDetails() {
  const { id } = useParams();
  const { products, suppliers } = useDemo();
  const p = products.find((p) => p.id === id);
  if (!p)
    return (
      <EmptyState
        title="Product not found"
        description="This product is not in the demo catalog."
        action={<LinkButton to="/catalog">Back to catalog</LinkButton>}
      />
    );
  return (
    <>
      <BackLink to="/catalog" label="Products" />
      <FeaturePage
        title={p.name}
        description={`${p.scientific} · ${p.strength} · ${p.form}`}
        actions={
          <>
            <Status value={p.status} />
            <LinkButton to={`/catalog/${p.id}/edit`}>
              <Pencil size={15} />
              Edit product
            </LinkButton>
          </>
        }
      >
        <Card>
          <Tabs
            items={[
              {
                label: "General information",
                content: (
                  <DetailList
                    items={[
                      { label: "Brand name", value: p.name },
                      { label: "Scientific name", value: p.scientific },
                      { label: "Manufacturer", value: p.manufacturer },
                      { label: "Dosage form", value: p.form },
                      { label: "Strength", value: p.strength },
                      { label: "Category", value: p.category },
                      {
                        label: "Active ingredients",
                        value: p.ingredients.join(", "),
                      },
                      { label: "Notes", value: p.notes },
                    ]}
                  />
                ),
              },
              {
                label: "Packages & barcodes",
                content: (
                  <div className="equal-columns">
                    <DetailList
                      items={[
                        { label: "Package", value: p.pack },
                        {
                          label: "Selling price",
                          value: <PriceDisplay amount={p.price} />,
                        },
                        {
                          label: "Cost",
                          value: <PriceDisplay amount={p.cost} />,
                        },
                        { label: "Barcode", value: p.barcode },
                      ]}
                    />
                    <div className="stack">
                      {(p.barcodes ?? [p.barcode]).map((code) => (
                        <BarcodeDisplay key={code} value={code} />
                      ))}
                    </div>
                  </div>
                ),
              },
              {
                label: "Price history",
                content: (
                  <Table
                    label="Demo price history"
                    rows={[
                      {
                        date: "11 Sep 2026",
                        price: p.price,
                        reason: "Current demo price",
                      },
                      {
                        date: "01 Aug 2026",
                        price: p.price - 2,
                        reason: "Previous sample price",
                      },
                    ]}
                    rowKey={(r) => r.date}
                    columns={[
                      {
                        key: "date",
                        header: "Effective date",
                        render: (r) => r.date,
                      },
                      {
                        key: "price",
                        header: "Selling price",
                        render: (r) => <PriceDisplay amount={r.price} />,
                      },
                      {
                        key: "reason",
                        header: "Reason",
                        render: (r) => r.reason,
                      },
                    ]}
                  />
                ),
              },
              {
                label: "Stock & movements",
                content: (
                  <div className="stack">
                    <DetailList
                      items={[
                        { label: "Available stock", value: p.stock },
                        { label: "Reorder level", value: p.reorder },
                        { label: "Batch", value: p.batch },
                        { label: "Expiry", value: p.expiry },
                      ]}
                    />
                    <LinkButton to="/inventory/movements">
                      View movement ledger
                    </LinkButton>
                    <Table
                      label="Recent product movements"
                      rows={movements.filter((m) => m.product === p.name)}
                      rowKey={(r) => r.id}
                      columns={[
                        { key: "date", header: "Date", render: (r) => r.date },
                        {
                          key: "type",
                          header: "Movement",
                          render: (r) => r.type,
                        },
                        { key: "in", header: "In", render: (r) => r.incoming },
                        {
                          key: "out",
                          header: "Out",
                          render: (r) => r.outgoing,
                        },
                      ]}
                    />
                  </div>
                ),
              },
              {
                label: "Supplier references",
                content: (
                  <div className="stack">
                    {suppliers.slice(0, 2).map((s) => (
                      <Card key={s.id}>
                        <SectionHeader
                          title={s.name}
                          action={
                            <Link to={`/suppliers/${s.id}`}>
                              Supplier details
                            </Link>
                          }
                        />
                        <p className="muted">
                          {s.phone} · {s.email}
                        </p>
                      </Card>
                    ))}
                  </div>
                ),
              },
            ]}
          />
        </Card>
      </FeaturePage>
    </>
  );
}
