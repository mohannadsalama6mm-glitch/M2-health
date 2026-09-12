import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import {
  FeaturePage,
  BackLink,
  FormSection,
  Input,
  Select,
  Textarea,
  RepeatableRows,
  Button,
  LinkButton,
  Alert,
  EmptyState,
} from "../../design-system";
import { useDemo } from "../../app/DemoContext";
import type { Product } from "../../mock/types";
export function ProductForm() {
  const { id } = useParams();
  return <ProductEditor key={id ?? "new"} id={id} />;
}
// Remount the draft on route identity changes; the shared catalog stays in memory.
function ProductEditor({ id }: { id?: string }) {
  const { products, setProducts, notify } = useDemo();
  const p = products.find((p) => p.id === id);
  const navigate = useNavigate();
  const [ingredients, setIngredients] = useState(p?.ingredients ?? [""]),
    [packages, setPackages] = useState(p?.packages ?? [p?.pack ?? ""]),
    [barcodes, setBarcodes] = useState(p?.barcodes ?? [p?.barcode ?? ""]),
    [error, setError] = useState("");
  if (id && !p)
    return (
      <EmptyState
        title="Product not found"
        description="Return to the catalog to select a product."
      />
    );
  return (
    <>
      <BackLink to="/catalog" label="Products" />
      <FeaturePage
        title={p ? "Edit product" : "Add product"}
        description="A structured catalog record. All changes stay in this UI session."
      >
        <form
          className="content-stack"
          onSubmit={(e) => {
            e.preventDefault();
            const data = new FormData(e.currentTarget);
            const name = String(data.get("name")).trim();
            if (!name) {
              setError("Enter a product name.");
              return;
            }
            if (barcodes.some((b) => !/^\d{8,14}$/.test(b))) {
              setError("Each barcode must contain 8–14 digits.");
              return;
            }
            if (new Set(barcodes).size !== barcodes.length) {
              setError("Each package barcode must be unique.");
              return;
            }
            if (
              products.some(
                (other) =>
                  other.id !== id &&
                  (other.barcodes ?? [other.barcode]).some((code) =>
                    barcodes.includes(code),
                  ),
              )
            ) {
              setError(
                "That barcode is already assigned to another demo product.",
              );
              return;
            }
            const next: Product = {
              id: p?.id ?? `p-${Date.now()}`,
              name,
              scientific: String(data.get("scientific")),
              manufacturer: String(data.get("manufacturer")),
              category: String(data.get("category")),
              form: String(data.get("form")),
              strength: String(data.get("strength")),
              barcodes,
              packages,
              barcode: barcodes[0],
              pack: packages.join(" / "),
              price: Number(data.get("price")),
              cost: Number(data.get("cost")),
              stock: p?.stock ?? 0,
              reorder: Number(data.get("reorder")),
              status: p?.status ?? "Active",
              batch: p?.batch ?? "Not assigned",
              expiry: p?.expiry ?? "Not assigned",
              days: p?.days ?? 365,
              ingredients,
              notes: String(data.get("notes")),
            };
            setProducts(
              p
                ? products.map((v) => (v.id === p.id ? next : v))
                : [next, ...products],
            );
            notify("Product saved in memory only.");
            navigate(`/catalog/${next.id}`);
          }}
        >
          {error && (
            <Alert tone="danger" title="Check the product information">
              {error}
            </Alert>
          )}
          <FormSection
            title="Basic information"
            description="Names and classification used across the workspace."
          >
            <Input
              name="name"
              label="Brand / product name"
              defaultValue={p?.name}
              required
            />
            <Input
              name="scientific"
              label="Scientific name"
              defaultValue={p?.scientific}
              required
            />
            <Input
              name="manufacturer"
              label="Manufacturer"
              defaultValue={p?.manufacturer}
              required
            />
            <Select
              showLabel
              name="category"
              label="Category"
              defaultValue={p?.category}
            >
              {[
                "Pain relief",
                "Antibiotics",
                "Vitamins",
                "Allergy care",
                "Respiratory",
                "Digestive care",
                "Medical devices",
                "Cardiovascular",
              ].map((c) => (
                <option key={c}>{c}</option>
              ))}
            </Select>
          </FormSection>
          <FormSection title="Drug details">
            <Input
              name="strength"
              label="Strength"
              defaultValue={p?.strength}
              required
            />
            <Select
              showLabel
              name="form"
              label="Dosage form"
              defaultValue={p?.form}
            >
              {[
                "Tablet",
                "Capsule",
                "Syrup",
                "Inhaler",
                "Softgel",
                "Sachet",
                "Device",
              ].map((f) => (
                <option key={f}>{f}</option>
              ))}
            </Select>
            <RepeatableRows
              label="Ingredients"
              values={ingredients}
              onChange={setIngredients}
              placeholder="Active ingredient"
            />
          </FormSection>
          <FormSection title="Packages & identification">
            <RepeatableRows
              label="Packages"
              values={packages}
              onChange={setPackages}
              placeholder="Box · 24 tablets"
            />
            <RepeatableRows
              label="Barcodes"
              values={barcodes}
              onChange={setBarcodes}
              placeholder="8–14 digits"
            />
          </FormSection>
          <FormSection title="Pricing & inventory defaults">
            <Input
              type="number"
              step="0.01"
              min={0}
              name="cost"
              label="Cost (EGP)"
              defaultValue={p?.cost ?? 0}
              required
            />
            <Input
              type="number"
              step="0.01"
              min={0}
              name="price"
              label="Selling price (EGP)"
              defaultValue={p?.price ?? 0}
              required
            />
            <Input
              type="number"
              min={0}
              name="reorder"
              label="Reorder level"
              defaultValue={p?.reorder ?? 10}
              required
            />
            <Input
              label="Opening stock"
              value="Set through Inventory adjustments"
              disabled
            />
          </FormSection>
          <FormSection title="Notes">
            <Textarea
              name="notes"
              label="Internal product notes"
              defaultValue={p?.notes}
            />
          </FormSection>
          <div className="form-actions">
            <LinkButton to="/catalog">Cancel</LinkButton>
            <Button type="submit" variant="primary">
              Save demo product
            </Button>
          </div>
        </form>
      </FeaturePage>
    </>
  );
}
