import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { Pencil } from "lucide-react";
import {
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
  PageHeader,
  Alert,
  Skeleton,
  Badge,
  type Column,
} from "../../design-system";
import { isDesktopRuntime } from "../../lib/tauri/client";
import {
  getProductDetail,
  getPackagePriceHistory,
  toMajor,
} from "../../lib/tauri/catalog";
import type {
  PackagePrice,
  ProductDetail,
} from "../../lib/tauri/catalog.types";
function productTitle(detail: ProductDetail): string {
  return (
    detail.product.commercialNameEn ??
    detail.product.commercialNameAr ??
    detail.product.scientificName ??
    "Unnamed product"
  );
}
export function ProductDetails() {
  const { id } = useParams();
  const desktop = isDesktopRuntime();
  const [detail, setDetail] = useState<ProductDetail | null>(null);
  const [history, setHistory] = useState<PackagePrice[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    if (!desktop || !id) return;
    let active = true;
    const timer = window.setTimeout(() => {
      if (!active) return;
      setLoading(true);
      setError("");
      getProductDetail(id)
        .then(async (result) => {
          if (!active) return;
          setDetail(result);
          const fallback =
            result.packages.find((p) => p.package.isDefault) ??
            result.packages[0];
          if (fallback)
            setHistory(await getPackagePriceHistory(fallback.package.id));
          setLoading(false);
        })
        .catch((err: unknown) => {
          if (!active) return;
          setError(
            err instanceof Error
              ? err.message
              : "The product could not be loaded.",
          );
          setLoading(false);
        });
    }, 0);
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [desktop, id, attempt]);
  if (!desktop || !id)
    return (
      <div className="feature-page">
        <BackLink to="/catalog" label="Products" />
        <PageHeader
          title="Product details"
          description="Product records come from the local database."
        />
        <Alert title="Desktop app required" tone="warning">
          Product details come from the local database. Open M² Health as the
          desktop app to view them.
        </Alert>
      </div>
    );
  if (loading)
    return (
      <div className="feature-page">
        <BackLink to="/catalog" label="Products" />
        <div className="stack" role="status" aria-label="Loading product">
          <Skeleton />
        </div>
      </div>
    );
  if (error && !detail)
    return (
      <div className="feature-page">
        <BackLink to="/catalog" label="Products" />
        <Alert title="Product not found" tone="danger">
          {error}
          <LinkButton to="/catalog">Back to catalog</LinkButton>
        </Alert>
      </div>
    );
  if (!detail)
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
  const { product } = detail;
  const defaultPackage =
    detail.packages.find((p) => p.package.isDefault) ?? detail.packages[0];
  const rows: Column<PackagePrice>[] = [
    {
      key: "effectiveFrom",
      header: "Effective date",
      render: (r) => r.effectiveFrom,
    },
    {
      key: "selling",
      header: "Selling price",
      render: (r) => <PriceDisplay amount={toMajor(r.sellingPriceMinor)} />,
    },
    {
      key: "cost",
      header: "Cost",
      render: (r) =>
        r.costPriceMinor != null ? (
          <PriceDisplay amount={toMajor(r.costPriceMinor)} />
        ) : (
          <span className="muted">—</span>
        ),
    },
    {
      key: "reason",
      header: "Reason",
      render: (r) => r.reason ?? <span className="muted">—</span>,
    },
  ];
  return (
    <>
      <BackLink to="/catalog" label="Products" />
      <div className="feature-page">
        <PageHeader
          title={productTitle(detail)}
          description={[
            product.scientificName,
            detail.manufacturer?.name,
            detail.category?.name,
            detail.route?.name,
          ]
            .filter(Boolean)
            .join(" · ")}
          actions={
            <>
              <Status value={product.isActive ? "Active" : "Inactive"} />
              <LinkButton to={`/catalog/${product.id}/edit`}>
                <Pencil size={15} />
                Edit product
              </LinkButton>
            </>
          }
        />
        {error && (
          <Alert title="This product could not be reloaded" tone="danger">
            {error}
            <ButtonLink onClick={() => setAttempt((v) => v + 1)}>
              Retry
            </ButtonLink>
          </Alert>
        )}
        <Card>
          <Tabs
            items={[
              {
                label: "General information",
                content: (
                  <DetailList
                    items={[
                      {
                        label: "Brand name",
                        value: product.commercialNameEn ?? "—",
                      },
                      {
                        label: "Arabic name",
                        value: product.commercialNameAr ?? "—",
                      },
                      {
                        label: "Scientific name",
                        value: product.scientificName ?? "—",
                      },
                      {
                        label: "Manufacturer",
                        value: detail.manufacturer?.name ?? "—",
                      },
                      {
                        label: "Category",
                        value: detail.category?.name ?? "—",
                      },
                      { label: "Route", value: detail.route?.name ?? "—" },
                      {
                        label: "Active ingredients",
                        value:
                          detail.activeIngredients.length > 0
                            ? detail.activeIngredients
                                .map(
                                  (i) =>
                                    `${i.activeIngredient.name}${i.strengthText ? ` · ${i.strengthText}` : ""}`,
                                )
                                .join(", ")
                            : "—",
                      },
                      {
                        label: "Description",
                        value: product.description ?? "—",
                      },
                      { label: "Notes", value: product.notes ?? "—" },
                    ]}
                  />
                ),
              },
              {
                label: "Packages & barcodes",
                content: (
                  <div className="stack">
                    {detail.packages.length === 0 ? (
                      <EmptyState
                        title="No packages yet"
                        description="Add a package from the edit screen."
                      />
                    ) : (
                      detail.packages.map((entry) => (
                        <Card key={entry.package.id}>
                          <div className="equal-columns">
                            <DetailList
                              items={[
                                {
                                  label: "Package",
                                  value: entry.package.packageLabel,
                                },
                                {
                                  label: "Pack size",
                                  value: entry.package.packSize ?? "—",
                                },
                                {
                                  label: "Unit",
                                  value: entry.package.unitName ?? "—",
                                },
                                {
                                  label: "Units per package",
                                  value:
                                    entry.package.unitsPerPackage?.toString() ??
                                    "—",
                                },
                                {
                                  label: "Strength",
                                  value: entry.package.strengthText ?? "—",
                                },
                                {
                                  label: "Status",
                                  value: (
                                    <div className="row">
                                      {entry.package.isDefault && (
                                        <Badge tone="info">Default</Badge>
                                      )}
                                      <Status
                                        value={
                                          entry.package.isActive
                                            ? "Active"
                                            : "Inactive"
                                        }
                                      />
                                    </div>
                                  ),
                                },
                                {
                                  label: "Selling price",
                                  value:
                                    entry.currentPrice != null ? (
                                      <PriceDisplay
                                        amount={toMajor(
                                          entry.currentPrice.sellingPriceMinor,
                                        )}
                                      />
                                    ) : (
                                      <span className="muted">—</span>
                                    ),
                                },
                                {
                                  label: "Cost",
                                  value:
                                    entry.currentPrice?.costPriceMinor !=
                                    null ? (
                                      <PriceDisplay
                                        amount={toMajor(
                                          entry.currentPrice.costPriceMinor,
                                        )}
                                      />
                                    ) : (
                                      <span className="muted">—</span>
                                    ),
                                },
                              ]}
                            />
                            <div className="stack">
                              {entry.barcodes.length === 0 ? (
                                <p className="muted">No barcodes assigned.</p>
                              ) : (
                                entry.barcodes.map((code) => (
                                  <BarcodeDisplay
                                    key={code.id}
                                    value={code.barcode}
                                  />
                                ))
                              )}
                            </div>
                          </div>
                        </Card>
                      ))
                    )}
                  </div>
                ),
              },
              {
                label: "Price history",
                content: defaultPackage ? (
                  <Table
                    label="Product price history"
                    rows={history ?? []}
                    rowKey={(r) => r.id}
                    columns={rows}
                  />
                ) : (
                  <EmptyState
                    title="No price history"
                    description="Set a price on a package to see its history here."
                  />
                ),
              },
            ]}
          />
        </Card>
      </div>
    </>
  );
}
function ButtonLink({
  children,
  onClick,
}: {
  children: React.ReactNode;
  onClick: () => void;
}) {
  return (
    <button type="button" className="text-link" onClick={onClick}>
      {children}
    </button>
  );
}
