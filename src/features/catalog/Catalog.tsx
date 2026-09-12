import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Plus, Pencil, Eye, Archive } from "lucide-react";
import {
  PageHeader,
  LinkButton,
  PriceDisplay,
  Status,
  Button,
  ConfirmationModal,
  SummaryStrip,
  Card,
  TableToolbar,
  SearchInput,
  Select,
  Table,
  Pagination,
  Alert,
  Checkbox,
  EmptyState,
  type Column,
} from "../../design-system";
import { isDesktopRuntime } from "../../lib/tauri/client";
import {
  listManufacturers,
  listCategories,
  listRoutes,
  listProducts,
  setProductActive,
  toMajor,
} from "../../lib/tauri/catalog";
import type {
  CatalogSortKey,
  Manufacturer,
  Category,
  Route,
  ProductListItem,
  ProductPage,
} from "../../lib/tauri/catalog.types";
import { useDemo } from "../../app/DemoContext";
const PAGE_SIZE = 10;
function SortHeader({
  label,
  active,
  direction,
  onToggle,
}: {
  label: string;
  active: boolean;
  direction: "asc" | "desc";
  onToggle: () => void;
}) {
  return (
    <button
      type="button"
      className="text-link"
      title={
        active
          ? `Sorted by ${label.toLowerCase()} (${direction}). Click to reverse.`
          : `Sort by ${label.toLowerCase()}`
      }
      onClick={onToggle}
    >
      {label}
      <span aria-hidden="true">
        {active ? (direction === "asc" ? " ↑" : " ↓") : ""}
      </span>
    </button>
  );
}
export function Catalog() {
  const desktop = isDesktopRuntime();
  const { notify } = useDemo();
  const [manufacturers, setManufacturers] = useState<Manufacturer[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [routes, setRoutes] = useState<Route[]>([]);
  const [search, setSearch] = useState("");
  const [manufacturerId, setManufacturerId] = useState("");
  const [categoryId, setCategoryId] = useState("");
  const [routeId, setRouteId] = useState("");
  const [includeInactive, setIncludeInactive] = useState(false);
  const [sort, setSort] = useState<CatalogSortKey>("name");
  const [sortDirection, setSortDirection] = useState<"asc" | "desc">("asc");
  const [page, setPage] = useState(1);
  const [data, setData] = useState<ProductPage | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [discontinue, setDiscontinue] = useState("");
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    if (!desktop) return;
    let active = true;
    Promise.all([listManufacturers(), listCategories(), listRoutes()])
      .then(([m, c, r]) => {
        if (!active) return;
        setManufacturers(m);
        setCategories(c);
        setRoutes(r);
      })
      .catch(() => {
        // The product query below reports its own error; filter menus just stay empty.
      });
    return () => {
      active = false;
    };
  }, [desktop]);
  useEffect(() => {
    if (!desktop) return;
    let active = true;
    const timer = window.setTimeout(
      () => {
        if (!active) return;
        setLoading(true);
        setError("");
        listProducts({
          search: search || null,
          manufacturerId: manufacturerId || null,
          categoryId: categoryId || null,
          routeId: routeId || null,
          includeInactive,
          sort,
          sortDirection,
          limit: PAGE_SIZE,
          offset: (page - 1) * PAGE_SIZE,
        })
          .then((result) => {
            if (!active) return;
            setData(result);
            setLoading(false);
          })
          .catch((err: unknown) => {
            if (!active) return;
            setError(
              err instanceof Error
                ? err.message
                : "The catalog could not be loaded.",
            );
            setLoading(false);
          });
      },
      search ? 250 : 0,
    );
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [
    desktop,
    search,
    manufacturerId,
    categoryId,
    routeId,
    includeInactive,
    sort,
    sortDirection,
    page,
    attempt,
  ]);
  const toggleSort = (key: CatalogSortKey) => {
    if (sort === key) setSortDirection((d) => (d === "asc" ? "desc" : "asc"));
    else {
      setSort(key);
      setSortDirection("asc");
    }
  };
  const pages = data ? Math.max(1, Math.ceil(data.total / data.limit)) : 1;
  const columns: Column<ProductListItem>[] = [
    {
      key: "name",
      header: (
        <SortHeader
          label="Product / scientific name"
          active={sort === "name" || sort === "scientific"}
          direction={sortDirection}
          onToggle={() =>
            toggleSort(sort === "scientific" ? "scientific" : "name")
          }
        />
      ),
      render: (p) => (
        <Link className="text-link" to={`/catalog/${p.id}`}>
          {p.nameEn ?? p.nameAr ?? "Unnamed product"}
          <span className="cell-secondary">
            {[p.scientificName, p.manufacturerName].filter(Boolean).join(" · ")}
          </span>
        </Link>
      ),
    },
    {
      key: "pack",
      header: "Package / barcode",
      render: (p) => (
        <>
          {p.packageLabel ?? "No package"}
          <span className="cell-secondary">
            {[p.packSize, p.barcode].filter(Boolean).join(" · ")}
          </span>
        </>
      ),
    },
    {
      key: "category",
      header: (
        <SortHeader
          label="Category"
          active={sort === "category"}
          direction={sortDirection}
          onToggle={() => toggleSort("category")}
        />
      ),
      render: (p) => p.categoryName ?? "—",
    },
    {
      key: "price",
      header: (
        <SortHeader
          label="Price"
          active={sort === "price"}
          direction={sortDirection}
          onToggle={() => toggleSort("price")}
        />
      ),
      render: (p) =>
        p.sellingPriceMinor != null ? (
          <PriceDisplay amount={toMajor(p.sellingPriceMinor)} />
        ) : (
          <span className="muted">—</span>
        ),
    },
    {
      key: "status",
      header: "Status",
      render: (p) => <Status value={p.isActive ? "Active" : "Inactive"} />,
    },
    {
      key: "actions",
      header: "Actions",
      render: (p) => (
        <div className="row">
          <LinkButton to={`/catalog/${p.id}`}>
            <Eye size={14} />
            <span className="sr-only">View {p.nameEn}</span>
          </LinkButton>
          <LinkButton to={`/catalog/${p.id}/edit`}>
            <Pencil size={14} />
            <span className="sr-only">Edit {p.nameEn}</span>
          </LinkButton>
          <Button
            size="sm"
            aria-label={`Discontinue ${p.nameEn}`}
            disabled={!p.isActive}
            onClick={() => setDiscontinue(p.id)}
          >
            <Archive size={14} />
          </Button>
        </div>
      ),
    },
  ];
  return (
    <div className="feature-page">
      <PageHeader
        title="Products"
        description="Your pharmacy catalog, from active ingredients to package pricing."
        actions={
          <LinkButton primary to="/catalog/new">
            <Plus size={16} />
            Add product
          </LinkButton>
        }
      />
      {!desktop && (
        <Alert title="Desktop app required" tone="warning">
          The catalog reads from the local database. Open M² Health as the
          desktop app to browse products.
        </Alert>
      )}
      <SummaryStrip
        items={[
          { label: "Products", value: data ? data.total : "—" },
          { label: "Active", value: data ? data.activeTotal : "—" },
          { label: "Categories", value: categories ? categories.length : "—" },
          {
            label: "Manufacturers",
            value: manufacturers ? manufacturers.length : "—",
          },
        ]}
      />
      {desktop && error && (
        <Alert title="The catalog could not be loaded" tone="danger">
          {error}
          <Button size="sm" onClick={() => setAttempt((v) => v + 1)}>
            Retry
          </Button>
        </Alert>
      )}
      {!desktop ? (
        <Card>
          <EmptyState
            title="Catalog requires the desktop app"
            description="Products and pricing are stored in the local SQLite database and can only be browsed inside M² Health."
          />
        </Card>
      ) : (
        <Card>
          <TableToolbar>
            <SearchInput
              label="Search Products"
              placeholder="Search products…"
              value={search}
              onChange={(e) => {
                setSearch(e.target.value);
                setPage(1);
              }}
            />
            <div className="row">
              <Select
                label="Manufacturer"
                value={manufacturerId}
                onChange={(e) => {
                  setManufacturerId(e.target.value);
                  setPage(1);
                }}
              >
                <option value="">All manufacturers</option>
                {manufacturers.map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.name}
                  </option>
                ))}
              </Select>
              <Select
                label="Category"
                value={categoryId}
                onChange={(e) => {
                  setCategoryId(e.target.value);
                  setPage(1);
                }}
              >
                <option value="">All categories</option>
                {categories.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.name}
                  </option>
                ))}
              </Select>
              <Select
                label="Route"
                value={routeId}
                onChange={(e) => {
                  setRouteId(e.target.value);
                  setPage(1);
                }}
              >
                <option value="">All routes</option>
                {routes.map((r) => (
                  <option key={r.id} value={r.id}>
                    {r.name}
                  </option>
                ))}
              </Select>
              <Checkbox
                label="Include inactive"
                checked={includeInactive}
                onChange={(e) => {
                  setIncludeInactive(e.target.checked);
                  setPage(1);
                }}
              />
            </div>
          </TableToolbar>
          {loading && !data ? (
            <div className="stack" role="status">
              <Alert title="Loading products…" tone="info" />
            </div>
          ) : (
            <Table
              label="Products"
              rows={data?.items ?? []}
              columns={columns}
              rowKey={(p) => p.id}
            />
          )}
          <div className="grid-footer">
            <span className="muted">
              {data ? `${data.total} results · local SQLite` : "—"}
            </span>
            <Pagination page={page} pages={pages} onChange={setPage} />
          </div>
        </Card>
      )}
      <ConfirmationModal
        open={!!discontinue}
        onOpenChange={() => setDiscontinue("")}
        title="Discontinue product?"
        description="The product will become inactive. Its record and history remain in the database."
        onConfirm={() => {
          void setProductActive(discontinue, false)
            .then(() => {
              notify("Product marked inactive.");
              setAttempt((v) => v + 1);
            })
            .catch((err: unknown) =>
              setError(
                err instanceof Error
                  ? err.message
                  : "The product could not be updated.",
              ),
            );
        }}
      />
    </div>
  );
}
