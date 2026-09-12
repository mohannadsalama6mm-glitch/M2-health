import { useState } from "react";
import { Link } from "react-router-dom";
import { Plus, Pencil, Eye, Archive } from "lucide-react";
import {
  FeaturePage,
  LinkButton,
  DataGrid,
  PriceDisplay,
  Status,
  Button,
  ConfirmationModal,
  SummaryStrip,
} from "../../design-system";
import { useDemo } from "../../app/DemoContext";
export function Catalog() {
  const { products, setProducts, notify } = useDemo();
  const [discontinue, setDiscontinue] = useState("");
  return (
    <FeaturePage
      title="Products"
      description="Your pharmacy catalog, from active ingredients to package pricing."
      actions={
        <LinkButton primary to="/catalog/new">
          <Plus size={16} />
          Add product
        </LinkButton>
      }
    >
      <SummaryStrip
        items={[
          {
            label: "Demo catalog",
            value: products.length,
            hint: "Products in this sample",
          },
          {
            label: "Active",
            value: products.filter((p) => p.status === "Active").length,
          },
          {
            label: "Categories",
            value: new Set(products.map((p) => p.category)).size,
          },
          {
            label: "Manufacturers",
            value: new Set(products.map((p) => p.manufacturer)).size,
          },
        ]}
      />
      <DataGrid
        label="Products"
        rows={products}
        rowKey={(p) => p.id}
        searchText={(p) =>
          `${p.name} ${p.scientific} ${(p.barcodes ?? [p.barcode]).join(" ")}`
        }
        filters={[
          {
            label: "Categories",
            options: [...new Set(products.map((p) => p.category))],
            value: (p) => p.category,
          },
          {
            label: "Manufacturers",
            options: [...new Set(products.map((p) => p.manufacturer))],
            value: (p) => p.manufacturer,
          },
          {
            label: "Dosage forms",
            options: [...new Set(products.map((p) => p.form))],
            value: (p) => p.form,
          },
          {
            label: "Statuses",
            options: ["Active", "Inactive"],
            value: (p) => p.status,
          },
        ]}
        columns={[
          {
            key: "name",
            header: "Product / scientific name",
            render: (p) => (
              <Link className="text-link" to={`/catalog/${p.id}`}>
                {p.name}
                <span className="cell-secondary">
                  {p.scientific} · {p.manufacturer}
                </span>
              </Link>
            ),
          },
          {
            key: "pack",
            header: "Package / barcode",
            render: (p) => (
              <>
                {p.pack}
                <span className="cell-secondary">{p.barcode}</span>
              </>
            ),
          },
          { key: "category", header: "Category", render: (p) => p.category },
          {
            key: "price",
            header: "Price",
            render: (p) => <PriceDisplay amount={p.price} />,
          },
          { key: "stock", header: "Stock", render: (p) => p.stock },
          {
            key: "status",
            header: "Status",
            render: (p) => <Status value={p.status} />,
          },
          {
            key: "actions",
            header: "Actions",
            render: (p) => (
              <div className="row">
                <LinkButton to={`/catalog/${p.id}`}>
                  <Eye size={14} />
                  <span className="sr-only">View {p.name}</span>
                </LinkButton>
                <LinkButton to={`/catalog/${p.id}/edit`}>
                  <Pencil size={14} />
                  <span className="sr-only">Edit {p.name}</span>
                </LinkButton>
                <Button
                  size="sm"
                  aria-label={`Discontinue ${p.name}`}
                  disabled={p.status === "Inactive"}
                  onClick={() => setDiscontinue(p.id)}
                >
                  <Archive size={14} />
                </Button>
              </div>
            ),
          },
        ]}
      />
      <ConfirmationModal
        open={!!discontinue}
        onOpenChange={() => setDiscontinue("")}
        title="Discontinue product?"
        description="The product will become inactive in this demo catalog. Existing sample history remains visible. Reloading resets this change."
        onConfirm={() => {
          setProducts(
            products.map((p) =>
              p.id === discontinue ? { ...p, status: "Inactive" } : p,
            ),
          );
          notify("Product marked inactive in this preview.");
        }}
      />
    </FeaturePage>
  );
}
