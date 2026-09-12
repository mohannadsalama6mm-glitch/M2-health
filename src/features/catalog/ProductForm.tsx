import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { Plus, Trash2 } from "lucide-react";
import {
  BackLink,
  FormSection,
  Input,
  Select,
  Textarea,
  Button,
  LinkButton,
  Alert,
  EmptyState,
  PageHeader,
  Switch,
  Skeleton,
} from "../../design-system";
import { isDesktopRuntime, NativeError } from "../../lib/tauri/client";
import {
  listManufacturers,
  listCategories,
  listRoutes,
  listActiveIngredients,
  getProductDetail,
  createProductFull,
  updateProductFull,
  toMinor,
} from "../../lib/tauri/catalog";
import { useDemo } from "../../app/DemoContext";
import type {
  ActiveIngredient,
  Category,
  CreateProductFull,
  Manufacturer,
  ProductDetail,
  Route,
} from "../../lib/tauri/catalog.types";
type IngredientDraft = {
  key: string;
  activeIngredientId: string;
  strengthText: string;
};
type PackageDraft = {
  key: string;
  id?: string | null;
  packageLabel: string;
  packSize: string;
  unitName: string;
  unitsPerPackage: string;
  strengthText: string;
  isDefault: boolean;
  isActive: boolean;
  barcodes: string[];
  sellingPrice: string;
  costPrice: string;
};
let draftCounter = 0;
function newKey(): string {
  draftCounter += 1;
  return `draft-${Date.now()}-${draftCounter}`;
}
function newIngredient(): IngredientDraft {
  return { key: newKey(), activeIngredientId: "", strengthText: "" };
}
function newPackage(isDefault: boolean): PackageDraft {
  return {
    key: newKey(),
    id: null,
    packageLabel: "",
    packSize: "",
    unitName: "",
    unitsPerPackage: "",
    strengthText: "",
    isDefault,
    isActive: true,
    barcodes: [""],
    sellingPrice: "",
    costPrice: "",
  };
}
export function ProductForm() {
  const { id } = useParams();
  return <ProductEditor key={id ?? "new"} id={id} />;
}
function ProductEditor({ id }: { id?: string }) {
  const desktop = isDesktopRuntime();
  const navigate = useNavigate();
  const { notify } = useDemo();
  const [manufacturers, setManufacturers] = useState<Manufacturer[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [routes, setRoutes] = useState<Route[]>([]);
  const [ingredients, setIngredients] = useState<ActiveIngredient[]>([]);
  const [loading, setLoading] = useState(!!id);
  const [error, setError] = useState("");
  const [notFound, setNotFound] = useState(false);
  const [nameEn, setNameEn] = useState("");
  const [nameAr, setNameAr] = useState("");
  const [scientificName, setScientificName] = useState("");
  const [manufacturerId, setManufacturerId] = useState("");
  const [categoryId, setCategoryId] = useState("");
  const [routeId, setRouteId] = useState("");
  const [description, setDescription] = useState("");
  const [notes, setNotes] = useState("");
  const [isActive, setIsActive] = useState(true);
  const [drug, setDrug] = useState<IngredientDraft[]>([]);
  const [packages, setPackages] = useState<PackageDraft[]>(() => [
    newPackage(true),
  ]);
  const [saving, setSaving] = useState(false);
  useEffect(() => {
    if (!desktop) return;
    let active = true;
    const timer = window.setTimeout(() => {
      if (!active) return;
      Promise.all([
        listManufacturers(),
        listCategories(),
        listRoutes(),
        listActiveIngredients(),
        id ? getProductDetail(id) : Promise.resolve(null),
      ])
        .then(
          ([m, c, r, a, detail]: [
            Manufacturer[],
            Category[],
            Route[],
            ActiveIngredient[],
            ProductDetail | null,
          ]) => {
            if (!active) return;
            setManufacturers(m);
            setCategories(c);
            setRoutes(r);
            setIngredients(a);
            if (detail) {
              const p = detail.product;
              setNameEn(p.commercialNameEn ?? "");
              setNameAr(p.commercialNameAr ?? "");
              setScientificName(p.scientificName ?? "");
              setManufacturerId(p.manufacturerId ?? "");
              setCategoryId(p.categoryId ?? "");
              setRouteId(p.routeId ?? "");
              setDescription(p.description ?? "");
              setNotes(p.notes ?? "");
              setIsActive(p.isActive);
              setDrug(
                detail.activeIngredients.map((i) => ({
                  key: newKey(),
                  activeIngredientId: i.activeIngredient.id,
                  strengthText: i.strengthText ?? "",
                })),
              );
              setPackages(
                detail.packages.length > 0
                  ? detail.packages.map((entry) => ({
                      key: newKey(),
                      id: entry.package.id,
                      packageLabel: entry.package.packageLabel,
                      packSize: entry.package.packSize ?? "",
                      unitName: entry.package.unitName ?? "",
                      unitsPerPackage:
                        entry.package.unitsPerPackage?.toString() ?? "",
                      strengthText: entry.package.strengthText ?? "",
                      isDefault: entry.package.isDefault,
                      isActive: entry.package.isActive,
                      barcodes:
                        entry.barcodes.length > 0
                          ? entry.barcodes.map((b) => b.barcode)
                          : [""],
                      sellingPrice:
                        entry.currentPrice != null
                          ? String(entry.currentPrice.sellingPriceMinor / 100)
                          : "",
                      costPrice:
                        entry.currentPrice?.costPriceMinor != null
                          ? String(entry.currentPrice.costPriceMinor / 100)
                          : "",
                    }))
                  : [newPackage(true)],
              );
            }
            setLoading(false);
          },
        )
        .catch((err: unknown) => {
          if (!active) return;
          setError(
            err instanceof Error
              ? err.message
              : "The form could not be loaded.",
          );
          setNotFound(err instanceof NativeError && err.code === "notFound");
          setLoading(false);
        });
    }, 0);
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [desktop, id]);
  const patchPackage = (key: string, patch: Partial<PackageDraft>) =>
    setPackages((list) =>
      list.map((p) => (p.key === key ? { ...p, ...patch } : p)),
    );
  const setDefaultPackage = (key: string) =>
    setPackages((list) =>
      list.map((p) =>
        p.key === key
          ? { ...p, isDefault: true, isActive: true }
          : { ...p, isDefault: false },
      ),
    );
  const removePackage = (key: string) =>
    setPackages((list) => {
      if (list.length === 1) return list;
      const next = list.filter((p) => p.key !== key);
      if (!next.some((p) => p.isDefault)) next[0].isDefault = true;
      return next;
    });
  const addBarcode = (key: string) =>
    patchPackage(key, {
      barcodes: [...(packages.find((p) => p.key === key)?.barcodes ?? []), ""],
    });
  const submit = async () => {
    if (!nameEn.trim() && !nameAr.trim()) {
      setError("Enter an English or Arabic commercial name.");
      return;
    }
    if (packages.length === 0) {
      setError("Add at least one package.");
      return;
    }
    for (const draft of packages) {
      if (!draft.packageLabel.trim()) {
        setError("Every package needs a package label.");
        return;
      }
      if (
        draft.unitsPerPackage.trim() !== "" &&
        (!/^\d+$/.test(draft.unitsPerPackage.trim()) ||
          Number(draft.unitsPerPackage) <= 0)
      ) {
        setError("Units per package must be a positive whole number.");
        return;
      }
      const selling = toMinor(draft.sellingPrice);
      const cost = toMinor(draft.costPrice);
      if (draft.sellingPrice.trim() !== "" && selling == null) {
        setError(
          "Selling price must be a non-negative EGP amount with at most two decimals.",
        );
        return;
      }
      if (draft.costPrice.trim() !== "" && cost == null) {
        setError(
          "Cost must be a non-negative EGP amount with at most two decimals.",
        );
        return;
      }
      for (const barcode of draft.barcodes) {
        const value = barcode.trim();
        if (value === "") continue;
        if (!/^[\x21-\x7E]{1,128}$/.test(value)) {
          setError(
            "Barcodes must contain 1–128 printable ASCII characters without spaces.",
          );
          return;
        }
      }
    }
    if (packages.filter((p) => p.isDefault).length > 1) {
      setError("Only one package can be the default.");
      return;
    }
    const full = {
      commercialNameEn: nameEn.trim() || null,
      commercialNameAr: nameAr.trim() || null,
      scientificName: scientificName.trim() || null,
      manufacturerId: manufacturerId || null,
      categoryId: categoryId || null,
      routeId: routeId || null,
      description: description.trim() || null,
      notes: notes.trim() || null,
      activeIngredients: drug
        .filter((d) => d.activeIngredientId)
        .map((d) => ({
          activeIngredientId: d.activeIngredientId,
          strengthText: d.strengthText.trim() || null,
        })),
      packages: packages.map((draft) => ({
        id: draft.id || null,
        packageLabel: draft.packageLabel.trim(),
        packSize: draft.packSize.trim() || null,
        unitName: draft.unitName.trim() || null,
        unitsPerPackage:
          draft.unitsPerPackage.trim() !== ""
            ? Number(draft.unitsPerPackage.trim())
            : null,
        strengthText: draft.strengthText.trim() || null,
        isDefault: draft.isDefault,
        isActive: draft.isActive,
        barcodes: draft.barcodes
          .map((b) => b.trim())
          .filter((b) => b !== "")
          .map((b) => ({ barcode: b })),
        sellingPriceMinor: toMinor(draft.sellingPrice),
        costPriceMinor: toMinor(draft.costPrice),
      })),
    } satisfies CreateProductFull;
    setSaving(true);
    try {
      const saved = id
        ? await updateProductFull({ ...full, id, isActive })
        : await createProductFull(full);
      notify(id ? "Product changes saved." : "Product added to the catalog.");
      navigate(`/catalog/${saved.id}`);
    } catch (err: unknown) {
      setError(
        err instanceof Error ? err.message : "The product could not be saved.",
      );
      setSaving(false);
    }
  };
  if (!desktop)
    return (
      <div className="feature-page">
        <BackLink to="/catalog" label="Products" />
        <PageHeader
          title={id ? "Edit product" : "Add product"}
          description="A structured catalog record saved to the local database."
        />
        <Alert title="Desktop app required" tone="warning">
          Products are saved to the local database. Open M² Health as the
          desktop app to make changes.
        </Alert>
      </div>
    );
  if (loading)
    return (
      <div className="feature-page">
        <BackLink to="/catalog" label="Products" />
        <div className="stack" role="status" aria-label="Loading product form">
          <Skeleton />
        </div>
      </div>
    );
  if (notFound)
    return (
      <div className="feature-page">
        <BackLink to="/catalog" label="Products" />
        <EmptyState
          title="Product not found"
          description="Return to the catalog to select a product."
          action={<LinkButton to="/catalog">Back to catalog</LinkButton>}
        />
      </div>
    );
  return (
    <>
      <BackLink to="/catalog" label="Products" />
      <div className="feature-page">
        <PageHeader
          title={id ? "Edit product" : "Add product"}
          description="A structured catalog record saved to the local database."
        />
        <form
          className="content-stack"
          onSubmit={(e) => {
            e.preventDefault();
            void submit();
          }}
        >
          {error && (
            <Alert title="Check the product information" tone="danger">
              {error}
            </Alert>
          )}
          <FormSection
            title="Basic information"
            description="Names and classification used across the workspace."
          >
            <Input
              label="Brand / product name (English)"
              value={nameEn}
              onChange={(e) => setNameEn(e.target.value)}
            />
            <Input
              label="Brand / product name (Arabic)"
              value={nameAr}
              onChange={(e) => setNameAr(e.target.value)}
            />
            <Input
              label="Scientific name"
              value={scientificName}
              onChange={(e) => setScientificName(e.target.value)}
            />
            <Select
              showLabel
              label="Manufacturer"
              value={manufacturerId}
              onChange={(e) => setManufacturerId(e.target.value)}
            >
              <option value="">No manufacturer</option>
              {manufacturers.map((m) => (
                <option key={m.id} value={m.id}>
                  {m.name}
                </option>
              ))}
            </Select>
            <Select
              showLabel
              label="Category"
              value={categoryId}
              onChange={(e) => setCategoryId(e.target.value)}
            >
              <option value="">No category</option>
              {categories.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name}
                </option>
              ))}
            </Select>
            <Select
              showLabel
              label="Route"
              value={routeId}
              onChange={(e) => setRouteId(e.target.value)}
            >
              <option value="">No route</option>
              {routes.map((r) => (
                <option key={r.id} value={r.id}>
                  {r.name}
                </option>
              ))}
            </Select>
            {id && (
              <Switch
                label="Product active"
                checked={isActive}
                onChange={setIsActive}
              />
            )}
            <Textarea
              label="Description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
            />
          </FormSection>
          <FormSection
            title="Active ingredients"
            description="Choose ingredients from the database; strengths are optional."
            action={
              <Button
                size="sm"
                onClick={() => setDrug((list) => [...list, newIngredient()])}
              >
                <Plus size={14} />
                Add ingredient
              </Button>
            }
          >
            {drug.length === 0 ? (
              <p className="muted">No active ingredients selected.</p>
            ) : (
              drug.map((d) => (
                <div className="form-grid" key={d.key}>
                  <Select
                    showLabel
                    label={`Ingredient ${drug.indexOf(d) + 1}`}
                    value={d.activeIngredientId}
                    onChange={(e) =>
                      setDrug((list) =>
                        list.map((item) =>
                          item.key === d.key
                            ? { ...item, activeIngredientId: e.target.value }
                            : item,
                        ),
                      )
                    }
                  >
                    <option value="">Select an ingredient…</option>
                    {ingredients.map((a) => (
                      <option key={a.id} value={a.id}>
                        {a.name}
                      </option>
                    ))}
                  </Select>
                  <div className="row">
                    <Input
                      label={`Ingredient ${drug.indexOf(d) + 1} strength`}
                      hideLabel={false}
                      value={d.strengthText}
                      placeholder="e.g. 500 mg"
                      onChange={(e) =>
                        setDrug((list) =>
                          list.map((item) =>
                            item.key === d.key
                              ? { ...item, strengthText: e.target.value }
                              : item,
                          ),
                        )
                      }
                    />
                    <Button
                      size="sm"
                      disabled={drug.length === 1}
                      aria-label={`Remove ingredient ${drug.indexOf(d) + 1}`}
                      onClick={() =>
                        setDrug((list) =>
                          list.filter((item) => item.key !== d.key),
                        )
                      }
                    >
                      <Trash2 size={14} />
                    </Button>
                  </div>
                </div>
              ))
            )}
          </FormSection>
          {packages.map((draft, index) => (
            <FormSection
              key={draft.key}
              title={`Package ${index + 1}`}
              action={
                <div className="row">
                  <Switch
                    label="Default package"
                    checked={draft.isDefault}
                    onChange={() => setDefaultPackage(draft.key)}
                  />
                  <Button
                    size="sm"
                    disabled={packages.length === 1}
                    aria-label={`Remove package ${index + 1}`}
                    onClick={() => removePackage(draft.key)}
                  >
                    <Trash2 size={14} />
                  </Button>
                </div>
              }
            >
              <Input
                label={`Package ${index + 1} label`}
                value={draft.packageLabel}
                placeholder="Box · 24 tablets"
                onChange={(e) =>
                  patchPackage(draft.key, { packageLabel: e.target.value })
                }
              />
              <Input
                label={`Package ${index + 1} pack size`}
                value={draft.packSize}
                placeholder="e.g. 24 tablets"
                onChange={(e) =>
                  patchPackage(draft.key, { packSize: e.target.value })
                }
              />
              <Input
                label={`Package ${index + 1} unit`}
                value={draft.unitName}
                placeholder="e.g. tablet"
                onChange={(e) =>
                  patchPackage(draft.key, { unitName: e.target.value })
                }
              />
              <Input
                label={`Package ${index + 1} units per package`}
                type="number"
                min={1}
                step={1}
                value={draft.unitsPerPackage}
                onChange={(e) =>
                  patchPackage(draft.key, { unitsPerPackage: e.target.value })
                }
              />
              <Input
                label={`Package ${index + 1} strength`}
                value={draft.strengthText}
                placeholder="e.g. 500 mg"
                onChange={(e) =>
                  patchPackage(draft.key, { strengthText: e.target.value })
                }
              />
              <Switch
                label={`Package ${index + 1} active`}
                checked={draft.isActive}
                onChange={(value) => {
                  patchPackage(draft.key, {
                    isActive: value,
                    isDefault: value ? draft.isDefault : false,
                  });
                }}
              />
              <Input
                label={`Package ${index + 1} selling price (EGP)`}
                type="number"
                step="0.01"
                min={0}
                value={draft.sellingPrice}
                onChange={(e) =>
                  patchPackage(draft.key, { sellingPrice: e.target.value })
                }
              />
              <Input
                label={`Package ${index + 1} cost (EGP)`}
                type="number"
                step="0.01"
                min={0}
                value={draft.costPrice}
                onChange={(e) =>
                  patchPackage(draft.key, { costPrice: e.target.value })
                }
              />
              <div className="repeatable">
                <div className="row spread">
                  <strong>Barcodes</strong>
                  <Button size="sm" onClick={() => addBarcode(draft.key)}>
                    <Plus size={14} />
                    Add barcode
                  </Button>
                </div>
                {draft.barcodes.map((barcode, bIndex) => (
                  <div className="row" key={`${draft.key}-b-${bIndex}`}>
                    <Input
                      label={`Package ${index + 1} barcode ${bIndex + 1}`}
                      value={barcode}
                      placeholder="Print code on the package"
                      onChange={(e) =>
                        patchPackage(draft.key, {
                          barcodes: draft.barcodes.map((value, n) =>
                            n === bIndex ? e.target.value : value,
                          ),
                        })
                      }
                    />
                    <Button
                      size="sm"
                      disabled={draft.barcodes.length === 1}
                      aria-label={`Remove package ${index + 1} barcode ${bIndex + 1}`}
                      onClick={() =>
                        patchPackage(draft.key, {
                          barcodes: draft.barcodes.filter(
                            (_, n) => n !== bIndex,
                          ),
                        })
                      }
                    >
                      <Trash2 size={14} />
                    </Button>
                  </div>
                ))}
              </div>
            </FormSection>
          ))}
          <div>
            <Button
              variant="outline"
              onClick={() =>
                setPackages((list) => [...list, newPackage(false)])
              }
            >
              <Plus size={15} />
              Add package
            </Button>
          </div>
          <FormSection title="Notes">
            <Textarea
              label="Internal product notes"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
            />
          </FormSection>
          <div className="form-actions">
            <LinkButton to="/catalog">Cancel</LinkButton>
            <Button type="submit" variant="primary" loading={saving}>
              {id ? "Save changes" : "Save product"}
            </Button>
          </div>
        </form>
      </div>
    </>
  );
}
