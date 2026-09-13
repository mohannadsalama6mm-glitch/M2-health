import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { Plus, Pencil } from "lucide-react";
import {
  PageHeader,
  DataGrid,
  Status,
  Button,
  Modal,
  BackLink,
  Card,
  DetailList,
  Input,
  Textarea,
  Alert,
  EmptyState,
  LinkButton,
  TableToolbar,
  SearchInput,
  Select,
  SectionHeader,
} from "../../design-system";
import { isDesktopRuntime } from "../../lib/tauri/client";
import { useDemo } from "../../app/DemoContext";
import {
  createCustomer,
  listCustomers,
  getCustomer,
  updateCustomer,
  setCustomerActive,
  createSupplier,
  listSuppliers,
  getSupplier,
  updateSupplier,
  setSupplierActive,
} from "../../lib/tauri/partners";
import type { Customer, Supplier } from "../../lib/tauri/partners.types";
type Partner = Customer | Supplier;
const formatTime = (iso: string) =>
  iso ? iso.slice(0, 16).replace("T", " ") : "—";
export function Partners({ kind }: { kind: "suppliers" | "customers" }) {
  const desktop = isDesktopRuntime();
  const { notify } = useDemo();
  const [rows, setRows] = useState<Partner[]>([]);
  const [search, setSearch] = useState("");
  const [filter, setFilter] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);
  const [editorOpen, setEditorOpen] = useState(false);
  const title = kind === "suppliers" ? "Suppliers" : "Customers";
  useEffect(() => {
    if (!desktop) return;
    let active = true;
    const timer = window.setTimeout(
      () => {
        if (!active) return;
        setLoading(true);
        setError("");
        const query = {
          search: search || null,
          isActive:
            filter === "active" ? true : filter === "inactive" ? false : null,
        };
        const request =
          kind === "suppliers" ? listSuppliers(query) : listCustomers(query);
        request
          .then((result) => {
            if (!active) return;
            setRows(result.items);
            setLoading(false);
          })
          .catch((err: unknown) => {
            if (!active) return;
            setError(
              err instanceof Error
                ? err.message
                : kind === "suppliers"
                  ? "Suppliers could not be loaded."
                  : "Customers could not be loaded.",
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
  }, [desktop, kind, search, filter, attempt]);
  if (!desktop)
    return (
      <EmptyState
        title={
          kind === "suppliers"
            ? "Connect to the desktop app to manage suppliers."
            : "Connect to the desktop app to manage customers."
        }
        description=""
      />
    );
  return (
    <div className="feature-page">
      <PageHeader
        title={title}
        description={
          kind === "suppliers"
            ? "Supplier contacts and purchasing references."
            : "Customer contacts and account references."
        }
        actions={
          <Button variant="primary" onClick={() => setEditorOpen(true)}>
            <Plus size={16} />
            {kind === "suppliers" ? "Add supplier" : "Add customer"}
          </Button>
        }
      />
      {error && (
        <Alert title={`The ${title.toLowerCase()} could not be loaded`} tone="danger">
          {error}
          <Button size="sm" onClick={() => setAttempt((v) => v + 1)}>
            Retry
          </Button>
        </Alert>
      )}
      <TableToolbar>
        <SearchInput
          label={`Search ${title.toLowerCase()}`}
          placeholder={`Search ${title.toLowerCase()}…`}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <div className="row">
          <Select
            label="Status"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
          >
            <option value="">All statuses</option>
            <option value="active">Active</option>
            <option value="inactive">Inactive</option>
          </Select>
        </div>
      </TableToolbar>
      {loading && rows.length === 0 ? (
        <Card>
          <SectionHeader title={`Loading ${title.toLowerCase()}…`} />
        </Card>
      ) : (
        <DataGrid
          label={title}
          rows={rows}
          rowKey={(r) => r.id}
          searchText={(r) => `${r.name} ${r.phone} ${r.email} ${r.code}`}
          columns={[
            {
              key: "name",
              header: "Name / phone",
              render: (r) => (
                <Link className="text-link" to={`/${kind}/${r.id}`}>
                  {r.name}
                  <span className="cell-secondary">{r.phone || "—"}</span>
                </Link>
              ),
            },
            { key: "email", header: "Email", render: (r) => r.email || "—" },
            { key: "address", header: "Address", render: (r) => r.address || "—" },
            { key: "code", header: "Code", render: (r) => r.code },
            {
              key: "status",
              header: "Status",
              render: (r) => (
                <Status value={r.isActive ? "Active" : "Inactive"} />
              ),
            },
          ]}
        />
      )}
      <PartnerEditorDialog
        key={editorOpen ? "open" : "closed"}
        kind={kind}
        open={editorOpen}
        onOpenChange={setEditorOpen}
        partner={null}
        notify={notify}
        onSaved={() => setAttempt((v) => v + 1)}
      />
    </div>
  );
}
function PartnerEditorDialog({
  kind,
  open,
  onOpenChange,
  partner,
  notify,
  onSaved,
}: {
  kind: "suppliers" | "customers";
  open: boolean;
  onOpenChange: (open: boolean) => void;
  partner: Partner | null;
  notify: (message: string) => void;
  onSaved: () => void;
}) {
  const [name, setName] = useState(partner?.name ?? "");
  const [phone, setPhone] = useState(partner?.phone ?? "");
  const [email, setEmail] = useState(partner?.email ?? "");
  const [address, setAddress] = useState(partner?.address ?? "");
  const [notes, setNotes] = useState(partner?.notes ?? "");
  const [error, setError] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const editing = !!partner;
  const isSupplier = kind === "suppliers";
  const label = isSupplier ? "supplier" : "customer";
  const save = () => {
    if (submitting || !name.trim()) return;
    setSubmitting(true);
    setError("");
    const input = {
      name: name.trim(),
      phone: phone.trim() || undefined,
      email: email.trim() || undefined,
      address: address.trim() || undefined,
      notes: notes.trim() || undefined,
    };
    const request = editing
      ? isSupplier
        ? updateSupplier({ id: partner!.id, ...input })
        : updateCustomer({ id: partner!.id, ...input })
      : isSupplier
        ? createSupplier(input)
        : createCustomer(input);
    request
      .then(() => {
        notify(editing ? `${label} updated.` : `${label} added.`);
        onSaved();
        onOpenChange(false);
      })
      .catch((err: unknown) => {
        setSubmitting(false);
        setError(
          err instanceof Error
            ? err.message
            : `The ${label} could not be saved.`,
        );
      });
  };
  return (
    <Modal
      open={open}
      onOpenChange={onOpenChange}
      title={editing ? `Edit ${label}` : `Add ${label}`}
      description={`Codes are generated server-side. Contact details are stored in the local database.`}
    >
      <div className="stack">
        {error && (
          <Alert title={`The ${label} could not be saved`} tone="danger">
            {error}
          </Alert>
        )}
        <Input
          label="Name"
          required
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <Input
          label="Phone"
          type="tel"
          value={phone}
          onChange={(e) => setPhone(e.target.value)}
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
        <Textarea
          label="Notes"
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
        />
        <div className="row end">
          <Button disabled={submitting} onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            variant="primary"
            loading={submitting}
            disabled={!name.trim()}
            onClick={save}
          >
            {editing ? "Save changes" : `Add ${label}`}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
export function PartnerDetails({
  kind,
}: {
  kind: "suppliers" | "customers";
}) {
  const { id } = useParams();
  const desktop = isDesktopRuntime();
  const { notify } = useDemo();
  const [partner, setPartner] = useState<Partner | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);
  const [editorOpen, setEditorOpen] = useState(false);
  const [toggling, setToggling] = useState(false);
  const label = kind === "suppliers" ? "supplier" : "customer";
  useEffect(() => {
    if (!desktop || !id) return;
    let active = true;
    const request = kind === "suppliers" ? getSupplier(id) : getCustomer(id);
    request
      .then((result) => {
        if (!active) return;
        setPartner(result);
        setError("");
        setLoading(false);
      })
      .catch((err: unknown) => {
        if (!active) return;
        setError(
          err instanceof Error
            ? err.message
            : kind === "suppliers"
              ? "The supplier could not be loaded."
              : "The customer could not be loaded.",
        );
        setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [desktop, id, kind, attempt]);
  if (!desktop)
    return (
      <EmptyState
        title={
          kind === "suppliers"
            ? "Connect to the desktop app to manage suppliers."
            : "Connect to the desktop app to manage customers."
        }
        description=""
      />
    );
  if (loading)
    return (
      <Card>
        <SectionHeader title={`Loading ${label}…`} />
      </Card>
    );
  if (error)
    return (
      <>
        <BackLink to={`/${kind}`} label={kind === "suppliers" ? "Suppliers" : "Customers"} />
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
      <EmptyState
        title={`${kind === "suppliers" ? "Supplier" : "Customer"} not found`}
        description="The contact could not be loaded."
        action={
          <LinkButton to={`/${kind}`}>
            Back to {kind === "suppliers" ? "suppliers" : "customers"}
          </LinkButton>
        }
      />
    );
  const activate = (active: boolean) => {
    if (toggling) return;
    setToggling(true);
    const request =
      kind === "suppliers"
        ? setSupplierActive(partner.id, active)
        : setCustomerActive(partner.id, active);
    request
      .then(() => {
        notify(`${label} ${active ? "activated" : "deactivated"}.`);
        setAttempt((v) => v + 1);
      })
      .catch((err: unknown) => {
        notify(
          err instanceof Error
            ? err.message
            : `The ${label} status could not be updated.`,
        );
      })
      .finally(() => setToggling(false));
  };
  return (
    <>
      <BackLink to={`/${kind}`} label={kind === "suppliers" ? "Suppliers" : "Customers"} />
      <div className="feature-page">
        <PageHeader
          title={partner.name}
          description={`${partner.code} · ${partner.isActive ? "Active" : "Inactive"}`}
          actions={
            <>
              <Button onClick={() => setEditorOpen(true)}>
                <Pencil size={15} />
                Edit {label}
              </Button>
              <Button
                variant={partner.isActive ? "secondary" : "primary"}
                loading={toggling}
                onClick={() => activate(!partner.isActive)}
              >
                {partner.isActive ? "Deactivate" : "Activate"}
              </Button>
            </>
          }
        />
        <div className="content-stack">
          <Card>
            <DetailList
              items={[
                { label: "Code", value: partner.code },
                { label: "Name", value: partner.name },
                { label: "Phone", value: partner.phone || "—" },
                { label: "Email", value: partner.email || "—" },
                { label: "Address", value: partner.address || "—" },
                { label: "Notes", value: partner.notes || "—" },
                {
                  label: "Status",
                  value: (
                    <Status value={partner.isActive ? "Active" : "Inactive"} />
                  ),
                },
                { label: "Created", value: formatTime(partner.createdAt) },
                { label: "Updated", value: formatTime(partner.updatedAt) },
              ]}
            />
          </Card>
          <Card>
            <SectionHeader title="Linked documents" />
            <EmptyState
              title="No linked documents yet"
              description={
                kind === "suppliers"
                  ? "Purchases referencing this supplier will be linked here when Reports and ledger history arrive in a later phase."
                  : "Sales referencing this customer will be linked here when Reports and ledger history arrive in a later phase."
              }
            />
          </Card>
        </div>
        <PartnerEditorDialog
          key={editorOpen ? "open" : "closed"}
          kind={kind}
          open={editorOpen}
          onOpenChange={setEditorOpen}
          partner={partner}
          notify={notify}
          onSaved={() => setAttempt((v) => v + 1)}
        />
      </div>
    </>
  );
}