import { useEffect, useState } from "react";
import { Link, useLocation } from "react-router-dom";
import { Plus, Pencil } from "lucide-react";
import {
  Alert,
  BackLink,
  Button,
  Card,
  DetailList,
  EmptyState,
  Input,
  Modal,
  PageHeader,
  SearchInput,
  SectionHeader,
  Select,
  Status,
  Table,
  TableToolbar,
  Textarea,
} from "../../design-system";
import {
  canonicalPartners as api,
  type Customer,
  type CustomerFields,
  type PartnerQuery,
  type Supplier,
  type SupplierFields,
} from "../../lib/tauri/canonical-partners";

type Kind = "suppliers" | "customers";
type Partner = Supplier | Customer;
const size = 20;
const errorText = (e: unknown) =>
  e instanceof Error
    ? e.message
    : typeof e === "object" && e && "message" in e
      ? String(e.message)
      : "The request failed.";
const stamp = (v: string) => v.slice(0, 16).replace("T", " ");
const isSupplier = (p: Partner): p is Supplier => "contactPerson" in p;

export function CanonicalPartners() {
  const { pathname } = useLocation();
  const kind: Kind = pathname.startsWith("/suppliers")
    ? "suppliers"
    : "customers";
  const id = pathname.split("/").filter(Boolean)[1];
  return id ? (
    <Details key={`${kind}-${id}`} kind={kind} id={id} />
  ) : (
    <List key={kind} kind={kind} />
  );
}

function List({ kind }: { kind: Kind }) {
  const supplier = kind === "suppliers";
  const title = supplier ? "Suppliers" : "Customers";
  const [search, setSearch] = useState(""),
    [status, setStatus] = useState(""),
    [page, setPage] = useState(0),
    [attempt, setAttempt] = useState(0);
  const [data, setData] = useState<{
      key: string;
      items: Partner[];
      total: number;
    } | null>(null),
    [error, setError] = useState(""),
    [editor, setEditor] = useState(false),
    [message, setMessage] = useState("");
  const key = JSON.stringify([kind, search, status, page, attempt]);
  useEffect(() => {
    let active = true;
    const timer = setTimeout(
      () => {
        const query: PartnerQuery = {
          search: search || null,
          isActive:
            status === "active" ? true : status === "inactive" ? false : null,
          limit: size,
          offset: page * size,
        };
        const request = supplier
          ? api.listSuppliers(query)
          : api.listCustomers(query);
        request
          .then((r) => {
            if (active) {
              setData({ key, items: r.items, total: r.total });
              setError("");
            }
          })
          .catch((e) => {
            if (active) {
              setData(null);
              setError(errorText(e));
            }
          });
      },
      search ? 200 : 0,
    );
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [key, search, status, page, attempt, supplier]);
  const current = data?.key === key ? data : null;
  const resetSearch = (value: string) => {
    setSearch(value);
    setPage(0);
  };
  return (
    <div className="feature-page canonical-partners">
      <PageHeader
        title={title}
        description={
          supplier
            ? "Canonical supplier identities for future purchasing references."
            : "Canonical business customer identities for future sale linking."
        }
        actions={
          <Button variant="primary" onClick={() => setEditor(true)}>
            <Plus size={16} />
            Add {supplier ? "supplier" : "customer"}
          </Button>
        }
      />
      {message && (
        <Alert title="Saved" tone="success">
          {message}
        </Alert>
      )}
      {error && (
        <Alert title={`${title} could not be loaded`} tone="danger">
          {error}
          <Button size="sm" onClick={() => setAttempt((v) => v + 1)}>
            Retry
          </Button>
        </Alert>
      )}
      <TableToolbar>
        <SearchInput
          label={`Search ${title.toLowerCase()}`}
          placeholder="Code, name, phone or email"
          value={search}
          onChange={(e) => resetSearch(e.target.value)}
        />
        <Select
          label="Partner status"
          showLabel
          value={status}
          onChange={(e) => {
            setStatus(e.target.value);
            setPage(0);
          }}
        >
          <option value="">All statuses</option>
          <option value="active">Active</option>
          <option value="inactive">Inactive</option>
        </Select>
      </TableToolbar>
      {!current ? (
        <Card>
          <p role="status">Loading {title.toLowerCase()}…</p>
        </Card>
      ) : current.items.length === 0 ? (
        <EmptyState
          title={
            search || status
              ? `No ${title.toLowerCase()} match these filters`
              : `No ${title.toLowerCase()} yet`
          }
          description={
            search || status
              ? "Change the search or status filter."
              : `Add the first ${supplier ? "supplier" : "customer"} when ready.`
          }
        />
      ) : (
        <Card>
          <Table
            label={`Canonical ${title.toLowerCase()}`}
            rows={current.items}
            rowKey={(r) => r.id}
            columns={[
              {
                key: "name",
                header: "Name",
                render: (r) => (
                  <Link className="text-link" to={`/${kind}/${r.id}`}>
                    {r.displayName}
                    <span className="cell-secondary">{r.code}</span>
                  </Link>
                ),
              },
              {
                key: "phone",
                header: "Phone",
                render: (r) => r.primaryPhone || "—",
              },
              { key: "email", header: "Email", render: (r) => r.email || "—" },
              ...(supplier
                ? [
                    {
                      key: "contact",
                      header: "Contact",
                      render: (r: Partner) =>
                        isSupplier(r) ? r.contactPerson || "—" : "—",
                    },
                  ]
                : []),
              {
                key: "status",
                header: "Status",
                render: (r) => (
                  <Status value={r.isActive ? "Active" : "Inactive"} />
                ),
              },
            ]}
          />
          <div className="grid-footer">
            <Button disabled={page === 0} onClick={() => setPage((v) => v - 1)}>
              Previous
            </Button>
            <span>
              {current.total} total · Page {page + 1}
            </span>
            <Button
              disabled={(page + 1) * size >= current.total}
              onClick={() => setPage((v) => v + 1)}
            >
              Next
            </Button>
          </div>
        </Card>
      )}
      <Editor
        key={`${kind}-${editor ? "open" : "closed"}`}
        kind={kind}
        partner={null}
        open={editor}
        onOpenChange={setEditor}
        onSaved={(p) => {
          setMessage(`${p.displayName} added.`);
          setAttempt((v) => v + 1);
        }}
      />
    </div>
  );
}

function Details({ kind, id }: { kind: Kind; id: string }) {
  const supplier = kind === "suppliers";
  const label = supplier ? "supplier" : "customer";
  const [partner, setPartner] = useState<Partner | null>(null),
    [error, setError] = useState(""),
    [attempt, setAttempt] = useState(0),
    [editor, setEditor] = useState(false),
    [busy, setBusy] = useState(false),
    [message, setMessage] = useState("");
  useEffect(() => {
    let active = true;
    (supplier ? api.getSupplier(id) : api.getCustomer(id))
      .then((r) => {
        if (active) {
          setPartner(r);
          setError("");
        }
      })
      .catch((e) => {
        if (active) setError(errorText(e));
      });
    return () => {
      active = false;
    };
  }, [supplier, id, attempt]);
  const toggle = async () => {
    if (!partner || busy) return;
    setBusy(true);
    setError("");
    try {
      const input = {
        id: partner.id,
        expectedRevision: partner.revision,
        active: !partner.isActive,
      };
      const next = supplier
        ? await api.setSupplierActive(input)
        : await api.setCustomerActive(input);
      setPartner(next);
      setMessage(`${label} ${next.isActive ? "activated" : "deactivated"}.`);
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };
  if (error && !partner)
    return (
      <>
        <BackLink
          to={`/${kind}`}
          label={supplier ? "Suppliers" : "Customers"}
        />
        <Alert title={`The ${label} could not be loaded`} tone="danger">
          {error}
          <Button size="sm" onClick={() => setAttempt((v) => v + 1)}>
            Retry
          </Button>
        </Alert>
      </>
    );
  if (!partner)
    return (
      <Card>
        <p role="status">Loading {label}…</p>
      </Card>
    );
  return (
    <>
      <BackLink to={`/${kind}`} label={supplier ? "Suppliers" : "Customers"} />
      <div className="feature-page canonical-partners">
        <PageHeader
          title={partner.displayName}
          description={`${partner.code} · Revision ${partner.revision}`}
          actions={
            <>
              <Button onClick={() => setEditor(true)}>
                <Pencil size={15} />
                Edit {label}
              </Button>
              <Button
                variant={partner.isActive ? "secondary" : "primary"}
                loading={busy}
                onClick={toggle}
              >
                {partner.isActive ? "Deactivate" : "Activate"}
              </Button>
            </>
          }
        />
        {message && (
          <Alert title="Updated" tone="success">
            {message}
          </Alert>
        )}
        {error && (
          <Alert title={`The ${label} could not be updated`} tone="danger">
            {error}
          </Alert>
        )}
        <Card>
          <DetailList
            items={[
              { label: "Code", value: partner.code },
              { label: "Name", value: partner.displayName },
              { label: "Primary phone", value: partner.primaryPhone || "—" },
              {
                label: "Secondary phone",
                value: partner.secondaryPhone || "—",
              },
              { label: "Email", value: partner.email || "—" },
              { label: "Address", value: partner.address || "—" },
              { label: "Notes", value: partner.notes || "—" },
              ...(isSupplier(partner)
                ? [
                    {
                      label: "Contact person",
                      value: partner.contactPerson || "—",
                    },
                    {
                      label: "Tax / registration ID",
                      value: partner.taxRegistrationId || "—",
                    },
                  ]
                : []),
              {
                label: "Status",
                value: (
                  <Status value={partner.isActive ? "Active" : "Inactive"} />
                ),
              },
              { label: "Created", value: stamp(partner.createdAt) },
              { label: "Updated", value: stamp(partner.updatedAt) },
            ]}
          />
        </Card>
        <Card>
          <SectionHeader title="Transaction history" />
          <EmptyState
            title="No linked workflow is active yet"
            description={
              supplier
                ? "Phase 5B can reference this stable supplier ID without changing this record."
                : "A later controlled Sales integration can link this customer while preserving sale-time snapshots."
            }
          />
        </Card>
        <Editor
          key={`${partner.id}-${partner.revision}-${editor ? "open" : "closed"}`}
          kind={kind}
          partner={partner}
          open={editor}
          onOpenChange={setEditor}
          onSaved={(p) => {
            setPartner(p);
            setMessage(`${label} updated.`);
          }}
        />
      </div>
    </>
  );
}

function Editor({
  kind,
  partner,
  open,
  onOpenChange,
  onSaved,
}: {
  kind: Kind;
  partner: Partner | null;
  open: boolean;
  onOpenChange: (v: boolean) => void;
  onSaved: (p: Partner) => void;
}) {
  const supplier = kind === "suppliers",
    existingSupplier = partner && isSupplier(partner) ? partner : null;
  const [name, setName] = useState(partner?.displayName ?? ""),
    [primary, setPrimary] = useState(partner?.primaryPhone ?? ""),
    [secondary, setSecondary] = useState(partner?.secondaryPhone ?? ""),
    [email, setEmail] = useState(partner?.email ?? ""),
    [address, setAddress] = useState(partner?.address ?? ""),
    [notes, setNotes] = useState(partner?.notes ?? ""),
    [contact, setContact] = useState(existingSupplier?.contactPerson ?? ""),
    [tax, setTax] = useState(existingSupplier?.taxRegistrationId ?? ""),
    [error, setError] = useState(""),
    [busy, setBusy] = useState(false);
  const optional = (v: string) => v.trim() || null;
  const save = async () => {
    if (busy) return;
    if (!name.trim()) {
      setError("Name is required.");
      return;
    }
    setBusy(true);
    setError("");
    try {
      let result: Partner;
      if (supplier) {
        const fields: SupplierFields = {
          displayName: name,
          contactPerson: optional(contact),
          primaryPhone: optional(primary),
          secondaryPhone: optional(secondary),
          email: optional(email),
          address: optional(address),
          taxRegistrationId: optional(tax),
          notes: optional(notes),
        };
        result = partner
          ? await api.updateSupplier({
              ...fields,
              id: partner.id,
              expectedRevision: partner.revision,
            })
          : await api.createSupplier(fields);
      } else {
        const fields: CustomerFields = {
          displayName: name,
          primaryPhone: optional(primary),
          secondaryPhone: optional(secondary),
          email: optional(email),
          address: optional(address),
          notes: optional(notes),
        };
        result = partner
          ? await api.updateCustomer({
              ...fields,
              id: partner.id,
              expectedRevision: partner.revision,
            })
          : await api.createCustomer(fields);
      }
      onSaved(result);
      onOpenChange(false);
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Modal
      open={open}
      onOpenChange={(v) => {
        if (!busy) onOpenChange(v);
      }}
      title={`${partner ? "Edit" : "Add"} ${supplier ? "supplier" : "customer"}`}
      description="Codes are generated atomically. Empty optional values are stored as null."
    >
      <div className="stack">
        {error && (
          <Alert title="Could not save" tone="danger">
            {error}
          </Alert>
        )}
        <Input
          label="Name"
          required
          autoFocus
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        {supplier && (
          <Input
            label="Contact person"
            value={contact}
            onChange={(e) => setContact(e.target.value)}
          />
        )}
        <Input
          label="Primary phone"
          type="tel"
          value={primary}
          onChange={(e) => setPrimary(e.target.value)}
        />
        <Input
          label="Secondary phone"
          type="tel"
          value={secondary}
          onChange={(e) => setSecondary(e.target.value)}
        />
        <Input
          label="Email"
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
        />
        <Input
          label="Address"
          value={address}
          onChange={(e) => setAddress(e.target.value)}
        />
        {supplier && (
          <Input
            label="Tax / registration ID"
            value={tax}
            onChange={(e) => setTax(e.target.value)}
          />
        )}
        <Textarea
          label="Notes"
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
        />
        <div className="row end">
          <Button disabled={busy} onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            variant="primary"
            loading={busy}
            disabled={!name.trim()}
            onClick={save}
          >
            Save {supplier ? "supplier" : "customer"}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
